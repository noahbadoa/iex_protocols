use super::protocol::*;

use super::{SizeShort, ErrorType};
use super::packet::Packet;
use pcap_parser::PcapHeader;
use tokio::io::AsyncReadExt;
use super::common::*;

pub const fn ref_to_mut_bytes<T : Sized>(val : &mut T) -> &mut [u8]{
    unsafe{
        core::slice::from_raw_parts_mut(core::ptr::from_mut(val).cast::<u8>(), std::mem::size_of::<T>())
    }
}

pub struct BoxedAlloc{
    pub layout : std::alloc::Layout,
    pub ptr : *mut u8
}

impl BoxedAlloc{
    pub fn new(layout : std::alloc::Layout) -> Self{
        let ptr = unsafe{std::alloc::alloc(layout)};

        Self { layout, ptr}
    }
}

unsafe impl Send for BoxedAlloc{}
unsafe impl Sync for BoxedAlloc{}
impl Drop for BoxedAlloc{
    fn drop(&mut self) {
        unsafe{std::alloc::dealloc(self.ptr, self.layout);}
    }
}

pub async fn process_pcap_body<Input : tokio::io::AsyncRead + std::marker::Unpin + Send + Sync, Protocol : IexProtocol, Output : ProcessPacketFn<Protocol::MessageEnum>>(header : PcapHeader, input_stream : &mut Input, output : &mut Output) -> Result<(), ErrorType>{
    if header.is_bigendian() {return Err(ErrorType::Unimplemented);}
    if header.is_modified_format() {return Err(ErrorType::Unimplemented);}

    let min_read_size = header.snaplen + NativeLegacyPcapBlock::SIZE as u32;
    let min_buffer_size = 2u32.pow(16);
    let buffer_size = (min_read_size * 2).max(min_buffer_size);

    let mut buffer_read_size = 0;
    let layout = std::alloc::Layout::from_size_align(buffer_size as usize, 16).unwrap();
    let buffer = BoxedAlloc::new(layout);

    let buffer = unsafe{std::slice::from_raw_parts_mut(buffer.ptr, buffer_size as usize)};
 
    let mut finished_reading = false;
    while !finished_reading {
        // filling
        while (buffer_read_size < min_read_size as usize) & (!finished_reading) {
            let uninit_buffer = unsafe{std::slice::from_raw_parts_mut(buffer.as_mut_ptr().add(buffer_read_size), buffer.len() - buffer_read_size)};
            let read_bytes = input_stream.read(uninit_buffer).await?;
            
            finished_reading |= read_bytes == 0;
            buffer_read_size += read_bytes;
        }

        
        let mut start = 0;
        let end = buffer_read_size;

        // draining
        loop {            
            let slice = unsafe{std::slice::from_raw_parts(buffer.as_ptr().add(start), end - start)};
            let packet: Option<Packet<NativeLegacyPcapBlock, &[u8]>> = Packet::new_checked(slice);
            if packet.is_none() {break;}
            let packet: Packet<NativeLegacyPcapBlock, &[u8]> = packet.ok_or(ErrorType::InvaildData)?;
            if packet.payload().len() < packet.data().captured_length() as usize {break;}
            let packet_header: &NativeLegacyPcapBlock = packet.data();

            if packet_header.original_length() != packet_header.captured_length() {return Err(ErrorType::Unimplemented);}

            let truncated_payload = &packet.payload()[0..(packet.data().captured_length() as usize)];
            start += packet.data().captured_length() as usize + NativeLegacyPcapBlock::SIZE;

            parse_payload::<Protocol, Output>(header.network, truncated_payload, output).await?;
        }   
        
        // clearing 
        let leftover_bytes = end - start;
        unsafe{std::ptr::copy(buffer.as_ptr().add(start), buffer.as_mut_ptr(), leftover_bytes);}
        buffer_read_size = leftover_bytes;
    }

    if buffer_read_size != 0 {return Err(ErrorType::InvaildData);}

    Ok(())
}
