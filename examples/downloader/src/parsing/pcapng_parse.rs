use pcap_parser::{traits::PcapNGPacketBlock, *};
use crate::parsing::common::{ProcessPacketFn, parse_payload};
use super::protocol::*;
use super::ErrorType;


use tokio::io::AsyncReadExt;
pub struct ShiftBuffer{
    alloc :*mut u8,
    capacity : usize,

    write_length : usize,
    read_length : usize
}

unsafe impl Send for ShiftBuffer{}
unsafe impl Sync for ShiftBuffer{}

impl ShiftBuffer{
    pub fn new(capacity : usize) -> Self{
        assert!(capacity != 0);
        let layout = std::alloc::Layout::from_size_align(capacity, 1).unwrap();
        let alloc = unsafe{std::alloc::alloc(layout)};

        Self::from_ptr(alloc, capacity)
    }

    const fn from_ptr(alloc :*mut u8, capacity : usize) -> Self{
        Self { alloc, capacity, write_length: 0, read_length: 0 }
    }

    const fn unutilized_section(&mut self) -> &mut [u8]{
        unsafe{core::slice::from_raw_parts_mut(self.alloc.add(self.write_length), self.capacity - self.write_length)}
    }

    pub const fn intilized_section(&mut self) -> &mut [u8]{
        unsafe{core::slice::from_raw_parts_mut(self.alloc.add(self.read_length), self.write_length - self.read_length)}
    }

    pub const unsafe fn advance(&mut self, by : usize){
        self.read_length += by;
    }

    // language lawyers look away
    // read traits technically requires buffer to be initialized https://doc.rust-lang.org/std/io/trait.Read.html
    pub async fn fill<IO : tokio::io::AsyncRead + std::marker::Unpin>(&mut self, read : &mut IO) -> Result<bool, std::io::Error>{
        let read_bytes = read.read(self.unutilized_section()).await?;
        self.write_length += read_bytes;

        Ok(read_bytes == 0)
    }

    pub const fn fill_bytes(&mut self, bytes : &[u8]){
        assert!((self.capacity - self.write_length) >= bytes.len());
        unsafe{
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), self.alloc.add(self.write_length), bytes.len());
        }
        
        self.write_length += bytes.len();
    }

    pub const fn shift(&mut self){
        unsafe{
            core::ptr::copy(self.alloc.add(self.read_length), self.alloc, self.write_length - self.read_length);
        }

        self.write_length = self.write_length - self.read_length;
        self.read_length = 0;
    }
}

impl Drop for ShiftBuffer{
    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.capacity, 1).unwrap();
        unsafe{std::alloc::dealloc(self.alloc, layout);}
    }
}

#[derive(Default)]
pub struct ParserState{
    block_linktypes : Vec<Linktype>,
}

pub async fn process_pcapng<Input : tokio::io::AsyncRead + std::marker::Unpin + Send + Sync, Protocol : IexProtocol, Output : ProcessPacketFn<Protocol::MessageEnum>>(leftovers : &[u8], input : &mut Input, process : &mut Output) -> Result<(), ErrorType>{
    let mut state = ParserState::default();

    let mut buffer = ShiftBuffer::new(2usize.pow(16));
    buffer.fill_bytes(leftovers);

    let mut finished_reading = false;

    loop {
        match finished_reading{
            false => {
                let section = buffer.intilized_section();
                let parsed = pcap_parser::pcapng::parse_block_le(section);

                let (extra, block) = match parsed {
                    Ok(ok) => {ok},

                    Err(error) => {
                        if let pcap_parser::nom::Err::Incomplete(_needed) = error {
                            buffer.shift();
                            finished_reading = buffer.fill(input).await?;

                            continue;
                        }
                        
                        return Err(ErrorType::InvaildData);
                    }
                };

                match_blocktype::<Protocol, Output>(&mut state, process, block).await?;

                let read = unsafe{extra.as_ptr().offset_from_unsigned(section.as_ptr())};
                unsafe{buffer.advance(read);}
            }

            true => {
                let section = buffer.intilized_section();
                if section.is_empty() {return Ok(());}

                let out = pcap_parser::pcapng::parse_block_le(section);
                if out.is_err() {return Err(ErrorType::InvaildData);}
                let (extra, block) = out.unwrap();
                match_blocktype::<Protocol, Output>(&mut state, process, block).await?;

                let read = unsafe{extra.as_ptr().offset_from_unsigned(section.as_ptr())};
                unsafe{buffer.advance(read);}
            }
        }
    }
}

async fn match_blocktype<Protocol : IexProtocol, Output : ProcessPacketFn<Protocol::MessageEnum>>(state : &mut ParserState, process : &mut Output, block: Block<'_>) -> Result<(), ErrorType>{

    match block {
        Block::SectionHeader(ref _shb) => {
            state.block_linktypes.clear();
        },

        Block::InterfaceDescription(ref idb) => {
            state.block_linktypes.push(idb.linktype);
        },

        Block::EnhancedPacket(ref epb) => {
            if !((epb.if_id as usize) < state.block_linktypes.len()) {return Err(ErrorType::InvaildData);} 

            let linktype: Linktype = state.block_linktypes[epb.if_id as usize];

            if epb.truncated() {return Err(ErrorType::Unimplemented);}

            parse_payload::<Protocol, Output>(linktype, epb.data, process).await?;
        },

        Block::SimplePacket(ref spb) => {
            if state.block_linktypes.is_empty() {return Err(ErrorType::InvaildData);} 

            let linktype = state.block_linktypes[0];
            let _blen = (spb.block_len1 - 16) as usize;

            if spb.truncated() {return Err(ErrorType::Unimplemented);}

            parse_payload::<Protocol, Output>(linktype, spb.data, process).await?;
        },

        _ => {}
    }

    Ok(())
}
