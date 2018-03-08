
extern crate futures;
use futures::{Future, Stream};
use futures::Async;
use futures::future::lazy;

extern crate tokio_core;
use tokio_core::net::{UdpSocket,UdpCodec, UdpFramed};
use tokio_core::reactor::{Core};

extern crate tokio_io;
use tokio_io::{AsyncRead,AsyncWrite};

extern crate bytes;

extern crate clap;
use clap::{App,Arg};

use std::str::FromStr;
use std::net::SocketAddr;
use std::process::exit;
use std::io;

fn main() {

    // parse args
    let args = App::new("syslogsrvr")
        .version("1.0.0")
        .author("cody <codylaeder@gmail.com>")
        .about("really shitty syslog server")
        .arg(Arg::with_name("socket")
             .short("s")
             .long("socket")
             .value_name("SOCKET")
             .required(true)
             .validator(validate_socket)
             .help("Pass in a socket to be listened upon"))
        .get_matches();

    // parse socket
    let socket = SocketAddr::from_str(args.value_of("socket").unwrap()).unwrap();


    // set up epoll instance
    let mut core = match Core::new() {
        Ok(x) => x,
        Err(e) => {
            println!("Could not build epoll instance. Error {:?}", e);
            exit(1)
        }
    };
    let handle0 = core.handle();
    let handle1 = core.handle();

    // start listening
    let future = match UdpSocket::bind(&socket, &handle0) {
        Ok(x) => DGramFuture{ data: x.framed(UdpGarabage::default()) },
        Err(e) => {
            println!("Failed to start listening on {:?} error {:?}", &socket, e);
            exit(1)
        }
    };

    match core.run(future) {
        Ok(()) => {
            println!("server exited");
        }
        Err(e) => {
            println!("server exited {:?}", e)
        }
    };
}

struct DGramFuture {
    data: UdpFramed<UdpGarabage>
}
impl Future for DGramFuture {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>,()> {
        loop {
            match self.data.poll() {
                Ok(Async::Ready(Option::None)) => {
                    println!("closed");
                    return Ok(Async::Ready(()));
                },
                Ok(Async::NotReady) => {
                    continue;
                },
                Err(e) => {
                    println!("closing error {:?}", e);
                    return Err(());
                },
                Ok(Async::Ready(Option::Some(ref x))) => {
                    println!("{:?}: {}", &x.addr, &x.message);
                    return Ok(Async::NotReady);
                },
            };
        }
    }
}

struct DGram {
    addr: SocketAddr,
    message: String,
}
struct UdpGarabage(());
impl Default for UdpGarabage {
    fn default() -> Self {
        UdpGarabage(())
    }
}
impl UdpCodec for UdpGarabage {
    type In = DGram;
    type Out = ();
    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> io::Result<DGram> {
        Ok(DGram{
            addr: src.clone(),
            message: String::from_utf8_lossy(buf).to_string(),
        })
    }
    fn encode(&mut self, msg: (), buf: &mut Vec<u8>) -> SocketAddr {
        panic!("shouldn't be called")
    }
}

fn validate_socket(x: String) -> Result<(),String> {
    match SocketAddr::from_str(&x) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not parse input `{}` error {:?}", &x, e))
    }
}
