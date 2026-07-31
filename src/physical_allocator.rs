const PAGE_SIZE: usize = 4096;

// Esta estructura se escribirá al principio de cada marco físico libre de 4 KB.
#[repr(C)]
struct FreeBlock {
    next: *mut FreeBlock,
}

pub struct PhysicalAllocator {
    head: *mut FreeBlock,
}

impl PhysicalAllocator {
    // Inicializador constante para poder usarlo en una variable estática global
    pub const fn new() -> Self {
        PhysicalAllocator {
            head: core::ptr::null_mut(),
        }
    }
    pub fn alloc(&mut self) -> Option<usize> {
        if self.head.is_null() {
            return None; // Nos quedamos sin memoria física
        }

        unsafe {
            // Guardamos la dirección del bloque actual
            let current_block = self.head;

            // Avanzamos la cabeza al siguiente bloque disponible
            self.head = (*current_block).next;

            // Retornamos la dirección del bloque que acabamos de sacar
            Some(current_block as usize)
        }
    }
    pub unsafe fn free(&mut self, address: usize) {
        // 1. Validar que la dirección esté alineada a 4 KB
        assert!(
            address % PAGE_SIZE == 0,
            "La dirección física no está alineada"
        );

        // 2. Convertir la dirección física en un puntero a nuestro FreeBlock
        let block = address as *mut FreeBlock;

        // 3. Apuntar el 'next' de este nuevo bloque a la cabeza actual
        (*block).next = self.head;

        // 4. Actualizar la cabeza para que apunte a este nuevo bloque
        self.head = block;
    }
    pub fn init_allocator(&mut self, ram_start: usize, ram_size: usize, end_of_kernel: usize) {
        let kernel_end = core::ptr::addr_of!(end_of_kernel) as usize;

        // Calcular el primer bloque de 4 KB alineado DESPUÉS de tu kernel
        let mut current_addr = PhysicalAllocator::align_up(kernel_end, PAGE_SIZE);
        let end_addr = ram_start + ram_size;

        unsafe {
            while current_addr + PAGE_SIZE <= end_addr {
                // Insertar cada marco en el asignador
                self.free(current_addr);
                current_addr += PAGE_SIZE;
            }
        }
    }
    // Función auxiliar para alinear direcciones hacia arriba
    pub fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
    fn malloc() {}
}
