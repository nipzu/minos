use core::arch::asm;
use core::marker::PhantomData;

const MAILBOX_BASE_ADDR: usize = 0x3F00B880;
// TODO: could we use offset here?
const MAILBOX_STATUS_PTR: *const u32 = (MAILBOX_BASE_ADDR + 0x18) as _;
const MAILBOX_WRITE_PTR: *mut u32 = (MAILBOX_BASE_ADDR + 0x20) as _;
const MAILBOX_READ_PTR: *const u32 = MAILBOX_BASE_ADDR as _;

const MAILBOX_FULL: u32 = 0x80000000;
const MAILBOX_EMPTY: u32 = 0x40000000;

const REQUEST_CODE: u32 = 0x00000000;
const RESPONSE_CODE: u32 = 0x80000000;
const END_TAG: u32 = 0x00000000;

const MAILBOX_PROPERTY_CHANNEL: u8 = 8;

/// A type that represents a properly aligned Mailbox buffer.
/// `LEN` is the maximum number of `u32`s that can fit in the buffer.
#[repr(C, align(16))]
pub struct MailboxMessageBuffer<const LEN: usize, T: MailboxTagType> {
    data: [u32; LEN],
    end: usize,
    _t: PhantomData<T>,
}

impl<const LEN: usize, T: MailboxTagType> MailboxMessageBuffer<LEN, T> {
    /// Creates a new `MailboxMessageBuffer` that can contain at most `4*LEN` bytes of data
    ///
    /// # Panics
    /// This code will panic if `LEN < 3`
    pub fn new() -> Self {
        assert!(LEN >= 3);
        let mut data = [0; LEN];

        unsafe {
            data.as_mut_ptr().offset(0).write_volatile(12);
            data.as_mut_ptr().offset(1).write_volatile(REQUEST_CODE);
            data.as_mut_ptr().offset(2).write_volatile(END_TAG);
        }

        Self {
            data,
            end: 2,
            _t: PhantomData,
        }
    }

    /// Tries to append a tag to the buffer. If it doesn't fit,
    /// the number of available `u32`s in the buffer is returned.
    pub fn try_add_tag<const BUFFER_LEN: usize>(
        &mut self,
        tag: T,
        buffer: [u32; BUFFER_LEN],
    ) -> Result<(), usize> {
        if self.end + 3 + BUFFER_LEN >= LEN {
            assert!(LEN >= self.end);
            return Err(LEN - self.end);
        }

        unsafe {
            let end_ptr = self.data.as_mut_ptr().add(self.end);

            // set tag, buffer len, and request code
            end_ptr.add(0).write_volatile(tag.get_value());
            end_ptr.add(1).write_volatile(4 * BUFFER_LEN as u32);
            end_ptr.add(2).write_volatile(REQUEST_CODE);

            for i in 0..BUFFER_LEN {
                // copy buffer contents
                end_ptr.add(3 + i).write_volatile(buffer[i]);
            }

            // set new end pointer
            self.end += 3 + BUFFER_LEN;
            // write end tag
            self.data.as_mut_ptr().add(self.end).write_volatile(END_TAG);
            // write len to first u32
            self.data
                .as_mut_ptr()
                .add(0)
                .write_volatile(4 * (self.end as u32 + 1));
        }

        Ok(())
    }

    /// Sends the buffer through the property channel returning a copy of the response.
    ///
    /// # Safety
    /// Caller must make sure that the contents of the buffer are safe.
    pub unsafe fn send(&self) -> Result<MailboxResponse<LEN, T>, ()> {
        while MAILBOX_STATUS_PTR.read_volatile() & MAILBOX_FULL > 0 {
            asm!("nop");
        }

        // TODO: should we pin or something, are there any guarantees that self.data won't be moved
        let message = (self.data.as_ptr() as u32 & !0xf) | MAILBOX_PROPERTY_CHANNEL as u32;

        MAILBOX_WRITE_PTR.write_volatile(message);

        loop {
            while MAILBOX_STATUS_PTR.read_volatile() & MAILBOX_EMPTY > 0 {
                asm!("nop");
            }

            if MAILBOX_READ_PTR.read_volatile() == message {
                if self.data.as_ptr().add(1).read_volatile() != RESPONSE_CODE {
                    return Err(());
                }
                break;
            }
        }

        let mut response_data = [0; LEN];

        for i in 0..LEN {
            response_data[i] = self.data.as_ptr().add(i).read_volatile();
        }

        Ok(MailboxResponse {
            data: response_data,
            end: self.end,
            _t: PhantomData,
        })
    }
}

