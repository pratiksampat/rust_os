# Operating System in Rust

My personal mirror of the [blog-os](http://os.phil-opp.com). All credits goes to the creator. I'm just learning from the blog and documenting how I understand things.

It may not be perfect. It may not even be complete as I am only writing for myself and the level of detail will facilitate my learning agenda.\
For a comprehensive guide and detailed steps please go to https://os.phil-opp.com
<hr>

# 03: VGA text mode abstraction


In this post we'll build upon the [Minimal Rust kernel](https://github.com/pratiksampat/rust_os/tree/02-Minimal-rust-kernel), we will take the concept forward of writing on the screen by encapsulating all the unsafety out in a separate module and also introduce support for the Rust's formatting macros.

## 1. Introduction to the VGA text buffer

```text
--------------------------------------------------------------------
|                Attribute         |            Character          |
====================================================================
| 7    | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
--------------------------------------------------------------------
|Blink |  Bg color |  Fg color     |           Code point          |
--------------------------------------------------------------------
```

16 bit array entry. First 8 bits(byte) define the ASCII code print.

The second byte determines how the character should be displayed. The color map is encoded as an enum within [vga_buffer](src/vga_buffer.rs).

The VGA text buffer is accessible via `memory mapped IO` (MMIO) to the address `0xb8000`. This means that the reads and writes to that address don't access the RAM, but directly the text buffer on the VGA hardware. This text buffer treats it as normal reads and writes so we don't have to threat them in a special way.

## 2. [The Rust VGA module](src/vga_buffer.rs)

### a. `Color` Enum

While defining the enum of colors we use the following attributes:

Dead code is allowed in this case as there would be some colors we don't use. Otherwise Rust would throw us an error.

```Rust
#[allow(dead_code)]
```

We use a C-like enum here to specify the number for each color. Each enum variant is stored as a `u8`. Only 4 bits are needed though but Rust does not have 4 bits.

```Rust
#[repr(u8)]
```

By deriving the Copy, Clone, Debug, PartialEq, and Eq traits, we enable copy semantics for the type and make it printable and comparable.

```Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

### b. `ColorCode` Structure

Implements the same layout as a `u8` and implements the bit shifting of 4 bits for the background-foreground colors.

```Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]

struct ColorCode(u8);

impl ColorCode {
    fn new (foreground: Color, background: Color) -> ColorCode {
        // first 4 bits for background, last 4 for foreground
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

```

Also, implements the copy semantics.

### c. `ScreenChar` Structure

Structure that encapsulates the 8 bit `ASCII` character and the `ColorCode` structure.

This is what drives what should be displayed on the screen and how.

```Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

```

### d. `Buffer` Structure

This structure holds the characters itself that we can hold.
The `BUFFER_WIDTH` is the number of `screenCharacter` we can hold in one line and the `BUFFER_HEIGHT` is the number of line we can hold. 

In this case: We can have 80 characters per line and we have 25 of such
lines.

```Rust
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
```

The volatile keyword suggests that even if we only do a write and there
no affect on the RAM, there can be other side effects and therefore must
not be optimized out.\
old: `chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],`

```Rust
use volatile::Volatile;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
```

To use `volatile` we add it to the dependencies in `Cargo.toml`

```TOML
# cargo.toml

[dependencies]
volatile = "0.2.6"
```

### e. Writer Structure

The structure which tracks the canvas.

```Rust
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    // Lifetime valid for the whole program run
    buffer: &'static mut Buffer,
}
```

This implements a assortment of functions.

```Rust
// Write a single byte
pub fn write_byte(&mut self, byte: u8);

// Moves the line up and clears the row below
fn new_line(&mut self);

// Helper used for clearing the row
fn clear_row(&mut self, row: usize);

// Calls into write_byte multiple times
pub fn write_string(&mut self, s: &str);
```

The main logic of writing is as follows:

```Rust
let character = ScreenChar {
    ascii_character: byte,
    color_code: color_code,
};

self.buffer.chars[row][col].write(character);
```

Note that we could use `self.buffer.chars[row][col] = character;` instead too but now that we used volatile, we need to use their write method which makes sure that this is not compiled out.

## 3. Global interface for writing

Now a few interesting things that happen here.

We want a mechanism by which we can expose the Writer to other modules. One way to do that is using `statics`.\
The Problem with `static` is that it is a compile time mechanism. But we have things like the pointer to the buffer who's operations are all run-time. This problem can be solved by making them `const` but that is not a solution.

We need a one-time initialization of statics with non-const functions.\
Introducing `lazy_static`. This makes initializations at runtime but right at the beginning, therefore making complex initializations easy.

There's a problem here too. The problem is that this is immutable to and to make it mutable everything else must be under unsafe again, which is again undesirable.

To get interior synchronized mutability we use spinlocks (Not mutexes because we don't have the concept of threads and blocking yet in our kernel).

```Rust
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
```

## 4. Defining our own `print!` and `println!` macro

The macro is taken from the original `println!` macro. The syntax has been made stupidly complicated for no reason.
```Rust
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
```

Within `main` use `println!()` regularly and it should spit it out from the VGA buffer.