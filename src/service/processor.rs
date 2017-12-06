use std::iter;
use std::sync::{Arc, Mutex};

use super::data;
use super::messages::Message;
use super::messages::commands::Command;
use super::messages::commands::requests as Requests;
use super::messages::commands::responses as Responses;
use super::templates;

macro_rules! error_resp {
    ($prefix:ident, $err:expr) => {
        return vec![
            Message {
                prefix: $prefix,
                command: $err,
            },
        ];
    };
}

macro_rules! verify_registered {
    ($prefix:ident, $client:expr) => {
        if !$client.registered() {
            error_resp!($prefix, Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
        }
    }
}

pub fn process_message(
    configuration: Arc<data::Configuration>,
    client: Arc<Mutex<data::Client>>,
    server_prefix: Option<String>,
    req: Message,
) -> Vec<Message> {
    trace!(
        "Processing request [{:?}].\nClient state: {:?}.\nServer prefix: {:?}.",
        req,
        client,
        server_prefix
    );

    match req.command {
        Command::NICK(Requests::Nick { nickname: nick }) => {
            // TODO(lazau): Validate nick based on
            // https://tools.ietf.org/html/rfc2812#section-2.3.1.
            let mut client = client.lock().unwrap();
            if client
                .server
                .lock()
                .unwrap()
                .replace_nick(&client, nick.clone())
                .err() == Some(data::Error::NickInUse)
            {
                error_resp!(
                    server_prefix,
                    Command::ERR_NICKNAMEINUSE(Responses::NICKNAMEINUSE { nick: nick.clone() })
                );

            }
            client.nick = Some(nick);

            maybe_welcome_sequence(&configuration, &client, server_prefix)
        }

        Command::USER(Requests::User {
                          username,
                          mode: _mode,
                          unused: _unused,
                          realname,
                      }) => {
            let mut client = client.lock().unwrap();
            client.user = Some(data::User {
                username: username,
                realname: realname,
            });

            maybe_welcome_sequence(&configuration, &client, server_prefix)
        }

        Command::JOIN(Requests::Join { join: jt }) => {
            //let mut client = client.lock().unwrap();
            //let mut server = client.server.lock().unwrap();
            let chan_data;
            match jt {
                Requests::JoinChannels::PartAll => {
                    client.lock().unwrap().part_all();
                    return Vec::new();
                }
                Requests::JoinChannels::Channels(r) => {
                    chan_data = client.lock().unwrap().join(
                        r.iter()
                            .zip(iter::repeat(None))
                            .collect(),
                    );
                }
                Requests::JoinChannels::KeyedChannels(r) => {
                    let mut chans = Vec::new();
                    let mut keys: Vec<Option<&String>> = Vec::new();
                    for &(c, k) in r.iter() {
                        chans.push(c);
                        keys.push(Some(k.clone()));
                    }
                    chan_data = client.lock().unwrap().join(
                        chans.iter().zip(keys).collect(),
                    );
                }
            };
            unimplemented!()
            /*if keys.len() > 0 && channels.len() != keys.len() {
                error_resp!(
                    server_prefix,
                    Command::ERR_NEEDMOREPARAMS(
                        Responses::NEEDMOREPARAMS { command: "JOIN".to_string() },
                    )
                );
            }

            let mut channel_info = Vec::with_capacity(channels.len());
            let client = client.lock().unwrap();
            verify_registered!(server_prefix, client);

            if channels.len() == 1 && channels[0] == "0" {
                unimplemented!()
            }

            let mut server = client.server.lock().unwrap();
            for (i, c) in channels.iter().enumerate() {
                channel_info.push((c, server.join_channel(&client, c, keys.get(i))));
            }*/

            /*channel_info
                        .into_iter()
                        .flat_map(|(channel, (topic, nicks))| {
                            vec![
                                Message {
                                    prefix: server_prefix.clone(),
                                    command: Command::JOIN(Requests::Join {
                                        channels: vec![channel.clone()],
                                        keys: Vec::new(),
                                    }),
                                },
                                Message {
                                }
                            ]
                        })
                        .collect()*/
        }

        u @ _ => {
            error!("Response to {:?} not yet implemented.", u);
            Vec::new()
        }
    }
}

// Returns WELCOME sequence if client has successfully registered.
fn maybe_welcome_sequence(
    configuration: &data::Configuration,
    client: &data::Client,
    server_prefix: Option<String>,
) -> Vec<Message> {
    if client.user.is_none() || client.nick.is_none() {
        return Vec::new();
    }

    #[derive(Serialize)]
    struct WelcomeData<'a> {
        network_name: &'a str,
        nick: &'a str,
    }

    vec![
        Message {
            prefix: server_prefix.clone(),
            command: Command::RPL_WELCOME(Responses::Welcome {
                nick: client.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(
                        templates::RPL_WELCOME_TEMPLATE_NAME,
                        &WelcomeData {
                            network_name: &configuration.network_name,
                            nick: client.nick.as_ref().unwrap(),
                        },
                    )
                    .unwrap(),
            }),
        },
        Message {
            prefix: server_prefix.clone(),
            command: Command::RPL_YOURHOST(Responses::YourHost {
                nick: client.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_YOURHOST_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: server_prefix.clone(),
            command: Command::RPL_CREATED(Responses::Created {
                nick: client.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_CREATED_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: server_prefix.clone(),
            command: Command::RPL_MYINFO(Responses::MyInfo::default()),
        },
    ]
}
