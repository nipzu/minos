const MAILBOX_BASE_PTR: *mut u32 = 0x3F00B880 as *mut u32;
const MAILBOX_STATUS_ADDRS: *mut u32 = unsafe { MAILBOX_BASE_PTR as u32 + 0x18 } as *mut u32;
const MAILBOX_WRITE_ADDR: *mut u32 = unsafe { MAILBOX_BASE_PTR as u32 + 0x20 } as *mut u32;
const MAILBOX_READ_ADDR: *mut u32 = unsafe { MAILBOX_BASE_PTR as u32 + 0x00 } as *mut u32;

const MAILBOX_FULL: u32 = 0x80000000;
const MAILBOX_EMPTY: u32 = 0x40000000;

const REQUEST_CODE: u32 = 0x00000000;
const RESPONSE_CODE: u32 = 0x80000000;
const END_TAG: u32 = 0x00000000;

#[repr(C, align(16))]
pub struct MailboxMessageBuffer<const LEN: usize> {
    data: [u32; LEN],
    end: usize,
}

impl<const LEN: usize> MailboxMessageBuffer<LEN> {
    pub fn new() -> Self {
        let mut data = [0; LEN];

        data[0] = 3;
        data[1] = REQUEST_CODE;
        data[2] = END_TAG;

        Self { data, end: 2 }
    }

    /// Tries to append a tag to the buffer. If it doesn't fit, `false` is returned and the buffer is left untouched.
    #[must_use]
    pub fn try_add_tag<const BUFFER_LEN: usize>(
        &mut self,
        tag: Tag,
        buffer: [u32; BUFFER_LEN],
    ) -> bool {
        if self.end + 3 + BUFFER_LEN > LEN {
            return false;
        }

        self.data[self.end] = tag.get_value();
        self.data[self.end + 1] = BUFFER_LEN as u32;
        self.data[self.end + 2] = REQUEST_CODE;

        self.data[self.end + 3..self.end + 3 + BUFFER_LEN].copy_from_slice(&buffer);

        self.end += 3 + BUFFER_LEN;
        self.data[self.end] = END_TAG;
        self.data[0] = self.end as u32 + 1;

        true
    }

    pub fn send(mut self, channel: u8) -> Option<[u32; LEN]> {
        unsafe {
            while MAILBOX_STATUS_ADDRS.read_volatile() & MAILBOX_FULL > 0 {
                asm!("nop");
            }

            let message = (self.data.as_mut_ptr() as u32 & !0xF) | channel as u32;

            MAILBOX_WRITE_ADDR.write_volatile(message);

            loop {
                while MAILBOX_STATUS_ADDRS.read_volatile() & MAILBOX_EMPTY > 0 {
                    asm!("nop");
                }

                if MAILBOX_READ_ADDR.read_volatile() == message {
                    if self.data[1] != RESPONSE_CODE {
                        return None;
                    }
                    break;
                }
            }
        }
        let mut response = [0; LEN];

        response.copy_from_slice(&self.data[5..]);

        Some(response)
    }
}

#[allow(dead_code)]
pub enum Tag {
    // VideoCore
    GetFirmwareVersion,

    // hardware
    GetBoardModel,
    GetBoardRevision,
    GetBoardMACAddress,
    GetBoardSerial,
    GetARMMemory,
    GetVCMemory,
    GetClocks,

    // config
    GetCommandLine,

    // shared resource management
    GetDMAChannels,

    // power
    GetPowerState,
    GetTiming,
    SetPowerState,

    // clocks
    GetClockState,
    SetClockState,
    GetClockRate,
    GetClockRateMeasured,
    SetClockRate,
    GetMaxClockRate,
    GetMinClockRate,
    GetTurbo,
    SetTurbo,

    // voltage
    GetVoltage,
    SetVoltage,
    GetMaxVoltage,
    GetMinVoltage,
    GetTemperature,
    GetMaxTemperature,
    AllocateMemory,
    LockMemory,
    UnlockMemory,
    ReleaseMemory,
    ExecuteCode,
    GetDispmanxResourceMemHandle,
    GetEDIDBlock,

    // frame buffer
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

impl Tag {
    pub fn get_value(&self) -> u32 {
        use Tag::*;
        match *self {
            // VideoCore
            GetFirmwareVersion => 0x00000001,

            // hardware
            GetBoardModel => 0x00010001,
            GetBoardRevision => 0x00010002,
            GetBoardMACAddress => 0x00010003,
            GetBoardSerial => 0x00010004,
            GetARMMemory => 0x00010005,
            GetVCMemory => 0x00010006,
            GetClocks => 0x00010007,

            // config
            GetCommandLine => 0x00050001,

            // shared resource management
            GetDMAChannels => 0x00060001,

            // power
            GetPowerState => 0x00020001,
            GetTiming => 0x00020002,
            SetPowerState => 0x00028001,

            // clocks
            GetClockState => 0x00030001,
            SetClockState => 0x00038001,
            GetClockRate => 0x00030002,
            GetClockRateMeasured => 0x00030047,
            SetClockRate => 0x00038002,
            GetMaxClockRate => 0x00030004,
            GetMinClockRate => 0x00030007,
            GetTurbo => 0x00030009,
            SetTurbo => 0x00038009,

            // voltage
            GetVoltage => 0x00030003,
            SetVoltage => 0x00038003,
            GetMaxVoltage => 0x00030005,
            GetMinVoltage => 0x00030008,
            GetTemperature => 0x00030006,
            GetMaxTemperature => 0x0003000A,
            AllocateMemory => 0x0003000C,
            LockMemory => 0x0003000D,
            UnlockMemory => 0x0003000E,
            ReleaseMemory => 0x0003000F,
            ExecuteCode => 0x00030010,
            GetDispmanxResourceMemHandle => 0x00030014,
            GetEDIDBlock => 0x00030020,

            // frame buffer
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
}
