use std::sync::{Arc, Mutex};

use super::data;
use super::messages;

pub fn process_message(
    client: Arc<Mutex<data::Client>>,
    server_prefix: Option<String>,
    req: messages::Message,
) -> Vec<messages::Message> {
    trace!(
        "Processing request [{:?}].\nClient state: {:?}.\nServer prefix: {:?}.",
        req,
        client,
        server_prefix
    );

    match req.command {
        messages::Command::Req(r) => {
            match r {
                messages::Request::NICK { nickname: nick } => {
                    // TODO(lazau): Validate nick based on
                    // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                    let mut client = client.lock().unwrap();
                    if client
                        .server
                        .lock()
                        .unwrap()
                        .replace_nick(&client, nick.clone())
                        .err() == Some(data::ServerError::NickInUse)
                    {
                        return vec![
                            messages::Message {
                                prefix: server_prefix,
                                command: messages::Command::Resp(
                                    messages::Response::ERR_NICKNAMEINUSE
                                ),
                            },
                        ];

                    }
                    client.nick = Some(nick);

                    maybe_welcome_sequence(&client, server_prefix)
                }

                messages::Request::USER {
                    username,
                    mode,
                    unused,
                    realname,
                } => {
                    let mut client = client.lock().unwrap();
                    client.user = Some(data::User {
                        username: username,
                        realname: realname,
                    });

                    maybe_welcome_sequence(&client, server_prefix)
                }

                u @ _ => {
                    error!("Response to {:?} not yet implemented.", u);
                    Vec::new()
                }
            }
        }
        r @ _ => {
            error!("{:?} isn't a client request. Dropping", r);
            Vec::new()
        }
    }
}

// Returns WELCOME sequence if client has successfully registered.
fn maybe_welcome_sequence(
    client: &data::Client,
    server_prefix: Option<String>,
) -> Vec<messages::Message> {
    if client.user.is_none() || client.nick.is_none() {
        return Vec::new();
    }

    vec![
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_WELCOME {
                message: Some(
                    "Welcome to the <networkname> Network, <nick>[!<user>@<host>]".to_string(),
                ),
            }),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_YOURHOST),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_CREATED),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_MYINFO),
        },
    ]
}
