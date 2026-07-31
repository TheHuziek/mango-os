use core::fmt::{self, Write};
#[derive(Debug)]
pub struct Uart {
    uart_base: usize,
}
impl Uart {
    pub fn new(uart_base: usize) -> Self {
        Uart {
            uart_base: uart_base,
        }
    }
    pub fn put_char(&mut self, c: u8) {
        let ptr = self.uart_base as *mut u8;
        unsafe {
            core::ptr::write_volatile(ptr, c);
        }
    }

    // Nuevo: Lee un carácter si hay uno disponible en el buffer
    pub fn get_char(&mut self) -> Option<u8> {
        let lsr_ptr = (self.uart_base + 5) as *const u8; // Line Status Register
        unsafe {
            // Si el bit 0 está encendido, hay datos listos
            if (core::ptr::read_volatile(lsr_ptr) & 1) != 0 {
                Some(core::ptr::read_volatile(self.uart_base as *const u8))
            } else {
                None
            }
        }
    }

    // Nuevo: Le dice al UART que genere una interrupción al recibir datos
    pub fn enable_rx_interrupt(&mut self) {
        let ier_ptr = (self.uart_base + 1) as *mut u8; // Interrupt Enable Register
        unsafe {
            // Escribir un 1 en el bit 0 activa la interrupción de recepción
            core::ptr::write_volatile(ier_ptr, 0x01);
        }
    }
}
// Implementar core::fmt::Write nos regala la capacidad de usar macros de Rust
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.put_char(byte);
        }
        Ok(())
    }
}
