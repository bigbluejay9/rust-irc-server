extern crate env_logger;
extern crate getopts;
extern crate log;

extern crate irc_server;

use std::net::SocketAddr;

use irc_server::service;

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

fn build_socketaddr(port: u32) -> SocketAddr {
    let addr = format!("0.0.0.0:{}", port);
    addr.parse::<SocketAddr>().unwrap()
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print help menu");
    opts.optflagopt(
        "s",
        "http_server_port",
        "Optional debugging HTTP server port.",
        "8888",
    );
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

    service::start(
        build_socketaddr(matches.free[0].parse().unwrap()),
        matches.opt_str("s").map_or(None, |p| {
            Some(build_socketaddr(p.parse().unwrap()))
        }),
    );
}
