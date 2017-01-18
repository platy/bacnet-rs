extern crate tokio_core;
extern crate env_logger;
extern crate futures;
extern crate bacnet;

use std::net::SocketAddr;
use std::str;
use std::io;
use std::env;

use futures::{Future, Stream, Sink};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use bacnet::bacnet_ip::{BipCodec, VLLFrame};
use bacnet::network;
use bacnet::serialise;
use bacnet::ast::ApduHeader;

/// whois-client destination-address [listen-address]
fn main() {
    drop(env_logger::init());

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let mut args = env::args().skip(1);
    let send_arg = args.next().unwrap();
    let bind_arg = args.next().unwrap_or("0.0.0.0:47808".to_string());
    let addr: SocketAddr = bind_arg.parse().unwrap();
    let b_addr: SocketAddr = send_arg.parse().unwrap();

    // Bind our socket
    let a = UdpSocket::bind(&addr, &handle).unwrap();

    // We're parsing each socket with the `BipCodec`, and then we
    // `split` each socket into the sink/stream halves.
    let (a_sink_t, a_stream) = a.framed(BipCodec).split();

    let a_sink = a_sink_t.with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}\n", &msg);
        Ok((addr, VLLFrame::OriginalUnicastNPDU(msg)))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}", &msg);
        let mut buf = Vec::new();
        network::encode(msg, &mut buf);
        Ok((addr, buf))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}", &msg);
        Ok((addr, network::NetworkRequest::expect_reply(msg)))
    }).with(move |(addr, apdu_header)| -> Result<_, io::Error> {
        println!("[a] sending: {:?}", &apdu_header);
        let mut buf = vec![];
        serialise::write_apdu_header(&mut buf, apdu_header);
        Ok((addr, buf))
    });

    // Start off by sending a ping from a to b, afterwards we just print out
    // what they send us and continually send pings
    // let pings = stream::iter((0..5).map(Ok));
    let a = a_sink.send((b_addr, ApduHeader::UnconfirmedReq { service: 8 })).and_then(|_| {
        let mut i = 0;
        a_stream.take(1).map(move |(_, msg)| {
            i += 1;
            println!("[a] recv: {:?}", &msg);
            ()
        }).into_future().map_err(|(err, _)| err) // makes a future to finish on the first received message
    });

    // it for our pinger to finish.
    drop(core.run(a));
}

