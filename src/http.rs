use crate::Result;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub enum HttpRequestMethod {
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
pub struct HttpRequestMethodParseError;

impl std::fmt::Display for HttpRequestMethodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid HTTP request method")
    }
}

impl std::error::Error for HttpRequestMethodParseError {}

impl FromStr for HttpRequestMethod {
    type Err = HttpRequestMethodParseError;

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
            _ => Err(HttpRequestMethodParseError),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpRequestMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(raw: &[u8]) -> Result<HttpRequest> {
        let mut lines = raw.split(|byte| *byte == '\n' as u8);

        // Parse status line
        // TODO: Remove \r
        println!("Parsing status line ...");
        let mut status_line = lines
            .next()
            .ok_or("could not read status line")?
            .split(|byte| *byte == ' ' as u8);
        let method_raw = status_line.next().ok_or("no method provided")?;
        let method_str = std::str::from_utf8(method_raw)?;
        let method = HttpRequestMethod::from_str(method_str)?;
        let path_raw = status_line.next().ok_or("no path provided")?;
        let path = std::str::from_utf8(path_raw)?.to_string();
        let version_raw = status_line.next().ok_or("no HTTP version provided")?;
        let version = std::str::from_utf8(version_raw)?.to_string();

        // Parse headers
        println!("Parsing headers ...");
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
            println!("HEADER LINE: {:?}", line);
            let mut v = line.split(|byte| *byte == ':' as u8);
            let key_raw = v.next().ok_or("no key provided")?;
            let key = std::str::from_utf8(key_raw)?.to_string();
            let value_raw = v.next().ok_or("no value provided")?;
            let value = std::str::from_utf8(value_raw)?.trim_start().to_string();
            headers.insert(key, value);
        }

        // Parse body
        println!("Parsing body ...");
        let mut lines = lines.peekable();
        let body = if lines.peek().is_some() {
            Some(lines.collect::<Vec<&[u8]>>().join(&('\n' as u8)))
        } else {
            None
        };

        Ok(HttpRequest {
            method: method,
            path: path,
            version: version,
            headers: headers,
            body: body,
        })
    }
}

pub enum HttpResponseStatus {
    OK,
}

pub struct HttpResponse {
    status: HttpResponseStatus,
    headers: HashMap<String, String>,
    body: Option<String>,
}
