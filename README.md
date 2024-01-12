# NES-emulator

> [!WARNING]
> This software is unfinished. Keep your expectations low.

A basic NES emulator.

# Features

- Almost complete 6502 CPU implementation with just some illegal opcodes missing
- Support to a subset of the [iNES](https://www.nesdev.org/wiki/INES) file
  format. (only reads the PRG ROM)

## Build from Source

```console
$ cargo build
$ cargo run <file-name>
```
