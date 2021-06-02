use core::{
    fmt::{Result, Write},
    mem::MaybeUninit,
};

use crate::nolock::NoLock;

mod font;
mod framebuffer;

use font::{get_char_pixels, PADDED_FONT_HEIGHT, PADDED_FONT_WIDTH};
use framebuffer::{Color, Framebuffer};

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

pub static CONSOLE: NoLock<MaybeUninit<Console>> = NoLock::new(MaybeUninit::uninit());

pub struct Console {
    cur_row: u32,
    cur_column: u32,
    num_columns: u32,
    num_rows: u32,
    framebuffer: Framebuffer,
}

impl Console {
    pub fn init() -> Console {
        let framebuffer = Framebuffer::init();
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

        con
    }

    fn newline(&mut self) {
        self.cur_column = 0;
        self.cur_row += 1;
        if self.cur_row == self.num_rows {
            self.cur_row -= 1;
            self.framebuffer
                .shift_up(PADDED_FONT_HEIGHT as u32, BACKGROUND_COLOR);
        }
    }

    fn clear(&mut self) {
        self.cur_row = 0;
        self.cur_column = 0;
        // this clears the screen
        self.framebuffer
            .shift_up(self.framebuffer.get_height(), BACKGROUND_COLOR);
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
                self.write_char(' ')?;
                while self.cur_column % 4 != 0 {
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
