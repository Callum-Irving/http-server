use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod http;

const HOST: &str = "127.0.0.1:8080";
const BASE_DIR: &str = ".";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Read command line args for host and port

    let server = TcpListener::bind(HOST).await?; // TODO: Handle error

    loop {
        let (stream, addr) = server.accept().await?;
        tokio::spawn(async move {
            let _ = handle_connection(stream).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    // TODO: If request is bad, send error message instead of closing
    // TODO: Log request
    // TODO: Handle range header
    // TODO: Send date

    stream.readable().await?;
    let mut buf = [0; 8192]; // Apparently this is the standard size for most HTTP servers
    stream.try_read(&mut buf)?;

    let trimmed = buf
        .split(|byte| *byte == 0)
        .next()
        .ok_or("could not trim request")?;
    let request = http::Request::new(trimmed)?;


    // Only accept GET request
    if request.method != http::RequestMethod::GET {
        stream
            .write(
                &http::Response::new(
                    "405 Method Not Allowed".to_string(),
                    HashMap::from([
                        ("Connection".into(), "close".into()),
                        ("Content-Length".into(), "0".into()),
                    ]),
                    None,
                )
                .into_bytes(),
            )
            .await?;
        stream.shutdown().await?;
        return Ok(());
    }

    // TODO: Read BASE_DIR + request.path as bytes then send that
    let file_raw = match fs::read(BASE_DIR.to_string() + &request.path) {
        Ok(bytes) => bytes,
        Err(_) => {
            // 404 Error
            let res = stream
                .write(
                    &http::Response::new(
                        "404 Not Found".into(),
                        HashMap::from([
                            ("Connection".into(), "close".into()),
                            ("Content-Length".into(), "0".into()),
                        ]),
                        None,
                    )
                    .into_bytes(),
                )
                .await;
            if res.is_err() {
                return Err(res.unwrap_err().into());
            }
            let res = stream.shutdown().await;
            if res.is_err() {
                return Err(res.unwrap_err().into());
            }
            return Ok(());
        }
    };

    let response = http::Response::new(
        "200 Ok".to_string(),
        HashMap::from([
            ("Content-Type".into(), "text/html; charset=UTF-8".into()),
            ("Content-Length".into(), file_raw.len().to_string()),
            ("Connection".to_string(), "close".to_string()),
        ]),
        Some(file_raw),
    );

    stream.write(&response.into_bytes()).await?;
    stream.shutdown().await?;
    Ok(())
}
