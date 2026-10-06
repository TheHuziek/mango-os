# Mango OS

A lightweight operating system project built in C and assembly, designed to explore kernel development, system architecture, and low-level software.

## Overview

Mango OS is a custom operating system project focused on:

- booting from a minimal kernel
- managing memory and processes
- supporting basic device interaction
- creating a clean, extensible OS architecture

This project is intended for learning and experimentation, and it can be expanded with drivers, a filesystem, userland programs, and more.

## Features

- Minimal kernel boot flow
- x86-compatible kernel foundation
- Custom memory and interrupt handling
- Extensible build system
- Easy local testing with emulation

## Project Structure

```text
mango-os/
├── boot/          # Bootloader and startup code
├── kernel/        # Core kernel implementation
├── include/       # Public headers
├── src/           # Source files
├── drivers/       # Device drivers
├── lib/           # Runtime/library helpers
├── tools/         # Build and utility scripts
├── Makefile       # Build entry point
├── README.md      # Project documentation
└── LICENSE        # License information
```

## Requirements

Before building Mango OS, make sure you have:

- GCC or Clang
- GNU Make
- NASM (if assembly files are used)
- QEMU (for running the OS in a virtual machine)
- A Unix-like environment or WSL on Windows

## Getting Started

Clone the repository:

```bash
git clone https://github.com/yourusername/mango-os.git
cd mango-os
```

Build the project:

```bash
make
```

Run the OS in QEMU:

```bash
qemu-system-x86_64 -kernel build/kernel.bin
```

If the project uses a different boot target, follow the build instructions in the Makefile or project scripts.

## Development Notes

- Keep the kernel small and modular.
- Prefer clear abstractions for memory, interrupts, and scheduler code.
- Test changes using an emulator before running on real hardware.
- Document new APIs and drivers as the project grows.

## Roadmap

- Add a working bootloader and kernel entry point
- Implement basic memory management
- Add interrupt and exception handling
- Build a simple process scheduler
- Support keyboard and display drivers
- Add a basic shell or userland environment
- Expand with filesystem support

## Contributing

Contributions are welcome. If you want to improve the project:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the relevant checks and tests
5. Submit a pull request with a clear description

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Contact

For questions, suggestions, or project updates, open an issue in the repository or contact the maintainer.

---

Built with the goal of learning, experimenting, and creating a small but functional operating system from the ground up.
