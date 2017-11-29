use serde;
use serde::ser::{self, Serialize, Serializer, SerializeSeq};

use std;
use std::str;

use super::JoinChannels;

pub fn to_string<T>(value: &T) -> std::result::Result<String, Error>
where
    T: Serialize,
{
    let mut serializer = IRCSerializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn message_prefix_serializer<S>(t: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    match t {
        &Some(ref prefix) => serializer.serialize_str(&format!(":{} ", prefix)),
        // TODO(lazau): Figure out how to return S::Ok.
        // For not rely on the fact that serializing unit produces nothing.
        &None => serializer.serialize_unit(),
    }
}

pub fn join_serializer<S: ser::Serializer>(
    t: &super::Request,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    unimplemented!()
}

/*impl Serialize for JoinChannels {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            &JoinChannels::Channels(ref c) => serializer.serialize_str(&c.join(",")),
            &JoinChannels::KeyedChannels(ref c) => {
                let (chans, keys): (Vec<String>, Vec<String>) = c.iter().cloned().unzip();
                let mut output = chans.join(",");
                output.push_str(" :");
                output.push_str(&keys.join(","));
                serializer.serialize_str(&output)
            }
            &JoinChannels::PartAll => serializer.serialize_str("0"),
        }
    }
}*/

#[derive(Debug)]
pub struct Error {}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        unimplemented!()
    }

    fn cause(&self) -> Option<&std::error::Error> {
        unimplemented!()
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        unimplemented!()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        unimplemented!()
    }
}

#[derive(Default, Debug)]
pub struct IRCSerializer {
    pub output: String,
    last_colon: Option<usize>,
}

impl<'a> ser::Serializer for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("{}", v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output += str::from_utf8(v).unwrap();
        Ok(())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // Don't actually write anything.
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        println!(
            "name; {} variant_index: {} variatn: {}",
            name,
            variant_index,
            variant
        );
        unimplemented!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output += &format!("{}", variant);
        Ok(self)
    }
}


impl<'a> ser::SerializeSeq for &'a mut IRCSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with(':') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut IRCSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let initial_len = self.output.as_bytes().len();
        self.output += " :";
        let mut new_colon_location = self.output.as_bytes().len() - 1;
        let after_space_len = self.output.as_bytes().len();
        value.serialize(&mut **self)?;
        // If value produced a string, remove the last colon, otherwise trim the " :" string added.
        if after_space_len != self.output.as_bytes().len() {
            if let Some(last_colon_pos) = self.last_colon {
                new_colon_location -= 1;
                unsafe {
                    self.output.as_mut_vec().remove(last_colon_pos);
                }
            }
            self.last_colon = Some(new_colon_location);
        } else {
            // Trim the added space.
            unsafe {
                self.output.as_mut_vec().truncate(initial_len);
            }
        }
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::to_string;
    use super::super::{Message, Command, Request, Response, JoinChannels};

    macro_rules! verify_serialize{
        ($serialized:expr, $message:expr) => {
            assert_eq!($serialized.to_string(), to_string(&$message).unwrap());
        }
    }

    #[test]
    fn test_serialize() {
        verify_serialize!(
            "JOIN :0",
            Message {
                prefix: None,
                command: Command::Req(Request::JOIN { channels: JoinChannels::PartAll }),
            }
        );

        verify_serialize!(
            "JOIN :#channel1,channel2",
            Message {
                prefix: None,
                command: Command::Req(Request::JOIN {
                    channels: JoinChannels::Channels(
                        vec!["#channel1".to_string(), "channel2".to_string()],
                    ),
                }),
            }
        );

        verify_serialize!(
            "JOIN #channel1,channel2 :key1,secret!",
            Message {
                prefix: None,
                command: Command::Req(Request::JOIN {
                    channels: JoinChannels::KeyedChannels(vec![
                        ("#channel1".to_string(), "key1".to_string()),
                        ("channel2".to_string(), "secret!".to_string()),
                    ]),
                }),
            }
        );

        verify_serialize!(
            "NICK :lazau",
            Message {
                prefix: None,
                command: Command::Req(Request::NICK { nickname: "lazau".to_string() }),
            }
        );

        verify_serialize!(
            "QUIT",
            Message {
                prefix: None,
                command: Command::Req(Request::QUIT { message: None }),
            }
        );

        verify_serialize!(
            "QUIT :quit_message",
            Message {
                prefix: None,
                command: Command::Req(Request::QUIT { message: Some("quit_message".to_string()) }),
            }
        );

        verify_serialize!(
            "PART :channel_a,channel_b,channel_c",
            Message {
                prefix: None,
                command: Command::Req(Request::PART {
                    channels: vec![
                        "channel_a".to_string(),
                        "channel_b".to_string(),
                        "channel_c".to_string(),
                    ],
                    message: None,
                }),
            }
        );

        verify_serialize!(
            ":irc.mozilla.org PART channel_11 :A parting message",
            Message {
                prefix: Some("irc.mozilla.org".to_string()),
                command: Command::Req(Request::PART {
                    channels: vec!["channel_11".to_string()],
                    message: Some("A parting message".to_string()),
                }),
            }
        );

        verify_serialize!(
            ":WiZ CONNECT eff.org 6667 :csd.bu.edu",
            Message {
                prefix: Some("WiZ".to_string()),
                command: Command::Req(Request::CONNECT {
                    target: "eff.org".to_string(),
                    port: Some(6667),
                    remote: Some("csd.bu.edu".to_string()),
                }),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 461",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::ERR_NEEDMOREPARAMS),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 001",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::RPL_WELCOME { message: None }),
            }
        );

        verify_serialize!(
            ":irc.freenode.net 001 :Welcome to the network, friend!",
            Message {
                prefix: Some("irc.freenode.net".to_string()),
                command: Command::Resp(Response::RPL_WELCOME {
                    message: Some("Welcome to the network, friend!".to_string()),
                }),
            }
        );
    }
}
