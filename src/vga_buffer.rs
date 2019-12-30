use core::fmt;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/* Structure that encapsulates what needs to be displayed on the screen */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    /*
     * This means that we can BUFFER WIDTH number of screen characters and we
     * can have BUFFER_HEIGHT of them.
     * In this case: We can have 80 characters per line and we have 25 of such
     * lines.
     * The volatile keyword suggests that even if we only do a write and there
     * no affect on the RAM, there can be other side effects and therefore must
     * not be optimized out.
     * old: chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
     */
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    // Lifetime valid for the whole program run
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                let character = ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                };

                /*
                 * Equivalent to self.buffer.chars[row][col] = character;
                 * We are using write because we have defined it as volatile and
                 * we don't want this stuff to get compiled out
                 */
                self.buffer.chars[row][col].write(character);
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // print-able ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part for print-able ASCII range. Write "■" (0xfe)
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
 * A global interface for writing.
 * The problem is that statics need to have all the values like compile-time but
 * we do not, example being the pointer to the buffer is at run time and hence
 * all the functions must be made const. This is bad and not should not be done.
 *
 * We need:- one-time initialization of statics with non-const functions.
 * Sol: lazy_static
 * This makes initializations at runtime but right at the beginning, therefore
 * making complex initializations easy.
 * Here, the problem is though it is immutable to make it mutable everything
 * else must be under unsafe again, which is again undesirable.
 *
 * To get interior synchronized mutability we use spinlocks (Not mutexes
 * because we don't have the concept of threads and blocking yet in our kernel)
 */
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/* Define our own print and println! macros.
 * This is stupidly complicated. I mean I get the point but still.
 */
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
