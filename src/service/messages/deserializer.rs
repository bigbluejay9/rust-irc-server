use serde;
use std;

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
