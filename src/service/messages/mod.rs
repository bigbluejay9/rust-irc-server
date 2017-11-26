mod responses;
mod requests;

pub use self::responses::Response;
pub use self::requests::Request;

use std;
use std::fmt::{self, Write};
use std::str;

use serde;
use serde::ser::{self, Serialize};

#[derive(Debug)]
pub enum ParseErrorKind {
    NoCommand,
    UnrecognizedCommand,
    NeedMoreParams,
    TooManyParams,
    ParseIntError,
    Other,
}

#[derive(Debug)]
pub struct ParseError {
    desc: &'static str,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: Command,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Req(requests::Request),
    Resp(responses::Response),
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, desc: &'static str) -> ParseError {
        ParseError {
            desc: desc,
            kind: kind,
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.desc
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IRC command parse error: {}", &self.desc)
    }
}

impl serde::ser::Error for ParseError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        unimplemented!()
    }
}

impl str::FromStr for Message {
    type Err = ParseError;
    // RFC 1459 2
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 1 || s.len() > 510 {
            return Err(ParseError::new(ParseErrorKind::Other, "bad command length"));
        }

        let mut remainder: &str = &s.trim_right();
        debug!("Processing {:?}", remainder);

        let mut prefix: Option<String> = None;
        if remainder.starts_with(':') {
            match remainder.find(' ') {
                Some(idx) => {
                    prefix = Some(remainder[0..idx].to_string());
                    remainder = &remainder[idx + 1..];
                }
                None => {
                    return Err(ParseError::new(
                        ParseErrorKind::NoCommand,
                        "only prefix given",
                    ));
                }
            }
        }
        if remainder.len() < 1 {
            return Err(ParseError::new(
                ParseErrorKind::NoCommand,
                "no command specified",
            ));
        }

        let command: Command = Command::Req(remainder.parse::<Request>()?);
        debug!(
            "Parsed {} to prefix: [{:?}]; command: [{:?}].",
            s,
            prefix,
            command,
        );

        Ok(Message {
            prefix: prefix,
            command: command,
        })
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref prefix) = self.prefix {
            assert!(prefix.find(' ').is_none());
            write!(f, ":{} ", prefix)?;
        }
        match self.command {
            Command::Req(ref r) => write!(f, "{}", r)?,
            Command::Resp(ref r) => write!(f, "{}", r)?,
        };
        Ok(())
    }
}

pub fn serialize_params(p: &Vec<String>) -> Result<String, std::fmt::Error> {
    if p.len() == 0 {
        return Ok("".to_string());
    }
    let mut out = String::new();
    for o in p.iter().take(p.len() - 1) {
        // TODO(lazau): Maybe just split it into more params rather than panicking?
        assert!(
            o.find(' ').is_none(),
            "generating params list with space in non-trailing param"
        );
        write!(out, "{} ", o)?;
    }
    write!(out, ":{}", p[p.len() - 1])?;
    return Ok(out);
}

#[derive(Default)]
pub struct IRCSerializer {
    output: String,
    last_colon: Option<usize>,
}

pub fn to_string<T>(value: &T) -> std::result::Result<String, ParseError>
where
    T: Serialize,
{
    let mut serializer = IRCSerializer::default(); //{ output: String::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;
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
        unimplemented!()
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
    type Error = ParseError;

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
    type Error = ParseError;

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
    type Error = ParseError;

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
    type Error = ParseError;

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
    type Error = ParseError;

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
    type Error = ParseError;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
        /*if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)*/
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unimplemented!()
        /*self.output += "}";
        Ok(())*/
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut IRCSerializer {
    type Ok = ();
    type Error = ParseError;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let initial_len = self.output.as_bytes().len();
        self.output += " :";
        let new_colon_location = self.output.as_bytes().len() - 1;
        let after_space_len = self.output.as_bytes().len();
        value.serialize(&mut **self)?;
        // If value produced a string, remove the last colon, otherwise trim the " :" string added.
        if after_space_len != self.output.as_bytes().len() {
            if let Some(last_colon_pos) = self.last_colon {
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
    use super::{Command, Message, Request};

    macro_rules! test_message_fail {
        ($name:ident, $s:expr) => {
            #[test]
            fn $name() {
                assert!((&format!("{}\r\n", $s)).parse::<Message>().is_err());
            }
        }
    }
    macro_rules! test_message_pass {
        ($name:ident, $input:expr, Message {
            prefix: $prefix:expr,
            command: $command:expr,
            params: [$($params:expr),*],
        }) => {
            #[test]
            fn $name() {
                let s = (&format!("{}\r\n",$input)).parse::<Message>().unwrap();
                let pf = $prefix.to_string();
                if pf.len() == 0 {
                    assert!(s.prefix.is_none());
                } else {
                    assert_eq!(s.prefix.unwrap(), $prefix.to_string());
                }
                assert_eq!(s.command, $command);
                let params:Vec<&str> = vec![$($params),*];
                let expect :Vec<String> = params.iter().map(|s| s.to_string()).collect();
                assert_eq!(expect.len(), s.params.len());
                expect.iter().zip(s.params.iter()).for_each(|p| assert_eq!(p.0, p.1));
            }
        }
    }

    test_message_fail!(empty, "");
    test_message_fail!(just_prefix, ":lazau");

    test_message_pass!(
        hello_world,
        "NICK world",
        Message {
            prefix: "",
            command: Command::Req(Request::NICK),
            params: ["world"],
        }
    );
    test_message_pass!(
        empty_param,
        "PASS",
        Message {
            prefix: "",
            command: Command::Req(Request::PASS),
            params: [],
        }
    );
    test_message_pass!(
        empty_param_trailer,
        "QUIT :",
        Message {
            prefix: "",
            command: Command::Req(Request::QUIT),
            params: [],
        }
    );
    test_message_pass!(
        full,
        ":lazau CONNECT server server2 :server 3 5 6",
        Message {
            prefix: ":lazau",
            command: Command::Req(Request::CONNECT),
            params: ["server", "server2", "server 3 5 6"],
        }
    );
}
