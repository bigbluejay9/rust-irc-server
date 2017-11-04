use std;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    desc: &'static str,
}

impl ParseError {
    pub fn new(desc: &'static str) -> ParseError {
        ParseError { desc: desc }
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
