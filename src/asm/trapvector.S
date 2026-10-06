
.section .text
.global trap_vector
.align 4                    # stvec requiere alineación a 4 bytes (MODE=Direct)

trap_vector:
    # 1. Reservar espacio en el stack (31 registros * 8 bytes = 248 bytes, alineado a 16 bytes = 256 bytes)
    addi sp, sp, -256

    # 2. Guardar registros de propósito general (x1 a x31, omitiendo x0 que siempre es 0)
    sd ra,   0*8(sp)
    # sp (x2) original no se guarda directamente aquí o se ajusta si es necesario
    sd gp,   2*8(sp)
    sd tp,   3*8(sp)
    sd t0,   4*8(sp)
    sd t1,   5*8(sp)
    sd t2,   6*8(sp)
    sd s0,   7*8(sp)
    sd s1,   8*8(sp)
    sd a0,   9*8(sp)
    sd a1,  10*8(sp)
    sd a2,  11*8(sp)
    sd a3,  12*8(sp)
    sd a4,  13*8(sp)
    sd a5,  14*8(sp)
    sd a6,  15*8(sp)
    sd a7,  16*8(sp)
    sd s2,  17*8(sp)
    sd s3,  18*8(sp)
    sd s4,  19*8(sp)
    sd s5,  20*8(sp)
    sd s6,  21*8(sp)
    sd s7,  22*8(sp)
    sd s8,  23*8(sp)
    sd s9,  24*8(sp)
    sd s10, 25*8(sp)
    sd s11, 26*8(sp)
    sd t3,  27*8(sp)
    sd t4,  28*8(sp)
    sd t5,  29*8(sp)
    sd t6,  30*8(sp)

    # 3. Llamar al manejador en Rust
    call rust_trap_handler

    # 4. Restaurar todos los registros guardados
    ld ra,   0*8(sp)
    ld gp,   2*8(sp)
    ld tp,   3*8(sp)
    ld t0,   4*8(sp)
    ld t1,   5*8(sp)
    ld t2,   6*8(sp)
    ld s0,   7*8(sp)
    ld s1,   8*8(sp)
    ld a0,   9*8(sp)
    ld a1,  10*8(sp)
    ld a2,  11*8(sp)
    ld a3,  12*8(sp)
    ld a4,  13*8(sp)
    ld a5,  14*8(sp)
    ld a6,  15*8(sp)
    ld a7,  16*8(sp)
    ld s2,  17*8(sp)
    ld s3,  18*8(sp)
    ld s4,  19*8(sp)
    ld s5,  20*8(sp)
    ld s6,  21*8(sp)
    ld s7,  22*8(sp)
    ld s8,  23*8(sp)
    ld s9,  24*8(sp)
    ld s10, 25*8(sp)
    ld s11, 26*8(sp)
    ld t3,  27*8(sp)
    ld t4,  28*8(sp)
    ld t5,  29*8(sp)
    ld t6,  30*8(sp)

    # 5. Liberar el espacio reservado en el stack
    addi sp, sp, 256

    # 6. Regresar del modo Supervisor (restaura sstatus.SIE y salta a sepc)
    sret