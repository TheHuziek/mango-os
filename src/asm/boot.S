.section .text.init
.global _start

_start:
    # Desactivamos interrupciones por ahora
    csrw sie, zero

    # Configuramos el Stack Pointer (sp) apuntando al tope de nuestra pila
    la sp, _stack_top

    # OpenSBI ya nos dejó el hartid en a0 y el DTB en a1.
    # Al llamar a rust_main, estos registros actuarán como los primeros
    # dos argumentos de nuestra función en Rust por la convención de llamadas.
    call rust_main

    # Si rust_main retorna (no debería), nos quedamos en un loop infinito
1:  wfi
    j 1b
