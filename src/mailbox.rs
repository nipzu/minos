const MAILBOX_BASE_PTR: *mut u32 = 0x3F00B880 as *mut u32;
const MAILBOX_STATUS_ADDRS: *mut u32 = unsafe {MAILBOX_BASE_PTR as u32 + 0x18} as *mut u32;
const MAILBOX_WRITE_ADDR: *mut u32 = unsafe {MAILBOX_BASE_PTR as u32 + 0x20} as *mut u32;
const MAILBOX_READ_ADDR: *mut u32 = unsafe {MAILBOX_BASE_PTR as u32 + 0x00} as *mut u32;

const MAILBOX_FULL: u32 = 0x80000000;
const MAILBOX_EMPTY: u32 = 0x40000000;

const REQUEST_CODE: u32 = 0x00000000;
const RESPONSE_CODE: u32 = 0x80000000;
const END_TAG: u32 = 0x00000000;

#[repr(C, align(16))]
pub struct MailboxMessage<const LEN: usize>
where
    [(); LEN + 6]: ,
{
    data: [u32; LEN + 6],
}

fn copy_from_slice<T: Copy>(dest: &mut [T], source: &[T]) {
    for i in 0..dest.len() {
        if i >= source.len() {
            return;
        }
        dest[i] = source[i];
    }
}

impl<const LEN: usize> MailboxMessage<LEN>
where
    [(); LEN + 6]: ,
{
    pub fn new(tag: Tag, request_value: [u32; LEN]) -> Self {
        let mut data = [0; LEN + 6];

        data[0] = LEN as u32;
        data[1] = REQUEST_CODE;

        data[2] = tag.get_value();
        data[3] = LEN as u32 * 4;
        data[4] = REQUEST_CODE;

        copy_from_slice(&mut data[5..LEN + 5], &request_value);

        data[LEN + 5] = END_TAG;

        Self { data }
    }

    pub fn send(mut self, channel: u8) -> Option<[u32; LEN]> {
        unsafe {
            while MAILBOX_STATUS_ADDRS.read_volatile() & MAILBOX_FULL > 0 { 
                asm!("nop"); 
            }
            
            let message = 
            (self.data.as_mut_ptr() as u32 & !0xF) | channel as u32;

            MAILBOX_WRITE_ADDR.write_volatile(
                message
            );

            loop {
                while MAILBOX_STATUS_ADDRS.read_volatile() & MAILBOX_EMPTY > 0 {
                    asm!("nop"); 
                }

                if MAILBOX_READ_ADDR.read_volatile() == message {
                    if self.data[1] != RESPONSE_CODE {
                        return None;
                    }
                    break
                }
            }
        }
        let mut response = [0; LEN];

        copy_from_slice(&mut response, &self.data[5..]);

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
