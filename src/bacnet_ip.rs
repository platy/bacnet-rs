/// BACnet/IP is a virtual link layer for BACnet using UDP as the underlying transport
/// and is described in Annex J of the BACnet spec it includes necessary link control
/// messages to enable BACnet internetworking.
///
/// This implementation currently only supports funtion 10 (Original unicast NPDU) 
/// which allows the upper layers of BACnet to operate over a single segment BACnet/IP
/// network.

use std::io;
use std::net::SocketAddr;
use std::str;

use futures::{Future, Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;

pub struct BipCodec;

impl UdpCodec for BipCodec {
    type In = (SocketAddr, VLLFrame);
    type Out = (SocketAddr, VLLFrame);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        use std::io::Error;
        use std::io::ErrorKind;

        if buf.len() < 4 {
            return Err(io::Error::new(ErrorKind::InvalidData, "BVLL UDP packet must be at least 4 octets"))
        }
        if buf[0] != 0x81 {   // BVLL for BACnet/IP
            return Err(Error::new(ErrorKind::InvalidData, "Does not appear to be a BVLL for Bacnet/IP message"));
        }
        let length_field = (buf[2] as u16) << 8 ^ (buf[3] as u16);
        if length_field != buf.len() as u16 {
            return Err(Error::new(ErrorKind::InvalidData, format!("Length field ({}) does not match actual message length ({})", length_field, buf.len())));
        }
        let frame = match buf[1] {
            0xA => {            // J.2.11.1 Original-Unicast-NPDU
                VLLFrame::OriginalUnicastNPDU(buf[4..].to_vec())
            },
            _ =>
                return Err(io::Error::new(ErrorKind::InvalidData, "Unsupported BVLL message"))
        };
        Ok((*addr, frame))
    }

    fn encode(&mut self,
            (addr, frame): (SocketAddr, VLLFrame),
            into: &mut Vec<u8>) -> SocketAddr {
        into.push(0x81);        // BVLL for BACnet/IP
        let (function, body) = match frame {
            VLLFrame::OriginalUnicastNPDU(data) => (0xA, data), // J.2.11 Original-Unicast-NPDU
        };
        into.push(function);    // BVLC Function : 1-octet
        let length: u16 = 4 + body.len() as u16;
        into.push((length >> 8) as u8);
        into.push((length & 0xF) as u8);
        into.extend(body);
        return addr
    }
}

// Virtual link layer messages (J.2)
#[derive(PartialEq, Debug)]
pub enum VLLFrame {
    // TODO VLC control messages
    OriginalUnicastNPDU(Vec<u8>),   // J.2.11 
}


#[test]
fn decodeErr() {
    let addr: SocketAddr = "0.0.0.0:50".parse().unwrap();
    assert_eq!(BipCodec.decode(&addr, &[0x45, 0, 0, 4]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(BipCodec.decode(&addr, &[0x81, 0, 0, 4]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(BipCodec.decode(&addr, &[0x81, 10, 1, 4]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(BipCodec.decode(&addr, &[0x81, 10, 0, 5]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    assert_eq!(BipCodec.decode(&addr, &[]).unwrap_err().kind(), io::ErrorKind::InvalidData);
    
}

#[test]
fn decodeNPDU() {
    let addr: SocketAddr = "0.0.0.0:50".parse().unwrap();
    let mut data = vec![0x81u8, 0x0a, 0, 9];
    data.extend("Hello".as_bytes());
    assert_eq!(BipCodec.decode(&addr, &data).unwrap(), (addr, VLLFrame::OriginalUnicastNPDU("Hello".as_bytes().to_vec())));
}

#[test]
fn encodeNPDU() {
    let addr: SocketAddr = "0.0.0.0:50".parse().unwrap();
    let mut buf = Vec::new();
    let npdu = "Hello".as_bytes().to_vec();
    let frame = VLLFrame::OriginalUnicastNPDU(npdu.clone());
    assert_eq!(BipCodec.encode((addr, frame), &mut buf), addr);
    let mut expected = vec![0x81u8, 0x0a, 0, 9];
    expected.extend(npdu);
    assert_eq!(expected, buf);
}
