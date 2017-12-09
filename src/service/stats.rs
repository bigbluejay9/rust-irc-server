use futures::*;
use futures::stream::Stream;

use serde_yaml;

use std::collections::HashMap;
use std::ops::Deref;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio_core::net::TcpListener;
use tokio_core::reactor::Handle;
use tokio_io::io::{lines, write_all};
use tokio_io::AsyncRead;

use super::connection;
use super::server;
use super::templates;

pub fn start_stats_server(
    http: Option<SocketAddr>,
    reactor: &Handle,
    configuration: Arc<server::Configuration>,
    server: Arc<Mutex<server::Server>>,
) {
    if http.is_none() {
        return;
    }

    let addr = http.unwrap();

    debug!("Starting debug HTTP server at {:?}.", addr);
    let lis = TcpListener::bind(&addr, &reactor).unwrap();
    let cloned_reactor = reactor.clone();
    let srv = lis.incoming()
        .for_each(move |(stream, addr)| {
            trace!("Accepted HTTP connection from {:?}.", addr);
            let configuration = Arc::clone(&configuration);
            let server = Arc::clone(&server);
            // TODO(lazau): For now we don't offload this to a worker thread. May want to.
            cloned_reactor.spawn_fn(move || {
                let (reader, writer) = stream.split();
                let buffered_reader = BufReader::new(reader);
                lines(buffered_reader)
                    .take_while(|line| future::ok(line.len() > 0))
                    .collect()
                    .and_then(move |line| {
                        trace!("Received HTTP request: {:?}.", line);
                        let mut output = DEBUG_HTTP_RESP.to_string();
                        output.push_str(&render(configuration, server));
                        trace!("Got output data: {:?}.", output);
                        write_all(writer, output.into_bytes())
                    })
                    .into_future()
                    .then(|_| future::ok::<(), ()>(()))
            });
            Ok(())
        })
        .map_err(|_| ());
    reactor.spawn(srv);
}

static DEBUG_HTTP_RESP: &'static str = "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n";

fn render(configuration: Arc<server::Configuration>, server: Arc<Mutex<server::Server>>) -> String {
    let maybe_serialized = serialize(Arc::clone(&configuration), server);
    if let Err(e) = maybe_serialized {
        return format!("Cannot serialize server data for rendering: {}", e);
    }
    let serialized = maybe_serialized.unwrap();
    trace!("About to render: {:?}.", serialized);
    match configuration.template_engine.render(
        templates::DEBUG_TEMPLATE_NAME,
        &serialized,
    ) {
        Ok(o) => o,
        Err(e) => {
            error!("Cannot render debug HTML template: {:?}.", e);
            "Failed to render debug template.".to_string()
        }
    }
}

#[derive(Debug, Serialize)]
struct DebugOutputData {
    configuration: String,

    // Nick -> HTML Element ID.
    nick_to_connections: HashMap<String, String>,

    // SocketPair -> ((ID Valid, HTML Element ID), Connection). There may be some connections without a Nick.
    connections: HashMap<String, ((bool, String), String)>,

    // Channels to Nicks.
    channels_to_nicks: HashMap<String, Vec<String>>,
}

fn serialize(
    configuration: Arc<server::Configuration>,
    server: Arc<Mutex<server::Server>>,
) -> Result<DebugOutputData, String> {
    let configuration_serialized = serde_yaml::to_string(configuration.deref()).map_err(|e| {
        e.to_string()
    })?;

    let mut heading_number = 0;
    let mut addr_to_heading = HashMap::new();
    let mut connections_cloned: Vec<Arc<Mutex<connection::Connection>>> = Vec::new();
    let mut nick_to_connections_serialized = HashMap::new();
    let mut channels_to_nicks_serialized = HashMap::new();
    {
        let server = server.lock().unwrap();
        for (n, addr) in server.nick_to_connection.iter() {
            nick_to_connections_serialized.insert(n.clone(), heading_number.to_string());
            addr_to_heading.insert(addr.to_string(), heading_number);
            heading_number += 1;
        }

        for c in server.connections.values() {
            connections_cloned.push(Arc::clone(&c));
        }

        for (name, chan) in server.channels.iter() {
            let mut nicks = Vec::new();
            for n in chan.nicks.iter() {
                nicks.push(n.clone());
            }
            channels_to_nicks_serialized.insert(name.clone(), nicks);
        }
    }

    let mut connections_serialized = HashMap::new();
    {
        for c in connections_cloned {
            let connection = c.lock().unwrap();
            let heading_number = addr_to_heading.get(&connection.socket.to_string());
            connections_serialized.insert(connection.socket.to_string(), (
                match heading_number {
                    Some(s) => (true, s.to_string()),
                    None => (false, "".to_string()),
                },
                serde_yaml::to_string(
                    connection.deref(),
                ).map_err(|e| e.to_string())?,
            ));
        }
    }

    Ok(DebugOutputData {
        configuration: configuration_serialized,
        nick_to_connections: nick_to_connections_serialized,
        connections: connections_serialized,
        channels_to_nicks: channels_to_nicks_serialized,
    })
}
