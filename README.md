# Operating System in Rust

My personal mirror of the [blog-os](http://os.phil-opp.com). All credits goes to the creator. I'm just learning from the blog and documenting how I understand things.

It may not be perfect. It may not even be complete as I am only writing for myself and the level of detail will facilitate my learning agenda.\
For a comprehensive guide and detailed steps please go to https://os.phil-opp.com
<hr>

[![Build Status](https://github.com/pratiksampat/rust_os/workflows/Build%20Code/badge.svg?branch=02-Minimal-rust-kernel)](https://github.com/pratiksampat/rust_os/actions?query=branch%3A02-Minimal-rust-kernel)

# 02: A minimal Rust kernel

> Run with: `$ cargo xrun`

In this post we'll build upon the [bare-metal rust binary](https://github.com/pratiksampat/rust_os/tree/01-running-bare-metal) and try to create a bootable disk image, that prints "`Hello, World!`" on the screen.

We will define our custom target, use the VGA buffer, supply a bootloader, attach it to our kernel and finally boot on QEMU.

## 1. Basics on how Booting works

When the computer is first turned on, it executes some self-testing code which detects the RAM and pre-initializes the CPU and hardware. After that it looks for a bootable disk and starts executing the OS from there.

The firmware standard used here is `BIOS` (Basic Input/Output system). Although `UEFI` is newer, `BIOS` is simpler.

### <u>Bios Boot</u>

* Computer is turned on
* `BIOS` is loaded from a special flash memory from the motherboard
* `BIOS` runs self tests, initialization routines and looks for bootable disks
* If it finds one, the control is transferred to its bootloader which is a 512 byte portion of executable code which can point to lager memory storage eventually
* Bootloader performs the following:
  * Determine the location of kernel image on disks
  * is responsible for the CPU switch from `16 bit real mode` to `32 bit protected mode`, and then to the `64 bit long mode`
  * Query information like the memory map from the BIOS and pass it to the Operating System kernel.

We use a tool called [bootImage](https://github.com/rust-osdev/bootimage) that automatically prepends a bootloader to the kernel.

## 2. Custom target specification

In the previous post we used `thumbv7em-none-eabihf` target triple but as you'll see we need a few more tweaks, that'll justify creating our own target triple in [x86_64-rust_os.json](x86_64-rust_os.json)

Picked from the x86 default configurations

```JSON
{
  "llvm-target": "x86_64-unknown-none",
  // defines the size of various integers
  "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "os": "none",
  "executables": true,
}
```

Instead of using the platform's default linker (which might not support Linux targets), we use the cross platform LLD linker that is shipped with Rust for linking our kernel.

```JSON
"linker-flavor": "ld.lld",
"linker": "rust-lld",
```

This setting specifies that the target doesn't support stack unwinding on panic. Now we can remove this attrubute from Cargo.toml

```JSON
"panic-strategy": "abort",
```

We'll need to handle interrupts at some point, so we need to disable some stack pointer optimizations called the `red zone` because it can cause stack corruption otherwise.

More about [red zone](https://os.phil-opp.com/red-zone/).

```JSON
"disable-redzone": true,
```

The `mmx` and `sse` determines support for `SIMD` (Single Instruction Multiple data) instructions, which can speed the programs up.

However, this can lead to problems in the OS, because we need to save and restore multiple registers which can be in the order of (512-1600 bytes), we disable this feature.

Also, floating point operations need `SIMD` by default and therefore use `soft-float` to emulate the instructions in software.

```JSON
"features": "-mmx,-sse,+soft-float"
```

## 3. Building the kernel

Now, that we have defined our own target running we can try building using

```Bash
$cargo build --target x86_64-rust_os.json

error[E0463]: can't find crate for `core`
```

It tells us that the rust compiler no longer finds the core library which contains the basic Rust types such as `Result`, `Option`, and `iterators` and is implicitly linked to all `no_std` crates.\
The problem is that the core library is distributed together with the Rust compiler as a precompiled library which is only valid for supported triples and not our custom target. We need to recompile `core` for these targets.

### Cargo xbuild

We use `cargo xbuild`. It is essentially a wrapper for `cargo build` that automatically cross-compiles `core` and another built-in libraries.\
Install:

```Bash
cargo install cargo-xbuild
```

Then run,

```Bash
cargo xbuild --target x86_64-rust_os.json
```

## 4. Printing to screen

The VGA buffer is located at `0xb8000`. We iterate over the String and we write to the vga raw pointer. To the offset we write the byte string and at the next address we write the color. In this case `0xb` which is light cyan.

Note that we do not want to introduce unsafe rust in our main program and it absolutely must be abstracted out.

## 5. Running the kernel

### Creating a bootimage

We use the `bootloader` crate to accomplish this

```TOML
# Cargo.toml

[dependencies]
bootloader = "0.8.0"
```

However, we still to link the bootloader and the kernel. For that, we use a tool called `bootimage`, that compiles kernel and the bootloader and links them together to create a bootable disk image. To install the tool:

```Bash
cargo install bootimage --version "^0.7.7"
```

Also:

```Bash
rustup component add llvm-tools-preview
```

Create a bootable disk image using

```Bash
cargo bootimage
```

### Booting on qemu

```Bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rust_os/debug/bootimage-rust_os.bin
```

### Using `Cargo run`

We can set a runner configuration key in for cargo:

```TOML
# .cargo/config

[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```

Run using

```Bash
cargo xrun
```
