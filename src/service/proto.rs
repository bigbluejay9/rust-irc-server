use std::str;
use std::io;

use bytes::BytesMut;

use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::pipeline::ServerProto;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;

pub struct IRCProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for IRCProto {
    type Request = String;
    type Response = Option<String>;

    type Transport = Framed<T, Utf8CrlfCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(Utf8CrlfCodec))
    }
}

pub struct Utf8CrlfCodec;

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
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<String>, io::Error> {
        debug!("In: {:?}.", src);
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
                    Ok(s) => {
                        debug!("Input line: {:?}.", src);
                        Ok(Some(s.to_string()))
                    }
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
