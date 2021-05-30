use crate::mailbox::{MailboxMessageBuffer, MailboxTagType};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
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

pub struct Framebuffer {
    buffer_addr: *mut u8,
    width: isize,
    height: isize,
    format: ColorFormat,
}

impl Framebuffer {
    pub fn init() -> Framebuffer {
        let mut message = MailboxMessageBuffer::<32, FramebufferTag>::new();

        message
            .try_add_tag(FramebufferTag::SetVirtualWidthHeight, [640, 480])
            .unwrap();
        message
            .try_add_tag(FramebufferTag::SetPhysicalWidthHeight, [640, 480])
            .unwrap();
        message
            .try_add_tag(FramebufferTag::SetVirtualOffset, [0; 2])
            .unwrap();
        message.try_add_tag(FramebufferTag::SetDepth, [32]).unwrap();
        message
            .try_add_tag(FramebufferTag::SetPixelOrder, [1])
            .unwrap();
        message
            .try_add_tag(FramebufferTag::AllocateBuffer, [4096, 0])
            .unwrap();

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
                FramebufferTag::SetVirtualWidthHeight => {
                    //assert!(dimensions.is_none());
                    //assert!(response_buffer.len() >= 2);
                    dimensions = Some((response_buffer[0], response_buffer[1]));
                }
                FramebufferTag::SetVirtualOffset => {
                    assert!(!is_set_virtual_offset);
                    is_set_virtual_offset = true;
                }
                FramebufferTag::SetDepth => {
                    assert!(depth.is_none());
                    assert!(!response_buffer.is_empty());
                    depth = Some(response_buffer[0]);
                }
                FramebufferTag::SetPixelOrder => {
                    assert!(pixel_order.is_none());
                    assert!(!response_buffer.is_empty());
                    pixel_order = Some(response_buffer[0]);
                }
                FramebufferTag::AllocateBuffer => {
                    assert!(buffer_addr.is_none());
                    assert!(!response_buffer.is_empty());
                    // TODO: explain this black magic bitmask
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

        Framebuffer {
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

    pub fn shift_up(&mut self, amount: u32, fill_color: Color) {
        let amount = amount as isize;
        let dst = self.buffer_addr;
        let src = unsafe {
            self.buffer_addr
                .offset(amount * self.width * self.format.size())
        };

        /*unsafe {
            volatile_copy_memory(
                dst as *mut u128,
                src as *const u128,
                (self.height.saturating_sub(amount) * self.width / 4) as usize,
            );
        }*/

        unsafe {
            crate::memory::volatile_copy_forwards_aligned(
                src as *mut u128,
                dst as *mut u128,
                (self.height.saturating_sub(amount) * self.width * self.format.size()) as usize,
            );
        }

        match self.format.size() {
            2 => {
                if self.get_width() % 8 == 0 {
                    unsafe {
                        let mut fill_data = 0u128;
                        let fill_data_ptr = (&mut fill_data as *mut u128).cast::<u8>();
                        for i in 0..8 {
                            fill_color.write_address(fill_data_ptr.offset(2 * i), self.format);
                        }
                        self.shift_up_128_bit_aligned(amount as isize, self.width / 8, fill_data)
                    }
                } else {
                    unsafe { self.shift_up_16_bit(amount as isize, fill_color) }
                }
            }
            4 => {
                if self.get_width() % 4 == 0 {
                    unsafe {
                        let mut fill_data = 0u128;
                        let fill_data_ptr = (&mut fill_data as *mut u128).cast::<u8>();
                        for i in 0..4 {
                            fill_color.write_address(fill_data_ptr.offset(4 * i), self.format);
                        }
                        self.shift_up_128_bit_aligned(amount as isize, self.width / 4, fill_data)
                    }
                } else {
                    unsafe { self.shift_up_32_bit(amount as isize, fill_color) }
                }
            }
            _ => unreachable!(),
        }
    }

    unsafe fn shift_up_16_bit(&mut self, amount: isize, fill_color: Color) {
        let mut fill_color_value = 0u16;
        fill_color.write_address(
            (&mut fill_color_value as *mut u16).cast::<u8>(),
            self.format,
        );

        /*for y in 0..self.height.saturating_sub(amount) {
            for x in 0..self.width {
                (self.buffer_addr as *mut u16)
                    .offset(y * self.width + x)
                    .write_volatile(
                        (self.buffer_addr as *mut u16)
                            .offset((y + amount) * self.width + x)
                            .read_volatile(),
                    )
            }
        }*/

        for y in self.height.saturating_sub(amount)..self.height {
            for x in 0..self.width {
                (self.buffer_addr as *mut u16)
                    .offset(y * self.width + x)
                    .write_volatile(fill_color_value);
            }
        }
    }

    unsafe fn shift_up_32_bit(&mut self, amount: isize, fill_color: Color) {
        let mut fill_color_value = 0u32;
        fill_color.write_address(
            (&mut fill_color_value as *mut u32).cast::<u8>(),
            self.format,
        );

        /*for y in 0..self.height.saturating_sub(amount) {
            for x in 0..self.width {
                (self.buffer_addr as *mut u32)
                    .offset(y * self.width + x)
                    .write_volatile(
                        (self.buffer_addr as *mut u32)
                            .offset((y + amount) * self.width + x)
                            .read_volatile(),
                    )
            }
        }*/

        for y in self.height.saturating_sub(amount)..self.height {
            for x in 0..self.width {
                (self.buffer_addr as *mut u32)
                    .offset(y * self.width + x)
                    .write_volatile(fill_color_value);
            }
        }
    }

    unsafe fn shift_up_128_bit_aligned(&mut self, amount: isize, width_u128s: isize, fill: u128) {
        /*for y in 0..self.height.saturating_sub(amount) {
            for x in 0..width_u128s {
                (self.buffer_addr as *mut u128)
                    .offset(y * width_u128s + x)
                    .write_volatile(
                        (self.buffer_addr as *mut u128)
                            .offset((y + amount) * width_u128s + x)
                            .read_volatile(),
                    )
            }
        }*/

        for y in self.height.saturating_sub(amount)..self.height {
            for x in 0..width_u128s {
                (self.buffer_addr as *mut u128)
                    .offset(y * width_u128s + x)
                    .write_volatile(fill);
            }
        }
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

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum FramebufferTag {
    AllocateBuffer,
    ReleaseBuffer,
    BlackScreen,
    GetPhysicalWidthHeight,
    TestPhysicalWidthHeight,
    SetPhysicalWidthHeight,
    GetVirtualWidthHeight,
    TestVirtualWidthHeight,
    SetVirtualWidthHeight,
    GetDepth,
    TestDepth,
    SetDepth,
    GetPixelOrder,
    TestPixelOrder,
    SetPixelOrder,
    GetAlphaMode,
    TestAlphaMode,
    SetAlphaMode,
    GetPitch,
    GetVirtualOffset,
    TestVirtualOffset,
    SetVirtualOffset,
    GetOverscan,
    TestOverscan,
    SetOverscan,
    GetPalette,
    TestPalette,
    SetPalette,
    SetCursorInfo,
    SetCursorState,
}

impl MailboxTagType for FramebufferTag {
    fn get_value(&self) -> u32 {
        use FramebufferTag::*;
        match *self {
            AllocateBuffer => 0x00040001,
            ReleaseBuffer => 0x00048001,
            BlackScreen => 0x00040002,
            GetPhysicalWidthHeight => 0x00040003,
            TestPhysicalWidthHeight => 0x00044003,
            SetPhysicalWidthHeight => 0x00048003,
            GetVirtualWidthHeight => 0x00040004,
            TestVirtualWidthHeight => 0x00044004,
            SetVirtualWidthHeight => 0x00048004,
            GetDepth => 0x00040005,
            TestDepth => 0x00044005,
            SetDepth => 0x00048005,
            GetPixelOrder => 0x00040006,
            TestPixelOrder => 0x00044006,
            SetPixelOrder => 0x00048006,
            GetAlphaMode => 0x00040007,
            TestAlphaMode => 0x00044007,
            SetAlphaMode => 0x00048007,
            GetPitch => 0x00040008,
            GetVirtualOffset => 0x00040009,
            TestVirtualOffset => 0x00044009,
            SetVirtualOffset => 0x00048009,
            GetOverscan => 0x0004000A,
            TestOverscan => 0x0004400A,
            SetOverscan => 0x0004800A,
            GetPalette => 0x0004000B,
            TestPalette => 0x0004400B,
            SetPalette => 0x0004800B,
            SetCursorInfo => 0x00008010,
            SetCursorState => 0x00008011,
        }
    }

    fn from_value(value: u32) -> Option<FramebufferTag> {
        use FramebufferTag::*;
        Some(match value {
            0x00040001 => AllocateBuffer,
            0x00048001 => ReleaseBuffer,
            0x00040002 => BlackScreen,
            0x00040003 => GetPhysicalWidthHeight,
            0x00044003 => TestPhysicalWidthHeight,
            0x00048003 => SetPhysicalWidthHeight,
            0x00040004 => GetVirtualWidthHeight,
            0x00044004 => TestVirtualWidthHeight,
            0x00048004 => SetVirtualWidthHeight,
            0x00040005 => GetDepth,
            0x00044005 => TestDepth,
            0x00048005 => SetDepth,
            0x00040006 => GetPixelOrder,
            0x00044006 => TestPixelOrder,
            0x00048006 => SetPixelOrder,
            0x00040007 => GetAlphaMode,
            0x00044007 => TestAlphaMode,
            0x00048007 => SetAlphaMode,
            0x00040008 => GetPitch,
            0x00040009 => GetVirtualOffset,
            0x00044009 => TestVirtualOffset,
            0x00048009 => SetVirtualOffset,
            0x0004000A => GetOverscan,
            0x0004400A => TestOverscan,
            0x0004800A => SetOverscan,
            0x0004000B => GetPalette,
            0x0004400B => TestPalette,
            0x0004800B => SetPalette,
            0x00008010 => SetCursorInfo,
            0x00008011 => SetCursorState,

            _ => return None,
        })
    }
}
