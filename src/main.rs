extern crate env_logger;
extern crate getopts;
extern crate log;
extern crate serde_yaml;
extern crate irc_server;

use std;
use irc_server::{configuration, service};

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} port", prog);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Print help menu.");
    opts.optflagopt(
        "g",
        "generate_default_config",
        "Generate default configuration file",
        "config.yaml",
    );
    opts.optflagopt("c", "config_file", "Configuration filename.", "config.yaml");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string());
        }
    };

    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        return;
    }

    if matches.opt_present("g") {
        let file = std::fs::File::create(matches.opt_str("g").unwrap_or("config.yaml")).unwrap();
        serde_yaml::to_writer(file, &configuration::Configuration::default()).unwrap();
        return;
    }

    let config = if matches.opt_present("c") {
        serde_yaml::from_reader(&std::fs::File::open(matches.opt_str("c").unwrap_or("config.yaml")).unwrap()).unwrap() } else {
                configuration::Configuration::default()
            }

            service::start(std::sync::Arc::new(&config));
    }
