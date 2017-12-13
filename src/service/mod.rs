mod codec;
mod messages;
mod server;
mod shared_state;
mod stats;
mod connection;
mod channel;
mod user;

use chrono;
use futures::Future;
use futures::future::Executor;
use futures::stream::Stream;
use futures_cpupool::CpuPool;
use hostname;
use std::sync::{Arc, Mutex};
use super::configuration;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

pub fn start(configuration: Arc<configuration::Configuration>) {
    debug!("Using configuration: {:#?}.", configuration);

    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();

    let shared_state = Arc::new(shared_state::SharedState::new(
        chrono::offset::Utc::now(),
        hostname::get_hostname().unwrap(),
        configuration,
    ));
    let srv = Arc::new(Mutex::new(server::Server::new(Arc::clone(&shared_state))));

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

    stats::start_stats_server(&handle, Arc::clone(&shared_state), Arc::clone(&srv));

    let lis = insecure_lis.incoming().for_each(move |(stream, _)| {
        connection::Connection::handle_new_connection(
            stream,
            Arc::clone(&shared_state),
            Arc::clone(&srv),
        )
    });

    match event_loop.run(lis) {
        Err(e) => error!("Listener failure: {:?}.", e),
        _ => {}
    };
}
