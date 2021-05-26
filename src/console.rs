use core::fmt::{Error, Result, Write};

use crate::font::get_char_pixels;
use crate::framebuffer::{Color, FrameBuffer};

pub struct Console<'fb> {
    cur_row: u32,
    cur_column: u32,
    framebuffer: &'fb mut FrameBuffer,
}

const MAX_NUM_COLUMNS: u32 = 100;
const MAX_NUM_ROWS: u32 = 30;

impl Console<'_> {
    pub fn new(framebuffer: &mut FrameBuffer) -> Console {
        for y in 0..framebuffer.get_height() {
            for x in 0..framebuffer.get_width() {
                framebuffer.set_pixel(x, y, Color::BLACK)
            }
        }

        Console {
            framebuffer,
            cur_column: 0,
            cur_row: 0,
        }
    }
}

impl Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        match c {
            '\n' => {
                self.cur_column = 0;
                self.cur_row += 1;
                if self.cur_row >= MAX_NUM_ROWS {
                    return Err(Error);
                }
            }
            _ => {
                let char_pixels = get_char_pixels(c);
                for x in 0..6 {
                    for y in 0..8 {
                        let color = char_pixels[x as usize][y as usize];

                        self.framebuffer.set_pixel(
                            (x + self.cur_column * 6) as u32,
                            (y + self.cur_row * 8) as u32,
                            color,
                        );
                        /*self.framebuffer.set_pixel(
                            2 * (x + self.cur_column * 6) as u32,
                            2 * (y + self.cur_row * 8) as u32,
                            color,
                        );
                        self.framebuffer.set_pixel(
                            2 * (x + self.cur_column * 6) as u32 + 1,
                            2 * (y + self.cur_row * 8) as u32,
                            color,
                        );
                        self.framebuffer.set_pixel(
                            2 * (x + self.cur_column * 6) as u32,
                            2 * (y + self.cur_row * 8) as u32 + 1,
                            color,
                        );
                        self.framebuffer.set_pixel(
                            2 * (x + self.cur_column * 6) as u32 + 1,
                            2 * (y + self.cur_row * 8) as u32 + 1,
                            color,
                        );*/
                    }
                }

                self.cur_column += 1;
                if self.cur_column == MAX_NUM_COLUMNS {
                    self.cur_column = 0;
                    self.cur_row += 1;
                    if self.cur_row >= MAX_NUM_ROWS {
                        return Err(Error);
                    }
                }
            }
        }

        Ok(())
    }
}
