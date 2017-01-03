#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate tokio_core;

use std::{env, io};
use std::net::SocketAddr;

use futures::{Future, Poll};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;


struct HullServer {
    socket: UdpSocket,
    buf: Vec<u8>,
    incoming: Option<(usize, SocketAddr)>,
}


impl Future for HullServer {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            if let Some((size, peer)) = self.incoming {
                println!("RECEIVED: {} bytes FROM {}", size, peer);
                self.incoming = None;
            }

            self.incoming = Some(try_nb!(self.socket.recv_from(&mut self.buf)));
        }
    }
}


fn main() {
    env_logger::init().expect("Unable to init logger");

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:48656".to_string());
    let addr = addr.parse::<SocketAddr>().expect("Error: cannot parse socket address");

    let mut ioloop = Core::new().expect("Error: cannot start event loop");
    let handle = ioloop.handle();
    let socket = UdpSocket::bind(&addr, &handle).expect("Error: cannot bind socket");

    println!("Listening on: {} using UDP", addr);

    ioloop.run(HullServer {
        socket: socket,
        buf: vec![0; 1024],
        incoming: None,
    }).unwrap();
}
