extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate futures_cpupool;

use std::net::SocketAddr;
use std::io::{BufRead, BufReader};

use tokio_core::net::{TcpStream, TcpListener};
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;
use tokio_io::io;

use futures::{future, Future, Stream};
use futures_cpupool::CpuPool;

mod parser;

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

fn process_line<A>(
    thread_pool: &CpuPool,
    line: Vec<u8>,
    buffered_reader: A,
) -> Box<Future<Item = (), Error = std::io::Error> + Send>
where
    A: BufRead,
{
    debug!("Processing line: {}.", String::from_utf8(line).unwrap());
    debug!("Reading next line...");
    line.clear();
    return thread_pool.spawn(io::read_until(buffered_reader, '\n' as u8, line).then(
        |r| {
            match r {
                Ok((buf, line)) => process_line(thread_pool, buf, line),
                Err(e) => Err(e),
            }
        },
    ));
    /*Box::new(
        io::read_until(reader, '\n' as u8, Vec::new())
            .then(|r| match r {
                Ok((buf, res)) => {
                    debug!("Read line: [{}].", String::from_utf8(res).unwrap());
                    io::read_until(buf, '\n' as u8, Vec::new())
                }
                Err(e) => panic!("{:?}", e),
            })
            .then(|r| match r {
                Ok((_, res)) => {
                    debug!("Read second line: [{}].", String::from_utf8(res).unwrap());
                    future::ok::<(), std::io::Error>(())
                }
                Err(e) => panic!("{:?}", e),
            }),
    )*/
}

fn start_server(addr: SocketAddr) -> std::io::Result<()> {
    let mut event_loop = Core::new()?;
    let thread_pool = CpuPool::new(1);
    let handle = event_loop.handle();
    let listener = TcpListener::bind(&addr, &handle)?;

    let server = listener.incoming().for_each(|(sock, remote)| {
        // The main thread's whole purpose is to accept incoming connections and dispatches work
        // into CpuPool threads.
        // http://berb.github.io/diploma-thesis/original/042_serverarch.html
        let tp = thread_pool.clone();
        handle.spawn_fn(move || {
            debug!("{:?}: {:?}", sock, remote);
            let (raw_reader, _writer) = sock.split();
            let reader = BufReader::new(raw_reader);
            tp.spawn(io::read_until(reader, '\n' as u8, Vec::new()).then(
                |r| match *r {
                    Ok((buf, line)) => process_line(tp, buf, line),
                    Err(e) => Err(e),
                },
            )).map_err(|e| {
                    warn!("Discarding worker thread error {:?}.", e);
                });
        });

        Ok(())
    });
    event_loop.run(server)
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

    match start_server(socket_addr) {
        Err(ref e) => error!("server failed: {}", e.to_string()),
        _ => {}
    }
}
