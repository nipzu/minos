use crate::framebuffer::Color;

static FONT_DATA: [u8; 576] = include!("../font_data.txt");

/// Returns the set pixels of the char in array[x][y] format.
/// Any character outside of the range ' '..='~' will be mapped
/// to a white square representing an unknown symbol.
pub fn get_char_pixels(c: char) -> [[Color; 8]; 6] {
    let char_index = if (' '..='~').contains(&c) {
        c as usize - ' ' as usize
    } else {
        95 // index for del which is mapped to a white square
    };

    const PIXELS_PER_CHAR_ROW: usize =
        6 // width of char
        *
        8 // height of char
        *
        16 // chars per row
    ;

    let start_index = 6 * (char_index % 16) + PIXELS_PER_CHAR_ROW * (char_index / 16);

    let mut pixels = [[Color::BLACK; 8]; 6];

    for y in 0..8 {
        for x in 0..6 {
            let index = start_index + x + 6 * 16 * y;
            pixels[x][y] = if (FONT_DATA[index / 8] >> (index % 8)) & 0x1 == 1 {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
    }

    pixels
}
