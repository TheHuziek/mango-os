#![no_std]
#![no_main]
use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use fdt::Fdt; // Importamos las herramientas de formateo

global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/trapvector.S"));
unsafe extern "C" {
    fn trap_vector();
}

mod physical_allocator;
mod uart;
use crate::uart::Uart;
//use fdt::Fdt;
//inicializar el plic
const PLIC_BASE: usize = 0x0c00_0000;
const UART_IRQ: u32 = 10; // QEMU asigna el IRQ 10 al UART0

const PLIC_PRIORITY: usize = PLIC_BASE + (UART_IRQ as usize) * 4;
const PLIC_SENABLE: usize = PLIC_BASE + 0x2080;
const PLIC_STHRESHOLD: usize = PLIC_BASE + 0x20_1000;
const PLIC_SCLAIM: usize = PLIC_BASE + 0x20_1004;
// La dirección de memoria donde QEMU mapea el UART 16550A
const UART_BASE: usize = 0x1000_0000;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(_hartid: usize, dtb: usize) -> ! {
    let mut uart = Uart::new(UART_BASE);
    let _ = writeln!(uart, "inicializando las interrupciones");
    //1 configurar el uart para comunicacion
    uart.enable_rx_interrupt();
    unsafe {
        // le damos prioridad 1 a uart para que importe tiene que ser mayor a 0
        core::ptr::write_volatile(PLIC_PRIORITY as *mut u32, 1);
        //habilitar el IRQ 10 en S-Mode
        core::ptr::write_volatile(PLIC_SENABLE as *mut u32, 1 << UART_IRQ);
        //aceptar interrupciones de cualquier prioridad mayor a 0
        core::ptr::write_volatile(PLIC_STHRESHOLD as *mut u32, 0);
    }
    // 3. Configurar CPU (CSRs)
    unsafe {
        // Declaramos que la etiqueta 'trap_vector' existe en ensamblador

        // stvec: Apuntamos el vector de interrupciones a nuestro código en boot.S
        asm!("csrw stvec, {}", in(reg) trap_vector as *const() as usize);

        // sie: Habilitamos "External Interrupts" (Bit 9)
        asm!("csrs sie, {}", in(reg) 1 << 9);

        // sstatus: Habilitamos interrupciones globales para S-mode (Bit 1)
        asm!("csrs sstatus, {}", in(reg) 1 << 1);
    }
    
    let fdt = unsafe {
        Fdt::from_ptr(dtb as *const u8)
            .expect("El puntero DTB es inválido o el formato es incorrecto")
    };
    let _ = writeln!(uart, "hay {} cpus", fdt.cpus().count());
    let memory = fdt.memory();
    let mut start: usize = 0x80000000;
    let mut size: usize = 0x4000000;
    for region in memory.regions() {
        start = region.starting_address as usize;
        size = region.size.unwrap_or(0);
        let _ = writeln!(uart, "start:{:#x} size:{:#x}", start, size);

        // Aquí es donde enviarías 'start' y 'size' a tu Asignador Físico
        // (tu Free List o Bitmap) para inicializarlo.
        // init_physical_allocator(start, size);
    }

    let _ = writeln!(uart, "Escribe algo! El kernel hara eco.");
    let _ = writeln!(uart, "el device tree esta en {:#x}", fdt.total_size());
    let _ = writeln!(
        uart,
        "el kernel termina en {:#x}",
        rust_main as *const () as usize
    );
    let mut physical_allocator: physical_allocator::PhysicalAllocator =
        physical_allocator::PhysicalAllocator::new();

    physical_allocator.init_allocator(start, size, rust_main as *const () as usize);
    let root_page=physical_allocator.alloc().unwrap();
    enable_mmu(root_page);
    loop {
        unsafe { asm!("wfi") }
    }
}
 fn enable_mmu(root_table_phys_addr: usize) {
    // 1. Validar alineación a nivel de página (4 KiB)
    assert!(root_table_phys_addr % 4096 == 0, "La tabla raíz no está alineada a 4K");

    // 2. Calcular el Physical Page Number (PPN)
    let ppn = root_table_phys_addr / 4096;

    // 3. Construir el valor del registro satp
    // MODE = 8 (Sv39) se desplaza al bit 60
    let mode_sv39: usize = 8 << 60;
    
    // ASID = 0 (bits 44 a 59 quedan en 0)
    let satp_val = mode_sv39 | ppn;

    // 4. Escribir en el CSR satp
    unsafe {core::arch::asm!("csrw satp, {}", in(reg) satp_val);
    // satp::write(satp_val);

    // 5. Ejecutar sfence.vma (vaciado del TLB)
    // El primer argumento indica la dirección virtual (0 = todas)
    // El segundo indica el ASID (0 = todos)
    asm!("sfence.vma zero, zero", options(nostack, preserves_flags));}
}
fn read_time() -> u64 {
    let mut time: u64;
    unsafe { core::arch::asm!("csrr {} ,time",out(reg) time) }
    time
}
// Leer el tiempo actual desde el registro CSR 'time' (solo lectura en S-mode)
fn sbi_timer_set(stime_value: u64) {
    let _error: isize;
    let _value: usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inout("a0") stime_value as usize => _error,
            lateout("a1") _value,
            in("a6") 0,          // FID = 0
            in("a7") 0x5449_4D45, // EID = TIME
        );
    }
}
// Este es el código que se ejecuta cada vez que ocurre una interrupción
fn uart_interrupt_handler() {
    // Reutilizamos una sola instancia
    let mut uart = Uart::new(UART_BASE);

    // Leemos todos los caracteres disponibles en la FIFO
    while let Some(c) = uart.get_char() {
        if c == b'\r' {
            let _ = writeln!(uart, "");
        } else {
            uart.put_char(c);
        }
    }
}
#[unsafe(no_mangle)]

pub extern "C" fn rust_trap_handler() {
    let cause: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) cause);
    }

    // En RISC-V 64-bit, el bit MSB (bit 63) indica si es interrupción o excepción
    let is_interrupt = (cause & (1 << 63)) != 0;
    let code = cause & !(1 << 63);

    if is_interrupt {
        match code {
            9 => {
                // 1. Reclamar (Claim) la interrupción en el PLIC
                let irq = unsafe { core::ptr::read_volatile(PLIC_SCLAIM as *const u32) };

                if irq == UART_IRQ {
                    uart_interrupt_handler();
                }

                // 2. Completar (Complete): Notificar al PLIC que la IRQ fue atendida
                if irq != 0 {
                    unsafe {
                        core::ptr::write_volatile(PLIC_SCLAIM as *mut u32, irq);
                    }
                }
            }
            _ => {}
        }
    } else {
        // Manejar excepciones aquí
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = Uart::new(UART_BASE);
    let _ = writeln!(uart, "\n[KERNEL PANIC]");
    let _ = writeln!(uart, "{}", info);
    loop {
        unsafe { asm!("wfi") }
    }
}
