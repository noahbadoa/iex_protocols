use iex_protocols::common::MessageParseError;
use iex_protocols::transport_latest::MessageProtocolId;
type ParseError<T> = core::result::Result<T, MessageParseError>;

pub trait ProtocolEnum : Sized{
    fn parse_packet(bytes : &[u8]) -> Option<Self>;
}

pub trait IexProtocol{
    const IDS : &'static [MessageProtocolId];
    type MessageEnum;
    fn parse_packet(bytes : &[u8]) -> ParseError<Self::MessageEnum>;
}

pub struct Deep<'a>{
    ignore : core::marker::PhantomData<&'a ()>
}
impl Deep<'static>{
    pub const THIS : Self = Self{ignore : core::marker::PhantomData};
}
impl<'a> IexProtocol for Deep<'a>{
    const IDS: &'static [MessageProtocolId] = &[MessageProtocolId::DEEP];
    type MessageEnum = iex_protocols::deep_latest::MessageEnum<'a>;
    fn parse_packet(bytes : &[u8]) -> ParseError<Self::MessageEnum> {
        iex_protocols::deep_latest::MessageEnum::parse_packet(bytes)
    }
}

pub struct TOPS<'a>{
    ignore : core::marker::PhantomData<&'a ()>
}
impl TOPS<'static>{
    pub const THIS : Self = Self{ignore : core::marker::PhantomData};
}
impl<'a> IexProtocol for TOPS<'a>{
    const IDS: &'static [MessageProtocolId] = &[MessageProtocolId::TOPS_V0, MessageProtocolId::TOPS_V1];
    type MessageEnum = iex_protocols::tops_latest::MessageEnum<'a>;
    fn parse_packet(bytes : &[u8]) -> ParseError<Self::MessageEnum> {
        iex_protocols::tops_latest::MessageEnum::parse_packet(bytes)
    }
}

pub struct DeepPlus<'a>{
    ignore : core::marker::PhantomData<&'a ()>
}
impl DeepPlus<'static>{
    pub const THIS : Self = Self{ignore : core::marker::PhantomData};
}
impl<'a> IexProtocol for DeepPlus<'a>{
    const IDS: &'static [MessageProtocolId] = &[MessageProtocolId::DEEP_PLUS];
    type MessageEnum = iex_protocols::deep_plus_latest::MessageEnum<'a>;
    fn parse_packet(bytes : &[u8]) -> ParseError<Self::MessageEnum> {
        iex_protocols::deep_plus_latest::MessageEnum::parse_packet(bytes)
    }
}
