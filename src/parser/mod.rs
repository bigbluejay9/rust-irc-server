pub mod errors;

#[derive(Debug)]
pub struct Command {
    prefix: Option<String>,
    command: String,
    params: Vec<String>,
}

#[derive(Debug)]
struct Syntax {
    prefix: Option<String>,
    command: String,
    params: Vec<String>,
}

pub fn parse_command(input: &String) -> Result<Command, errors::ParseError> {
    let syntax = parse_syntax(input)?;
    Ok(Command {
        prefix: syntax.prefix,
        command: syntax.command,
        params: syntax.params,
    })
}

// RFC 1459 2
fn parse_syntax(input: &String) -> Result<Syntax, errors::ParseError> {
    if input.len() < 2 || input.len() > 512 {
        return Err(errors::ParseError::new("bad command length"));
    }
    if !input.ends_with("\r\n") {
        return Err(errors::ParseError::new("command doesn't end with CR LF"));
    }

    let mut remainder: &str = &input.trim_right();
    debug!("Processing {:?}", remainder);

    let mut prefix: Option<String> = None;
    if remainder.starts_with(':') {
        match remainder.find(' ') {
            Some(idx) => {
                prefix = Some(remainder[0..idx].to_string());
                remainder = &remainder[idx + 1..];
            }
            None => {
                return Err(errors::ParseError::new("only command prefix given"));
            }
        }
    }

    if remainder.len() < 1 {
        return Err(errors::ParseError::new("no command specified"));
    }
    let command: String;
    match remainder.find(' ') {
        Some(idx) => {
            command = remainder[0..idx].to_string();
            remainder = &remainder[idx + 1..];
        }
        None => {
            command = remainder.to_string();
            remainder = "";
        }
    }

    let mut params: Vec<String> = Vec::new();
    while remainder.len() > 0 {
        if remainder.starts_with(':') {
            if remainder.len() == 1 {
                warn!("Empty trailing command parameter. Ignoring.")
            } else {
                params.push(remainder[1..].to_string());
            }
            break;
        }
        match remainder.find(' ') {
            Some(idx) => {
                if idx == 0 {
                    warn!("Empty whitespace in command paramter detected! Ignoring.");
                } else {
                    params.push(remainder[0..idx].to_string());
                }
                remainder = &remainder[idx + 1..];
            }
            None => {
                params.push(remainder.to_string());
                break;
            }
        }
    }

    debug!(
        "Parsed {} to prefix: [{:?}]; command: [{}]; params: [{:?}].",
        input,
        prefix,
        command,
        params
    );

    Ok(Syntax {
        prefix: prefix,
        command: command,
        params: params,
    })
}

#[cfg(test)]
mod test {
    use super::parse_syntax;

    macro_rules! test_syntax_fail {
        ($name:ident, $s:expr) => {
            #[test]
            fn $name() {
                assert!(parse_syntax(&format!("{}\r\n", $s)).is_err());
            }
        }
    }
    macro_rules! test_syntax_pass {
        ($name:ident, $input:expr, Syntax {
            prefix: $prefix:expr,
            command: $command:expr,
            params: [$($params:expr),*],
        }) => {
            #[test]
            fn $name() {
                let s = parse_syntax(&format!("{}\r\n",$input)).unwrap();
                let pf = $prefix.to_string();
                if pf.len() == 0 {
                    assert!(s.prefix.is_none());
                } else {
                    assert_eq!(s.prefix.unwrap(), $prefix.to_string());
                }
                assert_eq!(s.command, $command.to_string());
                let params:Vec<&str> = vec![$($params),*];
                let expect :Vec<String> = params.iter().map(|s| s.to_string()).collect();
                assert_eq!(expect.len(), s.params.len());
                expect.iter().zip(s.params.iter()).for_each(|p| assert_eq!(p.0, p.1));
            }
        }
    }

    test_syntax_fail!(empty, "");
    test_syntax_fail!(just_prefix, ":lazau");

    test_syntax_pass!(
        hello_world,
        "hello world",
        Syntax {
            prefix: "",
            command: "hello",
            params: ["world"],
        }
    );
    test_syntax_pass!(
        empty_param,
        "comm",
        Syntax {
            prefix: "",
            command: "comm",
            params: [],
        }
    );
    test_syntax_pass!(
        empty_param_trailer,
        "hello :",
        Syntax {
            prefix: "",
            command: "hello",
            params: [],
        }
    );
    test_syntax_pass!(
        full,
        ":lazau CONNECT server server2 :server 3 5 6",
        Syntax {
            prefix: ":lazau",
            command: "CONNECT",
            params: ["server", "server2", "server 3 5 6"],
        }
    );
}
