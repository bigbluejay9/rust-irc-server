use futures::*;
use futures::stream::Stream;

use handlebars::{self, Handlebars};

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

use super::data;

pub fn start_stats_server(
    http: Option<SocketAddr>,
    reactor: &Handle,
    server: Arc<Mutex<data::Server>>,
) {
    if http.is_none() {
        return;
    }

    let template_engine = Arc::new(Mutex::new(handlebars::Handlebars::new()));
    assert!(
        template_engine
            .lock()
            .unwrap()
            .register_template_string(DEBUG_TEMPLATE_NAME, DEBUG_HTML_TEMPLATE)
            .is_ok()
    );
    let addr = http.unwrap();

    debug!("Starting debug HTTP server at {:?}.", addr);
    let lis = TcpListener::bind(&addr, &reactor).unwrap();
    let cloned_reactor = reactor.clone();
    let srv = lis.incoming()
        .for_each(move |(stream, addr)| {
            trace!("Accepted HTTP connection from {:?}.", addr);
            let template_engine = Arc::clone(&template_engine);
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
                        output.push_str(&render(template_engine, server));
                        debug!("Got output data: {:?}.", output);
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

static DEBUG_TEMPLATE_NAME: &'static str = "debug_html_template";
static DEBUG_HTML_TEMPLATE: &'static str = "
<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <title>IRC Server State</title>
</head>
<body>
<h1>IRC Server State</h1>
<h2>Server</h2>
<pre>
{{server}}
</pre>
<table>
  <tr>
    <th>Known Nicks</th>
  </tr>
  {{#each nick_to_connections}}
  <tr>
    <td><a href=\"#{{this}}\">{{@key}}</a></td>
  </tr>
  {{/each}}
  </tr>
</table>
<table>
  <tr>
    <th>Socket Pair</th>
    <th>Connection Data</th>
  </tr>
{{#each connections}}
  <div id=\"{{this.0}}\">
  <tr>
    <td>{{@key}}</td>
    <td>
    <pre>
        {{this.1}}
    </pre>
    </td>
  </tr>
  </div>
{{/each}}
</table>

</body>
</html>";

fn render(template_engine: Arc<Mutex<Handlebars>>, server: Arc<Mutex<data::Server>>) -> String {
    let maybe_serialized = serialize(server);
    if let Err(e) = maybe_serialized {
        return format!("Cannot serialize server data for rendering: {}", e);
    }
    let serialized = maybe_serialized.unwrap();
    trace!("About to render: {:?}.", serialized);
    match template_engine.lock().unwrap().render(
        DEBUG_TEMPLATE_NAME,
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
    // Should be serialized form.
    server: String,

    // Nick -> HTML Element ID.
    nick_to_connections: HashMap<String, String>,

    // SocketPair -> (HTML Element ID, Client). There may be some clients without a Nick.
    connections: HashMap<String, (String, String)>,
}


fn serialize(server: Arc<Mutex<data::Server>>) -> Result<DebugOutputData, String> {
    let server_serialized;
    let mut heading_number = 0;
    let mut addr_to_heading = HashMap::new();
    let mut connections_cloned: Vec<Arc<Mutex<data::Client>>> = Vec::new();
    let mut nick_to_connections_serialized = HashMap::new();
    {
        let server = server.lock().unwrap();
        server_serialized = serde_yaml::to_string(server.deref()).map_err(
            |e| e.to_string(),
        )?;

        for (n, addr) in server.nick_to_client.iter() {
            nick_to_connections_serialized.insert(n.clone(), heading_number.to_string());
            addr_to_heading.insert(addr.to_string(), heading_number);
            heading_number += 1;
        }

        for c in server.connections.values() {
            connections_cloned.push(Arc::clone(&c));
        }
    }

    let mut connections_serialized = HashMap::new();
    {
        for c in connections_cloned {
            let client = c.lock().unwrap();
            let heading_number = addr_to_heading.get(&client.socket.to_string());
            connections_serialized.insert(client.socket.to_string(), (
                match heading_number {
                    Some(s) => s.to_string(),
                    None => "XXX".to_string(),
                },
                serde_yaml::to_string(
                    client.deref(),
                ).map_err(|e| e.to_string())?,
            ));
        }
    }

    Ok(DebugOutputData {
        server: server_serialized,
        nick_to_connections: nick_to_connections_serialized,
        connections: connections_serialized,
    })
}
/*
impl Serialize for DebugOutputData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("DebugOutputData", 2)?;
        s.serialize_field(
            "server",
            &serde_yaml::to_string(self.server.lock().unwrap().deref())
                .unwrap(),
        )?;
        s.serialize_field("connections", &self.connections)?;
        s.end()
    }
}

impl Serialize for DebugConnectionsData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_map(Some(self.connections.len()))?;

        for (k, v) in self.connections.iter() {
            s.serialize_key(k)?;
            s.serialize_value(v.lock().unwrap().deref())?;
        }

        s.end()
    }
}*/
