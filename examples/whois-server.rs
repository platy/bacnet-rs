extern crate tokio_core;
extern crate env_logger;
extern crate futures;
extern crate bacnet;

use std::net::SocketAddr;
use std::str;
use std::io;
use std::env;

use futures::{Stream, Sink};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use bacnet::bacnet_ip::{BipCodec, VLLFrame};
use bacnet::network;
use bacnet::serialise;
use bacnet::ast::ApduHeader;

/// whois-server [bind-address:port]
fn main() {
    drop(env_logger::init());

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let bind_arg = env::args().skip(1).next().unwrap_or("0.0.0.0:47808".to_string());
    let addr2: SocketAddr = bind_arg.parse().unwrap();

    // Bind both our sockets and then figure out what ports we got.
    let b = UdpSocket::bind(&addr2, &handle).unwrap();

    // We're parsing each socket with the `BipCodec`, and then we
    // `split` each socket into the sink/stream halves.
    let (b_sink_t, b_stream) = b.framed(BipCodec).split();

    let b_sink = b_sink_t.with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}\n", &msg);
        Ok((addr, VLLFrame::OriginalUnicastNPDU(msg)))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}", &msg);
        let mut buf = Vec::new();
        network::encode(msg, &mut buf);
        Ok((addr, buf))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}", &msg);
        Ok((addr, network::NetworkRequest::expect_reply(msg)))
    }).with(move |(addr, apdu_header)| -> Result<_, io::Error> {
        println!("[b] sending: {:?}", &apdu_header);
        let mut buf = vec![];
        serialise::write_apdu_header(&mut buf, apdu_header);
        Ok((addr, buf))
    });;  

    // The second client we have will receive the pings from `a` and then send
    // back pongs.
    let b_stream = b_stream.and_then(|(addr, msg)| {
        println!("[b] recv: {:?}", &msg);
        if let VLLFrame::OriginalUnicastNPDU(data) = msg {
            Ok((addr, try!(network::decode(data.as_slice()))))
        } else {
            panic!("Unsupported frame {:?}", msg);
        }
    }).map(|(addr, msg)| {
        println!("[b] recv: {:?}", &msg);
        (addr, ApduHeader::UnconfirmedReq { service: 0 })
    }).or_else(|err| {
        println!("[b] error: {}", err);
        Err(err)
    });
    let b = b_sink.send_all(b_stream);

    // Spawn the sender of pongs and then wait for our pinger to finish.
    drop(core.run(b));
}

