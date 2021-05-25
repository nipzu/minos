use crate::mailbox::{MailboxMessageBuffer, Tag};

pub struct FrameBuffer {
    buffer_addr: *mut u8,
    width: usize,
    height: usize,
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

        let dimensions = None;
        let depth = None;
        let pixel_order = None;
        let buffer_addr = None;
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
                Tag::SetVirtualOffset => (),
                Tag::GetDepth => (),
                Tag::GetPixelOrder => (),
                Tag::AllocateBuffer => {
                    assert!(buffer_addr.is_none());
                    assert!(!response_buffer.is_empty());
                    buffer_addr = Some(response_buffer[0] as *mut u8);
                }
                _ => (),
            }
        }

        todo!()
    }
}

enum ColorFormat {
    RGBA16, // 4 bits for each channel
    RGBA32, // 8 bits for each channel
    BGRA16, // 4 bits for each channel
    BGRA32, // 8 bits for each channel
}
