mod messages;

use hostname;

use std::str;
use std::collections::HashSet;
use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use bytes::BytesMut;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_io::AsyncRead;
use tokio_io::codec::{Encoder, Decoder};

pub fn start(addr: SocketAddr) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&addr, &handle).unwrap();

    let server = Arc::new(Mutex::new(Server {
        hostname: hostname::get_hostname().expect("unable to get hostname"),
        nicknames: HashSet::new(),
        clients: Vec::new(),
    }));

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = server.clone();
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            debug!("Accepting connection: {:?}, {:?}.", stream, addr);

            let client = Arc::new(Mutex::new(ConnectionData {
                remote_addr: addr,
                nick: None,
            }));

            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow clients.
            let (tx, rx) = mpsc::channel(5);
            server.lock().unwrap().clients.push(tx);

            let (sink, stream) = stream.framed(Utf8CrlfCodec).split();
            stream
                .select(rx.map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "can never happen")
                }))
                .then(move |event| {
                    debug!("Connection event: {:?}.", event);
                    if event.is_err() {
                        let err = event.err().unwrap();
                        warn!("Upstream error: {:?}.", err);
                        return future::err(err);
                    }

                    // TODO(lazau): Wrap result in a Result type, log issues.
                    let result = match event.unwrap() {
                        ClientEvent::Socket(s) => {
                            process_message(Arc::clone(&server), Arc::clone(&client), s)
                        }
                        // TODO(lazau): implement.
                        ClientEvent::Broadcast(b) => Some(
                            format!("got broadcast message {:?}.", b),
                        ),
                    };
                    future::ok(result)
                })
                .forward(sink)
                .then(move |e| {
                    if let Err(e) = e {
                        warn!("Connection error: {:?}.", e);
                    }

                    // TODO(lazau): Perform connection cleanup here.
                    /*let client = Arc::clone(&client).lock().unwrap();
                    if let Some(nick) = client.nick {
                        server.lock().unwrap().nicknames.remove(&nick);
                    }*/

                    future::ok(())
                })
        }));

        Ok(())
    });

    match event_loop.run(srv) {
        Err(e) => error!("Server failure: {:?}.", e),
        _ => {}
    };
}

#[derive(Debug)]
struct Server {
    hostname: String,
    nicknames: HashSet<String>,
    // Receiver may be gone, must check for send errors!
    clients: Vec<mpsc::Sender<ClientEvent>>,
}

// A union of socket events and server-wide events.
#[derive(Debug)]
enum ClientEvent {
    Socket(String),
    Broadcast(String),
}

#[derive(Debug)]
struct ConnectionData {
    remote_addr: SocketAddr,
    nick: Option<String>,
}

#[derive(Debug)]
struct Utf8CrlfCodec;

impl Encoder for Utf8CrlfCodec {
    type Item = Option<String>;
    type Error = io::Error;
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), io::Error> {
        match item {
            Some(ref s) => {
                dst.extend(s.as_bytes());
                dst.extend(b"\r\n");
            }
            None => {}
        }
        Ok(())
    }
}

impl Decoder for Utf8CrlfCodec {
    type Item = ClientEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<ClientEvent>, io::Error> {
        let mut crlf_pos: Option<usize> = None;
        for (pos, &c) in src.iter().enumerate() {
            if pos > 1 && c == b'\n' && src[pos - 1] == b'\r' {
                crlf_pos = Some(pos);
                break;
            }
        }

        match crlf_pos {
            Some(pos) => {
                let line = &src.split_to(pos + 1)[0..(pos - 1)];
                match str::from_utf8(&line) {
                    Ok(s) => Ok(Some(ClientEvent::Socket(s.to_string()))),
                    // TODO(lazau): Maybe optionally support ISO-8859-1?
                    Err(ref e) => {
                        debug!("Error: {:?}.", e.to_string());
                        Err(io::Error::new(
                            io::ErrorKind::Other,
                            "not valid utf-8 string",
                        ))
                    }
                }
            }
            None => Ok(None),
        }
    }
}

fn process_message(
    server: Arc<Mutex<Server>>,
    client: Arc<Mutex<ConnectionData>>,
    req: String,
) -> Option<String> {
    debug!(
        "Processing request [{:?}].\nClient state: {:?}.\nServer state: {:?}.",
        req,
        client,
        server
    );

    let message;
    match req.parse::<messages::Message>() {
        Ok(m) => message = m,
        Err(ref e) => {
            warn!("Bad client message [{:?}]: {:?}.", req, e);
            return None;
        }
    };

    match message.command {
        messages::Command::NICK => Some(req),
        messages::Command::USER => Some(req),
        u @ _ => {
            error!("Response to {:?} not yet implemented.", u);
            None
        }
    }
}
