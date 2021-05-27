use core::fmt::{Result, Write};

use spin::{lazy::Lazy, mutex::spin::SpinMutex};

mod font;
mod framebuffer;

use font::{get_char_pixels, PADDED_FONT_HEIGHT, PADDED_FONT_WIDTH};
use framebuffer::{Color, FrameBuffer};

const BACKGROUND_COLOR: Color = Color {
    r: 32,
    g: 32,
    b: 32,
    a: 255,
};

const FONT_COLOR: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};

pub static CONSOLE: Lazy<SpinMutex<Console>> = Lazy::new(Console::init);

unsafe impl Send for Console {}

pub struct Console {
    cur_row: u32,
    cur_column: u32,
    num_columns: u32,
    num_rows: u32,
    framebuffer: FrameBuffer,
}

impl Console {
    pub fn init() -> SpinMutex<Console> {
        let framebuffer = FrameBuffer::init();
        let num_columns = framebuffer.get_width() / PADDED_FONT_WIDTH as u32;
        let num_rows = framebuffer.get_height() / PADDED_FONT_HEIGHT as u32;

        let mut con = Console {
            framebuffer,
            cur_column: 0,
            cur_row: 0,
            num_columns,
            num_rows,
        };
        con.clear();

        SpinMutex::new(con)
    }

    fn newline(&mut self) {
        self.cur_column = 0;
        self.cur_row += 1;
        if self.cur_row == self.num_rows {
            self.shift_console();
        }
    }

    fn shift_console(&mut self) {
        self.cur_row -= 1;
        for y in 0..PADDED_FONT_HEIGHT as u32 * (self.num_rows - 1) {
            for x in 0..self.framebuffer.get_width() {
                self.framebuffer.set_pixel(
                    x,
                    y,
                    self.framebuffer.get_pixel(x, y + PADDED_FONT_HEIGHT as u32),
                )
            }
        }

        for y in PADDED_FONT_HEIGHT as u32 * (self.num_rows - 1)..self.framebuffer.get_height() {
            for x in 0..self.framebuffer.get_width() {
                self.framebuffer.set_pixel(x, y, BACKGROUND_COLOR)
            }
        }
    }

    fn clear(&mut self) {
        self.cur_row = 0;
        self.cur_column = 0;
        for y in 0..self.framebuffer.get_height() {
            for x in 0..self.framebuffer.get_width() {
                self.framebuffer.set_pixel(x, y, BACKGROUND_COLOR)
            }
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        match c {
            '\n' => {
                self.newline();
            }
            '\r' => {
                self.cur_column = 0;
            }
            '\t' => {
                for _ in 0..4 {
                    if self.cur_column == self.num_rows - 1 {
                        self.write_char('\n')?;
                        break;
                    }
                    self.write_char(' ')?;
                }
            }
            _ => {
                let char_pixels = get_char_pixels(c);
                for x in 0..PADDED_FONT_WIDTH {
                    for y in 0..PADDED_FONT_HEIGHT {
                        if char_pixels[x][y] {
                            self.framebuffer.set_pixel(
                                (x + self.cur_column as usize * PADDED_FONT_WIDTH) as u32,
                                (y + self.cur_row as usize * PADDED_FONT_HEIGHT) as u32,
                                FONT_COLOR,
                            );
                        }
                    }
                }

                self.cur_column += 1;
                if self.cur_column == self.num_columns {
                    self.newline();
                }
            }
        }

        Ok(())
    }
}
