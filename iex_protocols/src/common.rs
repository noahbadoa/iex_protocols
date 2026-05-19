use core::fmt::Debug;
use core::time::Duration;
use super::endian::Little;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
// https://doc.rust-lang.org/std/ascii/enum.Char.html unstable so just use u8
/// Space terminated ascii string
pub struct IexString<const MAX : usize>([u8; MAX]);

impl<const MAX : usize> core::fmt::Debug for IexString<MAX>{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let string = self.as_str();
        string.fmt(f)
    }
}


impl<const MAX : usize> IexString<MAX>{
    pub const fn as_bytes(&self) -> [u8; MAX]{self.0}
    pub const fn from_bytes(bytes : &[u8]) -> Option<Self>{
        if bytes.len() > MAX {return None;}

        let mut out = [b' '; MAX];
        let mut counter = 0;
        while counter < bytes.len() {
            let byte = bytes[counter];
            if byte > 127 {return None;}

            out[counter] = byte;
            counter += 1;
        }

        Some(Self(out))
    }

    pub const fn is_vaild(bytes : &[u8; MAX]) -> bool{
        let mut counter = 0;
        while counter < MAX {
            if bytes[counter] > 127 {return false;}
            counter += 1;
        }

        true
    }   

    pub const fn as_str(&self) -> &str {
        let mut counter = 0;
        while counter < MAX {
            if self.as_bytes()[counter] == b' ' {break;}

            counter += 1;
        }

        let slice = unsafe{core::slice::from_raw_parts(self.0.as_ptr(), counter)};
        unsafe{str::from_utf8_unchecked(slice)}
    }
    pub const fn from_str(symbol : &str) -> Option<Self>{Self::from_bytes(symbol.as_bytes())}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Security ticker symbol string
pub struct Symbol(pub IexString<8>);


impl Symbol{
    pub const fn as_str(&self) -> &str {self.0.as_str()}
    pub const fn from_str(symbol : &str) -> Option<Self>{
        // essentially just a transmute
        match IexString::from_str(symbol){ 
            None => None,
            Some(symbol) => Some(Self(symbol))
        }
    }
    pub const fn is_vaild(bytes : &[u8; 8]) -> bool{
        IexString::is_vaild(bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// todo document
pub struct Reason(pub IexString<4>);
impl Reason{
    pub const fn as_str(&self) -> &str {self.0.as_str()}
    pub const fn is_vaild(bytes : &[u8; core::mem::size_of::<Self>()]) -> bool{
        IexString::<4>::is_vaild(bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TradeId(pub Little<i64>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub Little<i64>);


/// monotonic nanosecond since utc epoch
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp(pub Little<i64>);

/// duration since utc epoch
impl core::convert::TryFrom<Timestamp> for Duration{
    type Error = core::num::TryFromIntError;
    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        let unsigned_nanos = <i64 as core::convert::TryInto<u64>>::try_into(value.0.to_native())?;
        Ok(Duration::from_nanos(unsigned_nanos))
    }
}

/// 0.0001 units of currency
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Price(pub Little<i64>);

impl core::ops::Add for Price{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(Little::<i64>::from_native(self.0.to_native() + rhs.0.to_native()))
    }
}

impl core::ops::Sub for Price{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(Little::<i64>::from_native(self.0.to_native() - rhs.0.to_native()))
    }
}

impl From<Price> for f64{
    fn from(value: Price) -> Self {
        value.0.to_native() as f64 / 1000.0f64
    }
}

impl From<Price> for f32{
    fn from(value: Price) -> Self {
        value.0.to_native() as f32 / 1000.0f32
    }
}


impl From<f64> for Price{
    fn from(value: f64) -> Self {
        Self(Little::<i64>::from_native((value * 1000.0f64) as i64))
    }
}

impl From<f32> for Price{
    fn from(value: f32) -> Self {
        Self(Little::<i64>::from_native((value * 1000.0f32) as i64))
    }
}

// don't blame me in 2106
/// Seconds since utc epoch
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventTime(pub Little<u32>);

/// duration since utc epoch
impl From<EventTime> for Duration{
    fn from(value: EventTime) -> Self {
        Duration::new(value.0.to_native() as u64, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageParseError{
    /// The message type is unknown.
    /// Most likely a forward compatibility error where a new protocol version introduces a new packet type.
    Unknown,

    /// Currently means the packet is malformed,
    /// in the future it might indicate backward incompatibility.
    /// The current version of the parser is backwards compatable with all prevous version of DEEP, DEEP+ and TOPS.
    /// Howover IEX says they may add new fields in the future which would necessitate a backward compatibility break (forward compatibility would remain intact).
    Invalid
}

