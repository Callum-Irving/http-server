use crate::Result;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

#[derive(Debug)]
pub struct RequestMethodParseError;

impl std::fmt::Display for RequestMethodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid HTTP request method")
    }
}

impl std::error::Error for RequestMethodParseError {}

impl FromStr for RequestMethod {
    type Err = RequestMethodParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(RequestMethodParseError),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: RequestMethod,
    pub path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(raw: &[u8]) -> Result<Request> {
        let mut lines = raw.split(|byte| *byte == '\n' as u8);

        // Parse status line
        // TODO: Remove \r
        let mut status_line = lines
            .next()
            .ok_or("could not read status line")?
            .split(|byte| *byte == ' ' as u8);
        let method_raw = status_line.next().ok_or("no method provided")?;
        let method_str = std::str::from_utf8(method_raw)?;
        let method = RequestMethod::from_str(method_str)?;
        let path_raw = status_line.next().ok_or("no path provided")?;
        let path = std::str::from_utf8(path_raw)?.to_string();
        let version_raw = status_line.next().ok_or("no HTTP version provided")?;
        let version = std::str::from_utf8(version_raw)?.to_string();

        // Parse headers
        let mut headers = HashMap::new();
        while let Some(line) = lines.next() {
            let line = if line.last() == Some(&13) {
                &line[0..line.len() - 1]
            } else {
                line
            };
            if line == [] {
                break;
            }
            let mut v = line.split(|byte| *byte == ':' as u8);
            let key_raw = v.next().ok_or("no key provided")?;
            let key = std::str::from_utf8(key_raw)?.to_string();
            let value_raw = v.next().ok_or("no value provided")?;
            let value = std::str::from_utf8(value_raw)?.trim_start().to_string();
            headers.insert(key, value);
        }

        // Parse body
        let mut lines = lines.peekable();
        let body = if lines.peek().is_some() {
            Some(lines.collect::<Vec<&[u8]>>().join(&('\n' as u8)))
        } else {
            None
        };

        Ok(Request {
            method: method,
            path: path,
            version: version,
            headers: headers,
            body: body,
        })
    }
}

// #[derive(Debug)]
// pub struct ResponseStatusError;
//
// impl std::fmt::Display for ResponseStatusError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "invalid HTTP response status")
//     }
// }
//
// impl std::error::Error for ResponseStatusError {}
//
// pub enum ResponseStatus {
//     OK,
// }
//
// impl ResponseStatus {
//     pub fn from_code(code: usize) -> Result<ResponseStatus> {
//         match code {
//             200 => Ok(ResponseStatus::OK),
//             _ => Err(ResponseStatusError.into()),
//         }
//     }
// }

pub struct Response {
    status: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(
        status: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Response {
        Response {
            status,
            headers,
            body,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut response_str = String::from(format!("HTTP/1.1 {}\r\n", self.status));
        for (key, value) in self.headers {
            response_str.push_str(format!("{}: {}\r\n", key, value).as_str());
        }
        response_str.push_str("\r\n");
        let mut bytes = response_str.into_bytes();
        if self.body.is_some() {
            bytes.append(&mut self.body.unwrap());
        }
        bytes
    }
}
