extern crate env_logger;
extern crate getopts;
#[macro_use]
extern crate log;

extern crate irc_server;

use std::net::SocketAddr;

use irc_server::service;

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
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
    debug!("Socket addr {:?}.", socket_addr);

    service::start(socket_addr);
}
