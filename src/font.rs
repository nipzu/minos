static FONT_DATA: [u8; FONT_WIDTH * FONT_HEIGHT * 96 / 8] = include!("../font_data.txt");

pub const FONT_WIDTH: usize = 5;
pub const FONT_HEIGHT: usize = 8;
const CHARS_PER_ROW: usize = 16;

const FONT_PADDING_LEFT: usize = 1;
const FONT_PADDING_RIGHT: usize = 1;
const FONT_PADDING_TOP: usize = 1;
const FONT_PADDING_BOTTOM: usize = 1;

pub const PADDED_FONT_WIDTH: usize = FONT_WIDTH + FONT_PADDING_LEFT + FONT_PADDING_RIGHT;
pub const PADDED_FONT_HEIGHT: usize = FONT_HEIGHT + FONT_PADDING_TOP + FONT_PADDING_BOTTOM;

/// Returns the set pixels of the char in array[x][y] format.
/// Any character outside of the range ' '..='~' will be mapped
/// to a white square representing an unknown symbol.
pub fn get_char_pixels(c: char) -> [[bool; PADDED_FONT_HEIGHT]; PADDED_FONT_WIDTH] {
    let char_index = if (' '..='~').contains(&c) {
        c as usize - ' ' as usize
    } else {
        95 // index for del which is mapped to a white square
    };

    let start_index = FONT_WIDTH * (char_index % CHARS_PER_ROW)
        + CHARS_PER_ROW * FONT_WIDTH * FONT_HEIGHT * (char_index / CHARS_PER_ROW);

    let mut pixels = [[false; PADDED_FONT_HEIGHT]; PADDED_FONT_WIDTH];

    for y in 0..FONT_HEIGHT {
        for x in 0..FONT_WIDTH {
            let index = start_index + x + FONT_WIDTH * CHARS_PER_ROW * y;
            pixels[x + FONT_PADDING_LEFT][y + FONT_PADDING_TOP] =
                (FONT_DATA[index / 8] >> (index % 8)) & 0x1 == 1
        }
    }

    pixels
}
