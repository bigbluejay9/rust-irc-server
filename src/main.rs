extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate getopts;
#[macro_use]
extern crate log;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::net::SocketAddr;
use std::str;
use std::io;

use bytes::BytesMut;

use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;
use tokio_proto::TcpServer;
use tokio_service::Service;

use futures::{future, Future};

struct CRLFCodec;

impl Encoder for CRLFCodec {
    type Item = String;
    type Error = io::Error;
    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), io::Error> {
        dst.extend(item.as_bytes());
        dst.extend(b"\r\n");
        Ok(())
    }
}

impl Decoder for CRLFCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<String>, io::Error> {
        let mut crlf_pos: Option<usize> = None;
        for (pos, &c) in src.iter().enumerate() {
            if pos > 1 && c == 0x0A && src[pos - 1] == 0x0D {
                crlf_pos = Some(pos);
                break;
            }
        }

        match crlf_pos {
            Some(pos) => {
                debug!("Saw line: {:?}.", src);
                let line = &src.split_to(pos + 1)[0..(pos - 1)];
                match str::from_utf8(&line) {
                    Ok(s) => Ok(Some(s.to_string())),
                    // TODO(lazau): Maybe optionally support ISO-8859-1?
                    Err(_) => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "not valid utf-8 string",
                    )),
                }
            }
            None => Ok(None),
        }
    }
}

struct LineProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    type Request = String;
    type Response = String;

    type Transport = Framed<T, CRLFCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CRLFCodec))
    }
}

struct IRC;

impl Service for IRC {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(future::ok(req))
    }
}

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

fn start_server(addr: SocketAddr) {
    debug!("Starting server on: {:?}.", addr);
    let server = TcpServer::new(LineProto, addr);
    server.serve(|| Ok(IRC));
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string());
        }
    };

    if matches.opt_present("h") || matches.free.len() != 1 {
        print_usage(&args[0], opts);
        return;
    }

    let mut addr = "127.0.0.1:".to_string();
    addr.push_str(&matches.free[0]);
    let socket_addr = match addr.parse::<SocketAddr>() {
        Err(e) => {
            print_usage(&args[0], opts);
            println!("Bad address to listen on {}: {}.", addr, e.to_string());
            return;
        }
        Ok(a) => a,
    };

    start_server(socket_addr);
}
