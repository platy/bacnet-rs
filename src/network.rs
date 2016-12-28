/// Module for procedures for the BACnet network layer described in clause 6.
///
/// The current implementation does not seek to implement routing and implements only what is
/// needed in the network layer for messages to be accepte by peers on a local BACnet segment.

use std::io;

#[derive(PartialEq, Debug)]
struct NetworkRequest {
    // destinationAddress
    data: Vec<u8>,
    network_priority: u8,
    data_expecting_reply: bool,
    // security_parameters
}

#[derive(PartialEq, Debug)]
struct NetworkIndication {
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

fn decode(data: &[u8]) -> io::Result<NetworkIndication> {
    use std::io::{Error, ErrorKind};

    if data[0] != 1 {
        return Err(Error::new(ErrorKind::InvalidData, format!("Unsupported: BACnet version : {}", data[0])));
    }
    println!("{}", data[1]);
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
    use super::NetworkRequest;
    use super::NetworkIndication;

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
        use std::io::{Error, ErrorKind};
        assert_eq!(ErrorKind::InvalidData, decode(&[0x2u8, 0b00000000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b10000000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b00100000, 1, 2]).unwrap_err().kind());
        assert_eq!(ErrorKind::InvalidData, decode(&[0x1u8, 0b00001000, 1, 2]).unwrap_err().kind());
    }
        
}

