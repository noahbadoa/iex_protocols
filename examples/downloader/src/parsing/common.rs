use iex_protocols::endian::Little;
use iex_protocols::transport_latest::TransportProtocalHeader;
use pcap_parser::Linktype;
use smoltcp::wire::EthernetFrame;
use crate::parsing::{ErrorType, SizeShort};
use super::packet::IsTransmutable;
use super::protocol::IexProtocol;

impl IsTransmutable for TransportProtocalHeader{
    unsafe fn is_transmutable(slice : &[u8]) -> bool {
        if slice.len() >= Self::SIZE{
            let bytes = unsafe{std::mem::transmute(slice.as_ptr())};
            Self::is_vaild(bytes)
        }else{
            false
        }
    }
}

pub trait ProcessPacketFn<Message> {
    fn proccess_packet(&mut self, parsed: Message) -> impl core::future::Future<Output = bool> + Send + Sync;
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeLegacyPcapBlock{
    pub seconds : Little<u32>,
    pub timestamp : Little<u32>,
    pub captured_length : Little<u32>,
    pub original_length : Little<u32>,
}

impl NativeLegacyPcapBlock{
    pub const fn captured_length(&self) -> u32{
        let x = self.captured_length;
        x.to_native()
    }

    pub const fn original_length(&self) -> u32{
        let x = self.original_length;
        x.to_native()
    }
}

impl IsTransmutable for NativeLegacyPcapBlock{
    unsafe fn is_transmutable(bytes : &[u8]) -> bool {
        if bytes.len() < Self::SIZE {return false;}
        let this : &Self = unsafe{std::mem::transmute(bytes.as_ptr())};
        
        this.captured_length() as usize + Self::SIZE <= bytes.len()
    }
}

pub async fn parse_payload<Protocol : IexProtocol, Output : ProcessPacketFn<Protocol::MessageEnum>>(linktype: Linktype, payload : &[u8], output : &mut Output) -> Result<(), ErrorType> {
    let payload = match linktype {
        Linktype::ETHERNET => {
            let frame = EthernetFrame::<&[u8]>::new_checked(payload).ok().ok_or(ErrorType::InvaildData)?;
            frame.payload()
        }

        _unsupported => {return Err(ErrorType::Unimplemented);}
    };
    
    let ipv4_packet = smoltcp::wire::Ipv4Packet::new_checked(payload);
    let (packet_type, payload) = match ipv4_packet {
        Ok(packet) => {
            if !packet.verify_checksum(){
                return Err(ErrorType::InvaildData);
            }

            (packet.next_header(), packet.payload())
        }

        Err(_) => {
            let packet = smoltcp::wire::Ipv6Packet::new_checked(payload).ok().ok_or(ErrorType::InvaildData)?;

            (packet.next_header(), packet.payload())
        }
    };


    let payload = match packet_type {
        smoltcp::wire::IpProtocol::Udp => {
            let udp_packet = smoltcp::wire::UdpPacket::new_checked(payload).ok().ok_or(ErrorType::InvaildData)?;
            udp_packet.payload()
        }

        _unsupported => {return Err(ErrorType::Unimplemented);}
    };

    let (header, iter) = TransportProtocalHeader::parse_packet(payload).ok_or(ErrorType::InvaildData)?;

    let protocol = header.message_protocol_id;
    if !Protocol::IDS.contains(&protocol){
        return Err(ErrorType::Unimplemented);
    }

    let message_count = header.message_count;
    if message_count.to_native() == 0 {return Ok(());}
    let mut current_iter = Some(iter.ok_or(ErrorType::InvaildData)?);

    let expected = header.message_count;
    let expected = expected.to_native();
    let mut counter = 0;

    while let Some(ref current) = current_iter{
        counter += 1;

        if counter > expected{
            return Err(ErrorType::InvaildData);
        }
        
        let parsed: Result<<Protocol as IexProtocol>::MessageEnum, iex_protocols::common::MessageParseError> = Protocol::parse_packet(current.current_payload());
        match parsed{
            Err(iex_protocols::common::MessageParseError::Invalid) => {
                return Err(ErrorType::InvaildData);
            }

            Err(iex_protocols::common::MessageParseError::Unknown) => {
                return Err(ErrorType::Unimplemented);
            }

            Ok(packet) => {
                if !output.proccess_packet(packet).await{
                    return Err(ErrorType::Processing);
                }
            }
        }

        current_iter = current.next();
    }

    if counter != expected {return Err(ErrorType::InvaildData);}

    Ok(())
}
