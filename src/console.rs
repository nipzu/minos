use core::fmt::{Error, Result, Write};

use crate::font::{get_char_pixels, PADDED_FONT_WIDTH, PADDED_FONT_HEIGHT};
use crate::framebuffer::{Color, FrameBuffer};

pub struct Console<'fb> {
    cur_row: u32,
    cur_column: u32,
    framebuffer: &'fb mut FrameBuffer,
}

const MAX_NUM_COLUMNS: u32 = 640 / PADDED_FONT_WIDTH as u32;
const MAX_NUM_ROWS: u32 = 480 / PADDED_FONT_HEIGHT as u32;

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
                for x in 0..PADDED_FONT_WIDTH {
                    for y in 0..PADDED_FONT_HEIGHT {
                        let color = char_pixels[x][y];

                        self.framebuffer.set_pixel(
                            (x + self.cur_column as usize * PADDED_FONT_WIDTH) as u32,
                            (y + self.cur_row as usize * PADDED_FONT_HEIGHT) as u32,
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
