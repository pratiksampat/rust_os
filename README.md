# Operating System in Rust

My personal mirror of the [blog-os](http://os.phil-opp.com). All credits goes to the creator. I'm just learning from the blog and documenting how I understand things.

It may not be perfect. It may not even be complete as I am only writing for myself and the level of detail will facilitate my learning agenda.\
For a comprehensive guide and detailed steps please go to https://os.phil-opp.com
<hr>

[![Build Status](https://github.com/pratiksampat/rust_os/workflows/Build%20Code/badge.svg?branch=01-running-bare-metal)](https://github.com/pratiksampat/rust_os/actions?query=branch%3A01-running-bare-metal)

# 01: A bare-metal rust binary

The standard library provides a lot of useful features like threads, files, heap memory, network etc.

When we are going to write our own Operating system, the OS cannot depend upon the standard library that Rust provides, simply because these standard library features are operating system dependent and here we are writing our own and we know we don't support all those fancy things yet.

Therefore, firstly we need to make a freestanding/bare-metal binary that is operating system independent.

## 1. Disable the standard library using

```Rust
// main.rs
#![no_std]
```

This means that we can't use `println!` anymore too in the `main()`. Get rid of that too.

## 2. Implement a panic handler.

The `panic_handler` attribute defines the function that the compiler should invoke when a `panic!()` occurs. The standard library offers.
Currently we just loop in there.

The signature of the panic handler is as below:
```Rust
// main.rs
fn panic(_info: &PanicInfo) -> ! {}
```

An interesting thing to note here is that it returns ["never" type](https://doc.rust-lang.org/nightly/std/primitive.never.html) `!`. Which means that it never resolves to any values at all. It makes the function "diverging". Useful in scenarios like `exit` where it exits the process without returning.

## 3. The `eh_personality` Language item

Language items are special functions and types that are required internally by the compiler. Example `Copy` trait is a language item which tells the compiler that which type have the copy semantics.

The `eh_personality` language item marks a function that is used for implementing stack unwinding. By default Rust uses unwinding to run the destructor of all live variables in case of a `panic`. This ensures all memory is freed correctly. However, this feature is OS specific and sadly must go too.

Add the following lines [`Cargo.toml`](Cargo.toml)

```toml
# Cargo.toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

This sets the `panic` strategy to `abort` and the `eh_personality` langauge item is no longer required.

## 4. Get rid of `main` and add the `start` attribute

`main` is not the first function that is called when the program is first run. There is a thing called runtimes which call `main`. Rust uses the C runtime library called the `crt0` (C runtime zero), which sets up the environment for a C application. This includes creating a stack and placing the arguments in the right registers. The C runtime then invokes the entry point of the Rust runtime, which is marked by the start language item. Rust only has a very minimal runtime, which takes care of some small things such as setting up stack overflow guards or printing a backtrace on panic. The runtime then finally calls the main function.

We do not have the `crt0` and hence we will overwrite it and call it `_start`

```Rust
// main.rs
#[no_mangle]
pub extern "C" fn _start() -> ! {}
```

We use `#[no_mangle]` to retain the name `_start` otherwise Rust might generate some name for it. We need the name `_start` because we need to tell the linker where the entry point is.

Also the function is marked with `extern "C"` to tell the compiler to use the C calling convention instead of Rusts' (which is unspecified)

## 5. Linking

We need to build for a bare-metal target. By default Rust will try to build for the host system. By compiling for the host, it will assume that we are compiling for a system that already has an OS and which uses the C runtime by default.

An example for such a bare metal environment is the `thumbv7em-none-eabihf` which describes an embedded ARM system.

We will define a custom target later but for now to compile use:

```
$ rustup target add thumbv7em-none-eabihf
$ cargo build --target thumbv7em-none-eabihf
```

### Compiling by changing the linker

Use:
```
cargo rustc -- -C link-arg=-nostartfiles
```
Now we don't need to specify the name of our entry point function explicitly since the linker looks for a function with the name `_start` by default.

This is probably not a good idea as our executable still expects various things, for example that a stack is initialized when the `_start` function is called. Without the C runtime, some of these requirements might not be fulfilled, which might cause our program to fail, e.g. through a segmentation fault.