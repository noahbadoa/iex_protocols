use crate::parsing::ErrorType;
use crate::parsing::{ProcessPacketFn, protocol::IexProtocol, SizeShort};
use crate::parsing::pcap_parse::ref_to_mut_bytes;
use super::pcap_parse::process_pcap_body;
use super::pcapng_parse::process_pcapng;
use tokio::io::AsyncReadExt;

pub async fn process_async<Input : tokio::io::AsyncRead + std::marker::Unpin + Send + Sync, Protocol : IexProtocol, Output : ProcessPacketFn<Protocol::MessageEnum>>(input : &mut Input, process : &mut Output) -> Result<(), ErrorType>{
    let mut header = std::mem::MaybeUninit::<[u8; pcap_parser::pcap::PcapHeader::SIZE]>::uninit();
    input.read_exact(ref_to_mut_bytes(&mut header)).await?;
    let bytes = unsafe{header.assume_init()};
    let header = pcap_parser::pcap::parse_pcap_header(bytes.as_slice());

    match header{
        Err(_) => {
            process_pcapng::<Input, Protocol, Output>(bytes.as_slice(), input, process).await
        }

        Ok((_, header)) => {
            process_pcap_body::<Input, Protocol, Output>(header, input, process).await
        }
    }
}
