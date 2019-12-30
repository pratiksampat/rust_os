# Operating System in Rust

My personal mirror of the [blog-os](http://os.phil-opp.com). All credits goes to the creator. I'm just learning from the blog and documenting how I understand things.

It may not be perfect. It may not even be complete as I am only writing for myself and the level of detail will facilitate my learning agenda.\
For a comprehensive guide and detailed steps please go to https://os.phil-opp.com
<hr>

# [01: A bare-metal rust binary](https://github.com/pratiksampat/rust_os/tree/01-running-bare-metal)

The standard library provides a lot of useful features like threads, files, heap memory, network etc.

When we are going to write our own Operating system, the OS cannot depend upon the standard library that Rust provides, simply because these standard library features are operating system dependent and here we are writing our own and we know we don't support all those fancy things yet.

Therefore, firstly we need to make a freestanding/bare-metal binary that is operating system independent.

Code and more explanation for the following on the branch: [01-running-bare-metal](https://github.com/pratiksampat/rust_os/tree/01-running-bare-metal)

# [02: A minimal Rust kernel](https://github.com/pratiksampat/rust_os/tree/02-Minimal-rust-kernel)


In this post we'll build upon the [bare-metal rust binary](https://github.com/pratiksampat/rust_os/tree/01-running-bare-metal) and try to create a bootable disk image, that prints "`Hello, World!`" on the screen.

We will define our custom target, use the 
* VGA buffer to print `Hello, World!`
* Supply a bootloader
* Link it to our kernel
* Finally boot on QEMU.

Code and more explanation for the following on the branch: [02-Minimal-Rust-kernel](https://github.com/pratiksampat/rust_os/tree/02-Minimal-rust-kernel)

# [03: VGA text mode abstraction](https://github.com/pratiksampat/rust_os/tree/03-VGA-text-mode-abstraction)


In this post we'll build upon the [Minimal Rust kernel](https://github.com/pratiksampat/rust_os/tree/02-Minimal-rust-kernel), we will take the concept forward of writing on the screen by encapsulating all the unsafety out in a separate module and also introduce support for the Rust's formatting macros.

Code and more explanation for the following on the branch: [03-VGA-text-mode-abstraction](https://github.com/pratiksampat/rust_os/tree/03-VGA-text-mode-abstraction)
