extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate futures_cpupool;

use tokio_core::net::{TcpStream, TcpListener};
use tokio_core::reactor::Core;
use tokio_io::io::copy;
use tokio_io::AsyncRead;
use std::net::SocketAddr;
use std::io;
use futures::{future, Future, Stream};
use futures_cpupool::CpuPool;

mod parser;

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

// Run in thread pool thread. Safe-ish to block.
//fn handler(stream: TcpStream, remote: SocketAddr) -> //
/*fn handler<F>(stream: TcpStream, remote: SocketAddr) -> impl Future<Item =(), Error = ()>
where F: Future<
Box<Future<Item = (), Error = ()>> {
    debug!("{:?}: {:?}", stream, remote);
    let (reader, writer) = stream.split();
    let fut = copy(reader, writer).then(move |result| {
        match result {
            Ok((amt, _, _)) => debug!("Wrote {} bytes.", amt),
            Err(e) => error!("Error on {:?}: {:?}.", remote, e),
        };
        Ok(())
    });

    Box::new(fut)
    //Box::new(future::ok::<bool, ()>(true))
    //Ok(())
}*/

fn start_server(addr: &SocketAddr) -> io::Result<()> {
    let mut core = Core::new()?;
    let cpu_pool = CpuPool::new_num_cpus();
    let handle = core.handle();
    let listener = TcpListener::bind(addr, &handle)?;
    let server = listener.incoming().for_each(|(sock, _remote)| {
        let (reader, writer) = sock.split();
        cpu_pool.spawn(move || {
            copy(reader, writer)
                .map(|amt| debug!("Wrote {:?} bytes.", amt))
                .map_err(|e| warn!("Failed to copy bytes: {:?}.", e))
        });
        // Spawn in select loop.
        /*handle.spawn(move || {
            // Spawn in CpuPool.
            cpu_pool.spawn_fn(move || handler(sock, remote))
        });*/
        //cpu_pool.spawn(handler(sock, remote));
        Ok(())
    });
    core.run(server)
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string());
        }
    };

    if matches.opt_present("h") || matches.free.len() != 1 {
        print_usage(&program, opts);
        return;
    }

    let mut addr = "127.0.0.1:".to_string();
    addr.push_str(&matches.free[0]);
    let socket_addr = match addr.parse::<SocketAddr>() {
        Err(e) => {
            print_usage(&program, opts);
            println!("Bad address to listen on {}: {}.", addr, e.to_string());
            return;
        }
        Ok(a) => a,
    };

    match start_server(&socket_addr) {
        Err(ref e) => error!("server failed: {}", e.to_string()),
        _ => {}
    }
}
