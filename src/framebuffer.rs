use crate::mailbox::{MailboxMessageBuffer, Tag};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
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
            ColorFormat::RGBA16 => Color {
                r: (address.offset(0).read_volatile() & 0x0F) << 4,
                g: address.offset(0).read_volatile() & 0xF0,
                b: (address.offset(1).read_volatile() & 0x0F) << 4,
                a: address.offset(1).read_volatile() & 0xF0,
            },
            ColorFormat::BGRA16 => Color {
                b: (address.offset(0).read_volatile() & 0x0F) << 4,
                g: address.offset(0).read_volatile() & 0xF0,
                r: (address.offset(1).read_volatile() & 0x0F) << 4,
                a: address.offset(1).read_volatile() & 0xF0,
            },
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
            ColorFormat::RGBA16 => {
                address.offset(0).write_volatile((r >> 4) | (g & 0xF0));
                address.offset(1).write_volatile((b >> 4) | (a & 0xF0));
            }
            ColorFormat::BGRA16 => {
                address.offset(0).write_volatile((b >> 4) | (g & 0xF0));
                address.offset(1).write_volatile((r >> 4) | (a & 0xF0));
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
            .try_add_tag(Tag::GetVirtualWidthHeight, [0; 2])
            .unwrap();
        message.try_add_tag(Tag::SetVirtualOffset, [0; 2]).unwrap();
        message.try_add_tag(Tag::GetDepth, [0]).unwrap();
        message.try_add_tag(Tag::GetPixelOrder, [0]).unwrap();
        message.try_add_tag(Tag::AllocateBuffer, [0; 2]).unwrap();

        let mut dimensions = None;
        let mut depth = None;
        let mut pixel_order = None;
        let mut buffer_addr = None;
        let mut is_set_virtual_offset = false;

        let response;
        unsafe {
            response = message.send().expect("failed to send framebuffer message");
        }

        for (tag, response_buffer) in response.iter() {
            match tag {
                Tag::GetVirtualWidthHeight => {
                    assert!(dimensions.is_none());
                    assert!(response_buffer.len() >= 2);
                    dimensions = Some((response_buffer[0], response_buffer[1]));
                }
                Tag::SetVirtualOffset => {
                    assert!(!is_set_virtual_offset);
                    is_set_virtual_offset = true;
                }
                Tag::GetDepth => {
                    assert!(depth.is_none());
                    assert!(!response_buffer.is_empty());
                    depth = Some(response_buffer[0]);
                }
                Tag::GetPixelOrder => {
                    assert!(pixel_order.is_none());
                    assert!(!response_buffer.is_empty());
                    pixel_order = Some(response_buffer[0]);
                }
                Tag::AllocateBuffer => {
                    assert!(buffer_addr.is_none());
                    assert!(!response_buffer.is_empty());
                    buffer_addr = Some(response_buffer[0] as *mut u8);
                }
                _ => (),
            }
        }

        assert!(is_set_virtual_offset);

        let format = match (depth, pixel_order) {
            (Some(16), Some(0x1)) => ColorFormat::RGBA16,
            (Some(32), Some(0x1)) => ColorFormat::RGBA32,
            (Some(16), Some(0x0)) => ColorFormat::BGRA16,
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
    RGBA16, // 4 bits for each channel
    RGBA32, // 8 bits for each channel
    BGRA16, // 4 bits for each channel
    BGRA32, // 8 bits for each channel
}

impl ColorFormat {
    fn size(&self) -> isize {
        match *self {
            ColorFormat::RGBA16 | ColorFormat::BGRA16 => 2,
            ColorFormat::RGBA32 | ColorFormat::BGRA32 => 4,
        }
    }
}
