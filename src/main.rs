extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;

mod parser;

use std::net::SocketAddr;

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

fn start_server(_addr: &SocketAddr, command: &String) -> Result<(), parser::errors::ParseError> {
    print!("Ok {:?}\n", parser::parse_command(command)?);
    Ok(())
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

    // XXX 2 -> 1
    if matches.opt_present("h") || matches.free.len() != 2 {
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

    match start_server(&socket_addr, &format!("{}\r\n", matches.free[1])) {
        Err(ref e) => error!("server failed: {}", e.to_string()),
        _ => {}
    }
}
