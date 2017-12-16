mod codec;
mod messages;
pub mod server;
pub mod shared_state;
pub mod connection;
pub mod channel;
pub mod user;

use chrono;
use futures::prelude::*;
use futures::stream::Stream;
use futures_cpupool::CpuPool;
use hostname;
use hyper::server::Http;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::configuration;
use super::debug;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

pub fn start(configuration: Arc<configuration::Configuration>) {
    trace!("Using configuration: {:#?}.", configuration);

    let thread_pool = CpuPool::new_num_cpus();
    let shared_state = Arc::new(shared_state::SharedState::new(
        chrono::offset::Utc::now(),
        hostname::get_hostname().unwrap(),
        &thread_pool,
        configuration,
    ));

    let srv = Arc::new(Mutex::new(server::Server::new(Arc::clone(&shared_state))));
    let connections = Arc::new(Mutex::new(HashMap::new()));
    let debug_service = Arc::new(debug::DebugService::new(
        Arc::clone(&shared_state),
        Arc::clone(&srv),
        Arc::clone(&connections),
    ));

    // Hyper owns core. That's why we gotta do this weird Core dance.
    // https://github.com/hyperium/hyper/issues/1075
    // Result<
    //     HTTP Server Enabled  - hyper::server::Http owned Core,
    //     HTTP Server Disabled - This module owned Core
    // >
    let core = match shared_state.configuration.debug_http_listen_address {
        Some(ref addr) => {
            debug!("Starting debug HTTP server at {:?}.", addr);
            Ok(
                Http::new()
                    .bind(addr, move || Ok(Arc::clone(&debug_service)))
                    .unwrap(),
            )
        }
        None => Err(Core::new().unwrap()),
    };
    let handle = match core {
        Ok(ref d) => d.handle(),
        Err(ref e) => e.handle(),
    };

    // TODO(lazau): Add secure listener.
    let insecure_lis = TcpListener::bind(
        shared_state
            .configuration
            .insecure_listen_address
            .as_ref()
            .unwrap(),
        &handle,
    ).unwrap();
    debug!(
        "Starting IRC server at {:?}.",
        insecure_lis.local_addr().unwrap()
    );

    let lis_handle = handle.clone();
    let lis = insecure_lis
        .incoming()
        .for_each(move |(stream, _)| {
            let shared_state = Arc::clone(&shared_state);
            let srv = Arc::clone(&srv);
            let connections = Arc::clone(&connections);
            lis_handle.spawn(thread_pool.spawn(
                connection::Connection::handle_new_connection(
                    stream,
                    shared_state,
                    srv,
                    connections,
                ),
            ));
            Ok(())
        })
        .map_err(|_| ());

    match core {
        Ok(dbg_srv) => {
            handle.spawn(lis);
            dbg_srv.run().unwrap();
        }
        Err(mut core) => {
            core.run(lis).unwrap();
        }
    };
}
