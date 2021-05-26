use crate::mailbox::{MailboxMessageBuffer, Tag};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const RED: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    unsafe fn read_address(address: *const u8, format: ColorFormat) -> Color {
        match format {
            ColorFormat::RGBA32 => Color {
                r: address.offset(0).read_volatile(),
                g: address.offset(1).read_volatile(),
                b: address.offset(2).read_volatile(),
                a: address.offset(3).read_volatile(),
            },
            ColorFormat::BGRA32 => Color {
                b: address.offset(0).read_volatile(),
                g: address.offset(1).read_volatile(),
                r: address.offset(2).read_volatile(),
                a: address.offset(3).read_volatile(),
            },
            ColorFormat::RGB16 => {
                let b0 = address.offset(0).read_volatile();
                let b1 = address.offset(1).read_volatile();
                Color {
                    r: b1 & 0b_1111_1000,
                    g: (b1 << 5) | ((b0 & 0b_1110_0000) >> 3),
                    b: b0 << 3,
                    a: 255,
                }
            }
            ColorFormat::BGR16 => {
                let b0 = address.offset(0).read_volatile();
                let b1 = address.offset(1).read_volatile();
                Color {
                    b: b1 & 0b_1111_1000,
                    g: (b1 << 5) | ((b0 & 0b_1110_0000) >> 3),
                    r: b0 << 3,
                    a: 255,
                }
            }
        }
    }

    unsafe fn write_address(self, address: *mut u8, format: ColorFormat) {
        let Color { r, g, b, a } = self;
        match format {
            ColorFormat::RGBA32 => {
                address.offset(0).write_volatile(r);
                address.offset(1).write_volatile(g);
                address.offset(2).write_volatile(b);
                address.offset(3).write_volatile(a);
            }
            ColorFormat::BGRA32 => {
                address.offset(0).write_volatile(b);
                address.offset(1).write_volatile(g);
                address.offset(2).write_volatile(r);
                address.offset(3).write_volatile(a);
            }
            ColorFormat::RGB16 => {
                // red bits
                // offset: 0, 0b_0000_0000;
                // offset: 1, 0b_1111_1000;
                // green bits
                // offset: 0, 0b_1110_0000;
                // offset: 1, 0b_0000_0111;
                // blue bits
                // offset: 0, 0b_0001_1111;
                // offset: 1, 0b_0000_0000;

                address
                    .offset(0)
                    .write_volatile(((g << 3) & 0b_1110_0000) | (b >> 3));
                address
                    .offset(1)
                    .write_volatile((g >> 5) | (r & 0b_1111_1000));
            }
            ColorFormat::BGR16 => {
                // Same as RGB16 but with R and B swapped
                address
                    .offset(0)
                    .write_volatile(((g << 3) & 0b_1110_0000) | (r >> 3));
                address
                    .offset(1)
                    .write_volatile((g >> 5) | (b & 0b_1111_1000));
            }
        }
    }
}

pub struct FrameBuffer {
    buffer_addr: *mut u8,
    width: isize,
    height: isize,
    format: ColorFormat,
}

impl FrameBuffer {
    pub fn init() -> FrameBuffer {
        let mut message = MailboxMessageBuffer::<32>::new();

        message
            .try_add_tag(Tag::SetVirtualWidthHeight, [640, 480])
            .unwrap();
        message
            .try_add_tag(Tag::SetPhysicalWidthHeight, [640, 480])
            .unwrap();
        message.try_add_tag(Tag::SetVirtualOffset, [0; 2]).unwrap();
        message.try_add_tag(Tag::SetDepth, [32]).unwrap();
        message.try_add_tag(Tag::SetPixelOrder, [1]).unwrap();
        message.try_add_tag(Tag::AllocateBuffer, [4096, 0]).unwrap();

        let mut buffer_addr = None;
        let mut dimensions = None;
        let mut depth = None;
        let mut pixel_order = None;
        let mut is_set_virtual_offset = false;

        let response;
        unsafe {
            response = message.send().expect("failed to send framebuffer message");
        }

        for (tag, response_buffer) in response.iter() {
            match tag {
                Tag::SetVirtualWidthHeight => {
                    //assert!(dimensions.is_none());
                    //assert!(response_buffer.len() >= 2);
                    dimensions = Some((response_buffer[0], response_buffer[1]));
                }
                Tag::SetVirtualOffset => {
                    assert!(!is_set_virtual_offset);
                    is_set_virtual_offset = true;
                }
                Tag::SetDepth => {
                    assert!(depth.is_none());
                    assert!(!response_buffer.is_empty());
                    depth = Some(response_buffer[0]);
                }
                Tag::SetPixelOrder => {
                    assert!(pixel_order.is_none());
                    assert!(!response_buffer.is_empty());
                    pixel_order = Some(response_buffer[0]);
                }
                Tag::AllocateBuffer => {
                    assert!(buffer_addr.is_none());
                    assert!(!response_buffer.is_empty());
                    // TODO: what is this black magic bitmask?
                    buffer_addr = Some((response_buffer[0] & 0x3FFFFFFF) as *mut u8);
                }
                _ => (),
            }
        }

        assert!(is_set_virtual_offset);

        let format = match (depth, pixel_order) {
            (Some(16), Some(0x1)) => ColorFormat::RGB16,
            (Some(16), Some(0x0)) => ColorFormat::BGR16,
            (Some(32), Some(0x1)) => ColorFormat::RGBA32,
            (Some(32), Some(0x0)) => ColorFormat::BGRA32,
            _ => panic!("received incorrect framebuffer pixel format"),
        };

        FrameBuffer {
            buffer_addr: buffer_addr.expect("did not receive framebuffer address"),
            width: dimensions
                .expect("did not reveive framebuffer dimensions")
                .0 as isize,
            height: dimensions
                .expect("did not reveive framebuffer dimensions")
                .1 as isize,
            format,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        let x = x as isize;
        let y = y as isize;
        assert!(x < self.width);
        assert!(y < self.height);

        let pixel_size = self.format.size();
        let offset = (y * self.width + x) * pixel_size;

        unsafe {
            let address = self.buffer_addr.offset(offset);
            Color::read_address(address, self.format)
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let x = x as isize;
        let y = y as isize;
        assert!(x < self.width);
        assert!(y < self.height);

        let pixel_size = self.format.size();
        let offset = (y * self.width + x) * pixel_size;

        unsafe {
            let address = self.buffer_addr.offset(offset);
            color.write_address(address, self.format)
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_height(&self) -> u32 {
        self.height as u32
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum ColorFormat {
    RGB16,
    RGBA32, // 8 bits for each channel
    BGR16,
    BGRA32, // 8 bits for each channel
}

impl ColorFormat {
    fn size(&self) -> isize {
        match *self {
            ColorFormat::RGB16 | ColorFormat::BGR16 => 2,
            ColorFormat::RGBA32 | ColorFormat::BGRA32 => 4,
        }
    }
}
