use futures::future;
use hyper;
use hyper::server::{Request, Response, Service};
use serde_yaml;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use super::service::server::Server;
use super::service::connection::{Connection, SocketPair};
use super::service::shared_state::SharedState;
use super::configuration::Configuration;
use super::templates;

#[derive(Debug)]
pub struct DebugService {
    shared_state: Arc<SharedState>,
    server: Arc<Mutex<Server>>,
    connections: Arc<Mutex<HashMap<SocketPair, Arc<Mutex<Connection>>>>>,
}

#[derive(Debug, Serialize)]
struct DebugOutputData {
    configuration: (bool, String),

    // SocketPair -> (Registered, (Nick, Nick HTML Element ID)).
    // There may be some connections without a Nick.
    connections: HashMap<String, (bool, (String, String))>,

    // Channel Name -> Vec<(Nick, Nick HTML Element ID)>.
    channels_to_nicks: HashMap<String, Vec<(String, String)>>,

    // Nick -> (User Serialized, Nick HTML Element ID, Channel, Channel HTML Element ID).
    user_to_channels: HashMap<String, (String, String, Vec<(String, String)>)>,
}

impl DebugService {
    pub fn new(
        shared_state: Arc<SharedState>,
        server: Arc<Mutex<Server>>,
        connections: Arc<Mutex<HashMap<SocketPair, Arc<Mutex<Connection>>>>>,
    ) -> Self {
        Self {
            shared_state,
            server,
            connections,
        }
    }

    fn render(&self) -> String {
        let maybe_serialized = self.serialize();
        if let Err(e) = maybe_serialized {
            return format!("Cannot serialize server data for rendering: {}", e);
        }
        let serialized = maybe_serialized.unwrap();
        trace!("About to render: {:?}.", serialized);
        match self.shared_state.template_engine.0.render(
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

    fn serialize(&self) -> Result<DebugOutputData, String> {
        let configuration = (
            self.shared_state.configuration.deref() == &Configuration::default(),
            serde_yaml::to_string(self.shared_state.configuration.deref())
                .map_err(|e| e.to_string())?,
        );

        let mut nick_id_counter = 0;
        let mut channel_id_counter = 0;
        let mut nick_to_id = HashMap::new();
        let mut channel_to_id = HashMap::new();
        let mut channels_to_nicks = HashMap::new();
        let mut connections_output = HashMap::new();
        let mut user_to_channels = HashMap::new();
        {
            let server = self.server.lock().unwrap();
            // Assign HTML element IDs to every nick.
            for user in server.users() {
                nick_to_id.insert(
                    user.nick().clone(),
                    format!("nick_{}", nick_id_counter.to_string()),
                );
                nick_id_counter += 1;
            }

            // Assign HTML element IDs to every channel.
            // Build channels_to_nicks.
            // Build channel -> Vec<Nick>.
            for (ident, chan) in server.channels() {
                channel_to_id.insert(
                    ident.name().clone(),
                    format!("channel_{}", channel_id_counter.to_string()),
                );
                channel_id_counter += 1;

                let mut nicks = Vec::new();
                for user in chan.users() {
                    nicks.push((
                        user.nick().clone(),
                        nick_to_id
                            .get(user.nick())
                            .unwrap_or(&"".to_string())
                            .clone(),
                    ));
                }
                channels_to_nicks.insert(ident.name().clone(), nicks);
            }
        }

        {
            let conns = self.connections.lock().unwrap();
            for (socket, conn) in conns.iter() {
                let conn = conn.lock().unwrap();
                if conn.registered() {
                    let u = conn.get_user();
                    let nick = u.nick();
                    connections_output.insert(socket.to_string(), (true, (
                        nick.clone(),
                        nick_to_id.get(nick).unwrap().clone(),
                    )));
                    let mut channels = Vec::new();
                    for chan in u.channels() {
                        channels.push((
                            chan.name().clone(),
                            channel_to_id
                                .get(chan.name())
                                .unwrap_or(&"".to_string())
                                .clone(),
                        ));
                    }
                    user_to_channels.insert(nick.clone(), (
                        serde_yaml::to_string(u).map_err(
                            |e| e.to_string(),
                        )?,
                        nick_to_id
                            .get(nick)
                            .unwrap_or(&"".to_string())
                            .clone(),
                        channels,
                    ));
                } else {
                    connections_output.insert(
                        socket.to_string(),
                        (false, ("".to_string(), "".to_string())),
                    );
                }
            }
        }

        Ok(DebugOutputData {
            configuration: configuration,
            connections: connections_output,
            channels_to_nicks: channels_to_nicks,
            user_to_channels: user_to_channels,
        })
    }
}

impl Service for DebugService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>; //<Item = Self::Response, Error = Self::Error>;

    fn call(&self, req: Request) -> Self::Future {
        trace!("Processing HTTP request: {:?}.", req);
        future::ok(Response::new().with_body(self.render()))
    }
}
