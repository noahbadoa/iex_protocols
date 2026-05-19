mod pcap_parse;
mod pcapng_parse;
mod packet;
pub mod protocol;
mod parse;

mod common;
pub use common::ProcessPacketFn;
pub use parse::process_async;



trait SizeShort : Sized{
    const SIZE : usize = core::mem::size_of::<Self>();
}
impl<T : Sized> SizeShort for T{}

#[derive(Debug)]
pub enum ErrorType {
    Net(reqwest::Error),
    Io(std::io::Error),
    Processing,
    Unimplemented,
    InvaildData,
}

unsafe impl Send for ErrorType{}
unsafe impl Sync for ErrorType{}

impl From<std::io::Error> for ErrorType{
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for ErrorType{
    fn from(value: reqwest::Error) -> Self {
        Self::Net(value)
    }
}