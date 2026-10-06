use bitflags::bitflags;
use core::arch::asm;
use crate::physical_allocator;

bitflags! {
    /// Banderas de permisos y estado para una PTE en RISC-V
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PteFlags: u64 {
        const VALID    = 1 << 0; // (V) Indica si la PTE es válida
        const READ     = 1 << 1; // (R) Permiso de lectura
        const WRITE    = 1 << 2; // (W) Permiso de escritura
        const EXECUTE  = 1 << 3; // (X) Permiso de ejecución
        const USER     = 1 << 4; // (U) Accesible desde User Mode
        const GLOBAL   = 1 << 5; // (G) Mapeo global (no se borra del TLB)
        const ACCESSED = 1 << 6; // (A) El hardware marca si fue accedida
        const DIRTY    = 1 << 7; // (D) El hardware marca si fue escrita
    }
}
/// Representa una tabla de páginas completa (sirve igual para L2, L1 o L0)
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Crea una tabla llena de ceros (512 entradas inválidas)
    pub const fn empty() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }
    pub fn map(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: PteFlags,
        // Se acepta un closure que entregue un frame físico (u64) o None si no hay memoria
        allocator: &mut impl FnMut() -> Option<u64>,
    ) -> Result<(), &'static str> {
        // Extraer los 3 índices de 9 bits para Sv39
        let vpn2 = ((virt_addr >> 30) & 0x1FF) as usize;
        let vpn1 = ((virt_addr >> 21) & 0x1FF) as usize;
        let vpn0 = ((virt_addr >> 12) & 0x1FF) as usize;

        let vpns = [vpn2, vpn1]; // Índices para L2 y L1
        let mut current_table = self as *mut PageTable;

        // Recorrer los niveles intermedios (L2 -> L1)
        for &vpn in vpns.iter() {
            let pte = unsafe { &mut (*current_table).entries[vpn] };

            if !pte.is_valid() {
                // ¡La rama no existe! Pedir un frame físico al asignador
                let new_frame_phys = allocator().ok_or("Sin memoria física")?;
                
                // Limpiar con ceros la nueva memoria (asumiendo Identity Mapping temporal)
                let new_table_ptr = new_frame_phys as *mut PageTable;
                unsafe {
                    core::ptr::write_bytes(new_table_ptr, 0, 1);
                }

                // Configurar el PTE para apuntar a la nueva tabla (V=1, permisos=0)
                let ppn = new_frame_phys >> 12;
                pte.set(ppn, PteFlags::VALID);
            }

            // Mover el puntero de la tabla actual al siguiente nivel
            // (La dirección física del siguiente nivel es ppn * 4096)
            let next_table_phys = pte.ppn() << 12;
            
            // ATENCIÓN: Esta conversión asume Identity Mapping o que sabes 
            // cómo traducir direcciones físicas a virtuales dentro del kernel.
            current_table = next_table_phys as *mut PageTable;
        }

        // Configurar el nodo hoja (L0)
        let leaf_pte = unsafe { &mut (*current_table).entries[vpn0] };
        
        // Configurar los permisos y el PPN final (bits 12..55 de la dirección física)
        let final_ppn = phys_addr >> 12;
        leaf_pte.set(final_ppn, flags | PteFlags::VALID);

        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Crea una entrada vacía (inválida por defecto, ya que V=0)
    pub const fn new() -> Self {
        Self(0)
    }

    /// Comprueba si la entrada es válida
    pub fn is_valid(&self) -> bool {
        (self.0 & PteFlags::VALID.bits()) != 0
    }

    /// Retorna el Physical Page Number (PPN).
    /// El PPN empieza en el bit 10 y ocupa 44 bits en Sv39.
    pub fn ppn(&self) -> u64 {
        (self.0 >> 10) & ((1 << 44) - 1)
    }

    /// Retorna las banderas configuradas en los primeros 8 bits
    pub fn flags(&self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0)
    }

    /// Configura la PTE. Mezcla el PPN (desplazado) y las banderas.
    pub fn set(&mut self, ppn: u64, flags: PteFlags) {
        // Máscara para evitar que un PPN corrupto sobreescriba bits reservados
        let clean_ppn = ppn & ((1 << 44) - 1);
        self.0 = (clean_ppn << 10) | flags.bits();
    }
}

/// Habilita la MMU en modo Sv39.
/// `root_table_physical_addr` debe estar alineada a 4KB (4096 bytes).
pub unsafe fn enable_mmu(root_table_physical_addr: usize) {
    // 1. Validar alineación
    assert!(root_table_physical_addr % 4096 == 0, "La tabla de páginas no está alineada a 4KB");

    // 2. Calcular el PPN (Physical Page Number)
    // Desplazamos 12 bits a la derecha para eliminar el offset de la página.
    let ppn = root_table_physical_addr >> 12;

    // 3. Configurar el registro satp
    // Formato satp 64-bit: [ Mode (4 bits) | ASID (16 bits) | PPN (44 bits) ]
    // Mode = 8 representa Sv39.
    let mode = 8usize << 60;
    let asid = 0usize << 44; // Address Space ID (0 para el espacio del kernel)
    let satp_val = mode | asid | ppn;

    // 4. Escribir en satp y purgar el TLB
    asm!(
        "csrw satp, {0}",
        "sfence.vma", // Invalida las entradas antiguas en el caché de traducción
        in(reg) satp_val,
        options(nostack)
    );
}