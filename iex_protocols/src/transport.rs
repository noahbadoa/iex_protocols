


use crate::{common::Timestamp, endian::Little};
use core::mem::{offset_of, size_of};

trait SizeShort : Sized{
    const SIZE : usize = core::mem::size_of::<Self>();
}

impl<T : Sized> SizeShort for T{}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version(pub u8);

impl Version{
    pub const V1 : Self = Self(1);
}


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageProtocolId(pub Little<u16>);

impl MessageProtocolId{
    pub const fn raw(&self) -> u16{self.0.to_raw()}
    pub const fn native(&self) -> u16{self.0.to_native()}

    pub const TOPS_V0 : Self = Self(Little::<u16>::from_native(0x8002));
    pub const TOPS_V1 : Self = Self(Little::<u16>::from_native(0x8003));
    pub const DEEP : Self = Self(Little::<u16>::from_native(0x8004));
    pub const DEEP_PLUS : Self = Self(Little::<u16>::from_native(0x8005));
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportProtocalHeader{
    pub version : Version,
    pub reserved : Little<u8>,
    pub message_protocol_id : MessageProtocolId,
    pub channel_id : Little<u32>,
    pub session_id : Little<u32>,

    /// Payload Length field value does not include the length of the IEX-TP Header
    pub payload_length : Little<u16>,
    pub message_count : Little<u16>,
    pub stream_offset : Little<i64>,
    pub first_message : Little<i64>,
    pub send_time : Timestamp
}

impl TransportProtocalHeader{
    pub const fn is_vaild(_bytes : &[u8; size_of::<Self>()]) -> bool{
        true
    }

    pub const fn parse_packet<'a>(bytes : &'a [u8]) -> Option<(&'a Self, Option<MessageBlockPacket<'a>>)>{
        if !Self::is_vaild(unsafe{core::mem::transmute(bytes.as_ptr())}){
            return None;
        }

        let this : &Self = unsafe{core::mem::transmute(bytes.as_ptr())};
        let message_count = this.message_count;

        if message_count.to_native() == 0{
            return Some((this, None));
        }

        // let size = this.payload_length as usize;
        // let start = this.stream_offset as usize + core::mem::size_of::<Self>();
        // let message_block = &bytes[start..(start + size)];
        // let message_block = &bytes[Self::SIZE..];
        let payload_length = this.payload_length;
        let message_block = unsafe{core::slice::from_raw_parts(bytes.as_ptr().add(Self::SIZE), payload_length.to_native() as usize)};

        let message = MessageBlockPacket::new_checked(message_block);
        if message.is_none() {return None;}

        Some((this, message))
    }
}


#[repr(C, packed)]
pub struct MessageBlock{
    pub length : Little<u16>
}

pub struct MessageBlockPacket<'a>(pub &'a [u8]);


impl<'a> MessageBlockPacket<'a>{
    pub const fn new_checked(data : &'a [u8]) -> Option<Self>{
        if data.len() < MessageBlock::SIZE {return None;}
        
        // read is on unaligned struct and not field itself so read safe
        let block = unsafe{data.as_ptr().cast::<MessageBlock>().read()}.length;

        if data.len() < (MessageBlock::SIZE + block.to_native() as usize) {return None;}

        let out = Self(data);
        Some(out)
    }

    pub const fn block(&self) -> &MessageBlock{
        unsafe{core::mem::transmute(self.0.as_ptr())}
    }

    pub const fn current_payload(&self) -> &[u8]{
        let length = self.block().length;

        unsafe{core::slice::from_raw_parts(self.0.as_ptr().add(MessageBlock::SIZE), length.to_native() as usize)}
    }

    pub const fn next(&self) -> Option<Self>{
        let block_length = self.block().length;

        let offset = MessageBlock::SIZE + block_length.to_native() as usize;
        let next_size = self.0.len().checked_sub(offset);
        if next_size.is_none() {return None;}
        let next_size = next_size.unwrap();

        let next_storage = unsafe{core::slice::from_raw_parts(self.0.as_ptr().add(offset), next_size)};
        Self::new_checked(next_storage)
    }
}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RequestType{
    SequencedMessages = 0x1,
    Bytestream = 0x2
}

impl RequestType{
    pub const fn is_vaild(src : u8) -> bool{
        (src == (Self::SequencedMessages as u8)) || (src == (Self::Bytestream as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BytestreamRequestRangeBlock{
    pub start_offset : u64,
    pub end_offset : u64,
}

impl BytestreamRequestRangeBlock{
    pub const KIND : RequestType = RequestType::Bytestream;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, packed)]
pub struct GapFillRequest{
    pub version : Little<u8>,
    pub request_type : RequestType,
    pub message_protocol_id : MessageProtocolId,
    pub channel_id : Little<u32>,
    pub session_id : Little<u32>,
    pub request_range_count : Little<u32>
}

impl GapFillRequest{
    pub const fn is_vaild(bytes : &[u8; Self::SIZE]) -> bool{
        RequestType::is_vaild(bytes[offset_of!(Self, request_type)])
    } 
}


