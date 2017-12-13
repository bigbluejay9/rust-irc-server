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

use super::server::Server;
use super::{connection, shared_state};
use super::super::{configuration, templates};

pub fn start_stats_server(
    reactor: &Handle,
    shared_state: Arc<shared_state::SharedState>,
    server: Arc<Mutex<Server>>,
) {
    return;
    //unimplemented!();
    /*    if http.is_none() {
        return;
    }

    let addr = http.unwrap();

    debug!("Starting debug HTTP server at {:?}.", addr);
    let lis = TcpListener::bind(&addr, &reactor).unwrap();
    let cloned_reactor = reactor.clone();
    let srv = lis.incoming()
        .for_each(move |(stream, addr)| {
            trace!("Accepted HTTP connection from {:?}.", addr);
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
                        output.push_str(&render(server));
                        trace!("Got output data: {:?}.", output);
                        write_all(writer, output.into_bytes())
                    })
                    .into_future()
                    .then(|_| future::ok::<(), ()>(()))
            });
            Ok(())
        })
        .map_err(|_| ());
    reactor.spawn(srv);*/
}

static DEBUG_HTTP_RESP: &'static str = "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n";

/*fn render(server: ServerTX) -> String {
    unimplemented!()
    /*let maybe_serialized = serialize(Arc::clone(&configuration), server, connections);
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
    }*/
}*/

#[derive(Debug, Serialize)]
struct DebugOutputData {
    configuration: String,

    // Nick -> HTML Element ID.
    nick_to_id: HashMap<String, String>,

    // Channel -> HTML Element ID.
    channel_to_id: HashMap<String, String>,

    // SocketPair -> ((Registered, HTML Element ID), Nick). There may be some connections without a Nick.
    connections: HashMap<String, (bool, String, String)>,

    // Channels to Users.
    channels_to_nicks: HashMap<String, Vec<String>>,

    // Nick -> User
    nicks_to_users: HashMap<String, String>,
}

/*fn serialize(
    server: ServerTX /*server: Arc<server::Server>,
    connections: Arc<Mutex<HashMap<super::SocketPair, Arc<Mutex<connection::Connection>>>>>,*/
) -> Result<DebugOutputData, String> {
    unimplemented!()
    /*let configuration_serialized = serde_yaml::to_string(configuration.deref()).map_err(|e| {
        e.to_string()
    })?;

    let mut nick_id_counter = 0;
    let mut channel_id_counter = 0;
    let mut nick_to_id = HashMap::new();
    let mut channel_to_id = HashMap::new();
    let mut channels_to_nicks = HashMap::new();
    let mut connections_output = HashMap::new();
    let mut nicks_to_users = HashMap::new();
    {
        let server = server.lock().unwrap();
        // Assign HTML element IDs to every nicknames.
        for user in server.users.keys() {
            nick_to_id.insert(
                user.nickname.clone(),
                format!("nick_{}", nick_id_counter.to_string()),
            );
            nick_id_counter += 1;
        }

        // Assign HTML element IDs to every channels.
        // Build channel -> Vec<Nick>.
        for (name, chan) in server.channels.iter() {
            channel_to_id.insert(
                name.clone(),
                format!("channel_{}", channel_id_counter.to_string()),
            );
            channel_id_counter += 1;

            let mut nicks = Vec::new();
            for user in chan.users.keys() {
                nicks.push(user.nickname.clone());
            }
            channels_to_nicks.insert(name.clone(), nicks);
        }
    }

    {
        let conns = connections.lock().unwrap();
        for (socket, conn) in conns.iter() {
            let conn = conn.lock().unwrap();
            if let Some(u) = conn.registered() {
                let nick = u.nick();
                connections_output.insert(socket.to_string(), (
                    true,
                    nick_to_id.get(nick).unwrap().clone(),
                    nick.clone(),
                ));
                nicks_to_users.insert(
                    nick.clone(),
                    serde_yaml::to_string(u).map_err(|e| e.to_string())?,
                );
            } else {
                connections_output.insert(socket.to_string(), (
                    false,
                    "".to_string(),
                    "".to_string(),
                ));
            }
        }
    }

    Ok(DebugOutputData {
        configuration: configuration_serialized,
        nick_to_id: nick_to_id,
        channel_to_id: channel_to_id,
        connections: connections_output,
        channels_to_nicks: channels_to_nicks,
        nicks_to_users: nicks_to_users,
    })*/
}*/
