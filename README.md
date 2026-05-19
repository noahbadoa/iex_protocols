Unoffical Bindings to IEX protocols.
Aims to be simple, zero-copy, and minimal (no-std). 

# Supported Protocols
The main goal of this crate is to parse historical data. As such it currently only supports formats that publicly available data uses.

- [x] Transport
- [ ] Options Transport
- [ ] FIX
- [ ] Options FIX
- [x] DEEP+
- [ ] DEEP+ SNAP
- [x] DEEP
- [ ] DEEP SNAP
- [ ] Options DEEP
- [x] TOPS
- [ ] TOPS SNAP
- [ ] Options TOPS

# Version Compatibility
The documentation for all protocols state that changes will only be made that either add new fields to the end of existing structs
or add new options to bitflags. This (theoretically) means that old version of this crate will be able to handle newer protocol version 
forever by simply ignoring the newly added data.

However, new protocol version may be backwards incompatible with older protocol versions. In that cases, the incompatible protocol
versions will be split into seperate modules. Non-breaking updates will be incorporated direcitly into existing modules. When parsing 
new data from the network use {protocol}_latest to always get the most up to date version. When parsing historical data it's better to 
explicitly specficy the protcol version. 

# Usage
For a complete example of parsing historical data wrapped in the full network stack see examples folder

```rust
use iex_protocols::{transport_latest, tops_latest, deep_latest, deep_plus_latest};
use iex_protocols::transport_latest::MessageProtocolId;
use iex_protocols::common::MessageParseError;

pub enum ImplementedProtocols<'a>{
    Deep(deep_latest::MessageEnum<'a>),
    Tops(tops_latest::MessageEnum<'a>),
    DeepPlus(deep_plus_latest::MessageEnum<'a>),
}

pub fn parse_transport_packet<'a>(packet : &'a [u8]) -> Option<Vec<ImplementedProtocols<'a>>>{
    let mut output : Vec<ImplementedProtocols<'a>> = Vec::new();
    let (header, messages) = transport_latest::TransportProtocalHeader::parse_packet(packet)?;

    let mut messages = messages;
    while let Some(ref message) = messages{
        let payload = message.current_payload();

        match header.message_protocol_id{
            MessageProtocolId::TOPS_V0 | MessageProtocolId::TOPS_V1 => {
                let payload = tops_latest::MessageEnum::parse_packet(payload);
                match payload {
                    Err(MessageParseError::Invalid) => {return None;}
                    Err(MessageParseError::Unknown) => {}
                    Ok(payload) => {
                        output.push(ImplementedProtocols::Tops(payload));
                    }
                }
            }

            MessageProtocolId::DEEP => {
                let payload = deep_latest::MessageEnum::parse_packet(payload);
                match payload {
                    Err(MessageParseError::Invalid) => {return None;}
                    Err(MessageParseError::Unknown) => {}
                    Ok(payload) => {
                        output.push(ImplementedProtocols::Deep(payload));
                    }
                }
            }

            MessageProtocolId::DEEP_PLUS => {
                let payload = deep_plus_latest::MessageEnum::parse_packet(payload);
                match payload {
                    Err(MessageParseError::Invalid) => {return None;}
                    Err(MessageParseError::Unknown) => {}
                    Ok(payload) => {
                        output.push(ImplementedProtocols::DeepPlus(payload));
                    }
                }
            }
            _ => {return None;}
        }

        messages = message.next();
    }

    Some(output)
}
```