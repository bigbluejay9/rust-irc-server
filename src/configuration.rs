use std;

static DEFAULT_VERSION: &'static str = "1.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub version: String,
    pub network_name: String,

    pub insecure_listen_address: Option<std::net::SocketAddr>,
    pub secure_listen_address: Option<std::net::SocketAddr>,
    pub debug_http_listen_address: Option<std::net::SocketAddr>,

    pub channel_message_queue_length: usize,
    pub connection_message_queue_length: usize,
    pub user_message_queue_length: usize,
    pub server_message_queue_length: usize,
}

impl std::default::Default for Configuration {
    fn default() -> Self {
        Self {
            version: DEFAULT_VERSION.to_string(),
            network_name: "IRC Network".to_string(),

            insecure_listen_address: Some("0.0.0.0:6667".parse().unwrap()),
            secure_listen_address: Some("0.0.0.0:6697".parse().unwrap()),
            debug_http_listen_address: Some("0.0.0.0:8000".parse().unwrap()),

            channel_message_queue_length: 25,
            connection_message_queue_length: 10,
            user_message_queue_length: 10,
            server_message_queue_length: 50,
        }
    }
}
