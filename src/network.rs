//! Module for procedures for the BACnet network layer described in clause 6.
//!
//! The current implementation does not seek to implement routing and implements only what is
//! needed in the network layer for messages to be accepte by peers on a local BACnet segment.

use std::io;

#[derive(PartialEq, Debug)]
pub struct NetworkRequest {
    // destinationAddress
    data: Vec<u8>,
    network_priority: u8,
    data_expecting_reply: bool,
    // security_parameters
}

#[derive(PartialEq, Debug)]
pub struct NetworkIndication {
    // source_addres
    // destination_address
    data: Vec<u8>,
    network_priority: u8,
    data_expecting_reply: bool,
    // security_parameters
}

struct _NetworkReleaseRequest {
    // destination_address
}

struct _NetworkReportIndication {
    // peer_address
    // error_condition
    // error_parameters
    // security_parameters
}

impl NetworkRequest {
    /// Construct a network request for the given data expecting no reply
    pub fn noreply(data: Vec<u8>) -> NetworkRequest {
        NetworkRequest {
            data: data,
            network_priority: 0,
            data_expecting_reply: false,
        }
    }

    /// Construct a network request for the given data expecting a reply
    pub fn expect_reply(data: Vec<u8>) -> NetworkRequest {
        NetworkRequest {
            data: data,
            network_priority: 0,
            data_expecting_reply: true,
        }
    }
}

// Encode a network request for transmission.
//
// # Note
// 
// Currently only local networking is supported
//
// # Panics
//
// If network priority is greater than 3
pub fn encode(request: NetworkRequest, into: &mut Vec<u8>) {
    assert!(request.network_priority < 4, format!("Valid network priorities are 0..3, provided : {}", request.network_priority));
    into.push(0x01);
    let reply_flag = if request.data_expecting_reply { 4 } else { 0 };
    into.push(reply_flag ^ request.network_priority);
    into.extend(request.data);
}

// Decode an NPDU into a network indication
pub fn decode(data: &[u8]) -> io::Result<NetworkIndication> {
    use std::io::{Error, ErrorKind};

    if data[0] != 1 {
        return Err(Error::new(ErrorKind::InvalidData, format!("Unsupported: BACnet version : {}", data[0])));
    }
    if (data[1] & (1 << 7)) > 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Unsupported: Network layer message"));
    }
    if (data[1] & (1 << 5)) > 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Unsupported: messages with foreign destination"));
    }
    if (data[1] & (1 << 3)) > 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Unsupported: messages with foreign source"));
    }


    Ok(NetworkIndication {
        data: data[2..].to_vec(),
        network_priority: data[1] & 0b011,
        data_expecting_reply: data[1] & 0b100 > 0,
    })
}

#[cfg(test)]
mod test {
    use super::decode;
    use super::encode;
    use super::NetworkRequest;
    use super::NetworkIndication;

    #[test]
    fn encode_local_0() {
        let data = vec![0x01u8, 0x02, 0x03];
        let mut buf = Vec::new();
        encode(NetworkRequest {
            data: data.clone(),
            network_priority: 0,
            data_expecting_reply: false,
        }, &mut buf);
        let mut expecting = vec![0x01, 0x00];
        expecting.extend(data);
        assert_eq!(buf, expecting);
    }

    #[test]
    fn encode_local_3() {
        let data = vec![0x01u8, 0x02, 0x03];
        let mut buf = Vec::new();
        encode(NetworkRequest {
            data: data.clone(),
            network_priority: 3,
            data_expecting_reply: true,
        }, &mut buf);
        let mut expecting = vec![0x01, 0x07];
        expecting.extend(data);
        assert_eq!(buf, expecting);
    }

    #[test]
    fn decode_local() {
        assert_eq!(NetworkIndication {
            data: vec![1, 2],
            network_priority: 0,
            data_expecting_reply: false,
        }, decode(&[0x1u8, 0b00000000, 1, 2]).unwrap());
        assert_eq!(NetworkIndication {
            data: vec![1, 2],
            network_priority: 1,
            data_expecting_reply: true,
        }, decode(&[0x1u8, 0b00000101, 1, 2]).unwrap());
        assert_eq!(NetworkIndication {
            data: vec![1, 2],
            network_priority: 3,
            data_expecting_reply: true,
        }, decode(&[0x1u8, 0b00000111, 1, 2]).unwrap());
    }

    #[test]
    fn decode_unsupported() {
        use std::io::ErrorKind;
        assert_eq!(ErrorKind::InvalidData, decode(&[0x2u8, 0b00000000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b10000000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b00100000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b00001000, 1, 2]).unwrap_err().kind());
    }
        
}

