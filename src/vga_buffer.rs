use volatile::Volatile;
// volatile allows us to add safety for future rust versions
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

// represent colors as numbers -- 0x0 = Black, 0x1 = Blue, etc.
// C-like enum allows us to specify the number for each color, stored as a u8 thanks to repr(u8)
#[allow(dead_code)]
// Enables copy semnantics for type and allows for printability + comparability
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


// Allows us to represent a full color code specifying foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Ensures identical data layout to u8
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


// represents screen characters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// represents a proper text buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// allows us to write to the screen
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // writes a single byte to the screen
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
                // self.buffer.chars[row][col] = ScreenChar {
                //     ascii_character: byte,
                //     color_code,
                // };
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    // Deletes a single byte from the current cursor position
    pub fn delete_byte(&mut self) {
        // Ensure there are characters to delete
        if self.column_position > 0 {
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position; // Move one position left to delete

            // Shift characters to the left
            for c in col..(BUFFER_WIDTH - 1) {
                let next_char = self.buffer.chars[row][c + 1].read();
                self.buffer.chars[row][c].write(next_char);
            }

            // Clear the last character position
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            self.buffer.chars[row][BUFFER_WIDTH - 1].write(blank);

            // Update column position
            self.column_position -= 1;
        }
    }

    // in the case the byte is \n character or we reach the end of the buffer
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // clear the row we are on to allow for new text
        self.clear_row(BUFFER_HEIGHT - 1);

        // send the buffer back to the beginning
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

    // writes whole strings by taking bytes and printing them one-by-one
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range -- prints default character '■'
                _ => self.write_byte(0xfe),
            }

        }
    }
}

// allows a way for us to use Rust's formatting macros to print different types
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// global interface that can be used without having to carry a Writer instance to each module!
// Lazy static lets us initialize the WRITER when called instead of on compile time, which results in errors
// Mutex allows us to safely have interior mutability to our WRITER, allowing us to change it
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Macros allowing us to use our buffer to print stuff
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! delete {
    ($($arg:tt)*) => ($crate::vga_buffer::_delete());
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// a helper function to print out stuff using our buffer 
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // prevents deadlocks
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _delete() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // prevents deadlocks
    interrupts::without_interrupts(|| {
        WRITER.lock().delete_byte();
    });
}

// prints some random text to the screen
// pub fn print_something() {
//     // let mut writer = Writer {
//     //     column_position: 0,
//     //     color_code: ColorCode::new(Color::Yellow, Color::Black),
//     //     // buffers points to byte 0xb8000 and is cast to mutable raw pointer
//     //     buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     // };

//     // // writes byte and then strings
//     // writer.write_byte(b'H');
//     // writer.write_string("ello ");

//     // // 'ö' is not supported by the VGA buffer, so two default characters are printed instead
//     // writer.write_string("Wörld!");

//     use core::fmt::Write;
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Yellow, Color::Black),
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };

//     writer.write_byte(b'H');
//     writer.write_string("ello!\n");
//     // rust formatting macro!
//     write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
// }



#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
