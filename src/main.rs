
extern crate futures;
use futures::{Future, Stream};
use futures::Async;
use futures::future::lazy;

extern crate tokio_core;
use tokio_core::net::{TcpListener,TcpStream};
use tokio_core::reactor::{Core};

extern crate tokio_io;
use tokio_io::{AsyncRead,AsyncWrite};
use tokio_io::codec::{Framed,LinesCodec};

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
    let listener = match TcpListener::bind(&socket, &handle0) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to start listening on {:?}", &socket);
            exit(1)
        }
    };

    // construct a future
    let server = listener.incoming().for_each(|(sock,addr)|{
        handle1.spawn(
            IRatherDislikeFutures::new(
                addr, 
                sock.framed(LinesCodec::new())));
        Ok(())
    });
}

struct IRatherDislikeFutures {
    addr: SocketAddr,
    stream: Framed<TcpStream,LinesCodec>
}
impl IRatherDislikeFutures {
    fn new(addr: SocketAddr, stream: Framed<TcpStream,LinesCodec>) -> Self {
        println!("new connection {:?}", &addr);
        IRatherDislikeFutures{ addr, stream }
    }
}
impl Future for IRatherDislikeFutures {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>,()> {
        match self.stream.poll() {
            Ok(Async::Ready(Option::Some(x))) => {
                println!("{}", x);
                Ok(Async::NotReady)
            }
            Ok(Async::Ready(Option::None)) => {
                println!("client {:?} closed", &self.addr);
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => {
                Ok(Async::NotReady)
            }
            Err(e) => {
                println!("client {:?} returned error {:?}", &self.addr, e);
                Err(())
            }
        }
    }
}

fn validate_socket(x: String) -> Result<(),String> {
    match SocketAddr::from_str(&x) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not parse input `{}` error {:?}", &x, e))
    }
}
