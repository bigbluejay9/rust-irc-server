use std::iter;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::super::server::{Configuration, ServerError};
use super::{Client, User};
use super::super::Broadcast;
use super::super::messages::Message;
use super::super::messages::commands::Command;
use super::super::messages::commands::requests as Requests;
use super::super::messages::commands::responses as Responses;
use super::super::templates;

macro_rules! error_resp {
    ($err:expr) => {
        return vec![
            Message {
                prefix: None,
                command: $err,
            },
        ];
    };
}

pub fn process_message(
    configuration: Arc<Configuration>,
    client: Arc<Mutex<Client>>,
    req: Message,
) -> Vec<Message> {
    trace!("Client state: {:?}.", client);

    macro_rules! verify_registered {
        ($client:expr) => {
            if !$client.registered() {
                error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
            }
        }
    }

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
                .err() == Some(ServerError::NickInUse)
            {
                error_resp!(Command::ERR_NICKNAMEINUSE(
                    Responses::NICKNAMEINUSE { nick: nick.clone() },
                ));

            }
            client.nick = Some(nick);

            maybe_welcome_sequence(&configuration, &client)
        }

        Command::USER(Requests::User {
                          username,
                          mode: _mode,
                          unused: _unused,
                          realname,
                      }) => {
            let mut client = client.lock().unwrap();
            client.user = Some(User {
                username: username,
                realname: realname,
            });

            maybe_welcome_sequence(&configuration, &client)
        }

        Command::JOIN(Requests::Join { join: jt }) => {
            let mut client = client.lock().unwrap();
            verify_registered!(client);
            match jt {
                Requests::JoinChannels::PartAll => {
                    client.part_all();
                    Vec::new()
                }
                Requests::JoinChannels::Channels(r) => {
                    client.join(r.iter().zip(iter::repeat(None)).collect())
                }
                Requests::JoinChannels::KeyedChannels(r) => {
                    let (chans, keys): (Vec<String>, Vec<String>) = r.into_iter().unzip();
                    client.join(chans.iter().zip(keys.iter().map(|k| Some(k))).collect())
                }
            }
        }

        Command::PART(Requests::Part { channels, message }) => {
            let mut client = client.lock().unwrap();
            verify_registered!(client);
            client.part(&channels, &message)
        }

        Command::MODE(Requests::Mode {
                          target,
                          mode_string,
                          mode_args,
                      }) => {
            let mut client = client.lock().unwrap();
            verify_registered!(client);
            if &target != client.nick.as_ref().unwrap() {
                error_resp!(Command::ERR_USERSDONTMATCH(Responses::UsersDontMatch {
                    nick: client.nick.clone().unwrap(),
                }));
            }
            if mode_string.is_none() {
                unimplemented!();
            }
            let mode = mode_string.unwrap();
            if mode.len() < 2 {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                    nick: client.nick.clone().unwrap(),
                }));
            }
            let set = if mode.starts_with("+") {
                super::SetMode::Add
            } else if mode.starts_with("-") {
                super::SetMode::Remove
            } else {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                    nick: client.nick.clone().unwrap(),
                }));
            };

            let m = mode.chars()
                .nth(1)
                .unwrap()
                .to_string()
                .parse::<super::UserMode>()
                .map_err(|_| {
                    error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                        nick: client.nick.clone().unwrap(),
                    }))
                })
                .unwrap();
            client.set_mode(&set, &vec![m]);
            Vec::new()
        }

        Command::PING(Requests::Ping { originator, target }) => {
            if target.is_some() && target.unwrap() != configuration.hostname {
                unimplemented!();
            }
            vec![
                Message {
                    prefix: None,
                    command: Command::PONG(Requests::Pong {
                        originator: configuration.hostname.clone(),
                        target: None,
                    }),
                },
            ]
        }

        u @ _ => {
            error!("Response to {:?} not yet implemented.", u);
            Vec::new()
        }
    }
}

pub fn process_broadcast(
    configuration: Arc<Configuration>,
    client: Arc<Mutex<Client>>,
    b: Arc<Broadcast>,
) -> Vec<Message> {
    trace!("Client state: {:?}.", client);
    match b.deref() {
        &Broadcast::Join(ref user, ref channel) => {
            vec![
                Message {
                    prefix: Some(format!("{}", user)),
                    command: Command::JOIN(Requests::Join {
                        join: Requests::JoinChannels::Channels(vec![channel.clone()]),
                    }),
                },
            ]
        }
        &Broadcast::Part => unimplemented!(),
        &Broadcast::PrivateMessage => unimplemented!(),
        u @ _ => {
            error!("Broadcast message {:?} not yet implemented.", u);
            Vec::new()
        }
    }
}


// Returns WELCOME sequence if client has successfully registered.
fn maybe_welcome_sequence(configuration: &Configuration, client: &Client) -> Vec<Message> {
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
            prefix: None,
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
            prefix: None,
            command: Command::RPL_YOURHOST(Responses::YourHost {
                nick: client.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_YOURHOST_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_CREATED(Responses::Created {
                nick: client.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_CREATED_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_MYINFO(Responses::MyInfo::default()),
        },
    ]
}