/// A struct representing the response to a Mailbox message.
/// `LEN` is the capacity of the buffer in `u32`s.
pub struct MailboxResponse<const LEN: usize, T: MailboxTagType> {
    data: [u32; LEN],
    end: usize,
    _t: PhantomData<T>,
}

impl<const LEN: usize, T: MailboxTagType> MailboxResponse<LEN, T> {
    /// Returns an iterator that iterates through the tags in the response.
    pub fn iter(&self) -> MailboxResponseIterator<T> {
        MailboxResponseIterator {
            response_data: &self.data,
            cur: 2, // start at first tag
            end: self.end,
            _t: PhantomData,
        }
    }
}

pub struct MailboxResponseIterator<'data, T: MailboxTagType> {
    response_data: &'data [u32],
    cur: usize,
    end: usize,
    _t: PhantomData<T>,
}

impl<'data, T: MailboxTagType> Iterator for MailboxResponseIterator<'data, T> {
    type Item = (T, &'data [u32]);
    fn next(&mut self) -> Option<Self::Item> {
        let mut tag = None;
        let mut value_buffer: &[u32] = &[];
        while tag.is_none() {
            if self.cur >= self.end {
                return None;
            }
            assert!(self.cur < self.end);
            let buffer_len = self.response_data[self.cur + 1] as usize / 4;
            assert!(self.cur + 3 + buffer_len <= self.end);
            assert!(self.response_data[self.cur + 2] >> 31 == 1);

            tag = T::from_value(self.response_data[self.cur]);
            let response_len = ((self.response_data[self.cur + 2] as usize & 0x7fff_ffff) + 3) / 4;
            value_buffer = &self.response_data[self.cur + 3..self.cur + 3 + response_len];
            self.cur += 3 + buffer_len;
        }

        Some((tag.unwrap(), value_buffer))
    }
}

pub trait MailboxTagType: Sized {
    fn get_value(&self) -> u32;
    fn from_value(value: u32) -> Option<Self>;
}

/// An enum representing one of the possible tags in a Mailbox message.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum MiscTag {
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
}

impl MailboxTagType for MiscTag {
    fn get_value(&self) -> u32 {
        use MiscTag::*;
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
        }
    }

    fn from_value(value: u32) -> Option<MiscTag> {
        use MiscTag::*;
        Some(match value {
            // VideoCore
            0x00000001 => GetFirmwareVersion,

            // hardware
            0x00010001 => GetBoardModel,
            0x00010002 => GetBoardRevision,
            0x00010003 => GetBoardMACAddress,
            0x00010004 => GetBoardSerial,
            0x00010005 => GetARMMemory,
            0x00010006 => GetVCMemory,
            0x00010007 => GetClocks,

            // config
            0x00050001 => GetCommandLine,

            // shared resource management
            0x00060001 => GetDMAChannels,

            // power
            0x00020001 => GetPowerState,
            0x00020002 => GetTiming,
            0x00028001 => SetPowerState,

            // clocks
            0x00030001 => GetClockState,
            0x00038001 => SetClockState,
            0x00030002 => GetClockRate,
            0x00030047 => GetClockRateMeasured,
            0x00038002 => SetClockRate,
            0x00030004 => GetMaxClockRate,
            0x00030007 => GetMinClockRate,
            0x00030009 => GetTurbo,
            0x00038009 => SetTurbo,

            // voltage
            0x00030003 => GetVoltage,
            0x00038003 => SetVoltage,
            0x00030005 => GetMaxVoltage,
            0x00030008 => GetMinVoltage,
            0x00030006 => GetTemperature,
            0x0003000A => GetMaxTemperature,
            0x0003000C => AllocateMemory,
            0x0003000D => LockMemory,
            0x0003000E => UnlockMemory,
            0x0003000F => ReleaseMemory,
            0x00030010 => ExecuteCode,
            0x00030014 => GetDispmanxResourceMemHandle,
            0x00030020 => GetEDIDBlock,

            _ => return None,
        })
    }
}
