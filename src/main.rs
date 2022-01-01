use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod http;

const HOST: &str = "127.0.0.1:8080";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Read command line args for host and port

    let server = TcpListener::bind(HOST).await?; // TODO: Handle error

    loop {
        let (stream, addr) = server.accept().await?;
        tokio::spawn(async move {
            println!("Got request from: {}", addr);
            let _ = handle_connection(stream).await;
        });
    }
}

const RESPONSE: &str = "HTTP/1.1 200 OK\n\
                        Content-Type: application/json; charset=utf-8\n\
                        Content-Length: 75\n\
                        Connection: close\n\
                        \n\
                        {\n\
                        \"userId\": 1,\n\
                        \"id\": 1,\n\
                        \"title\": \"delectus aut autem\",\n\
                        \"completed\": false\n\
                        }";

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    // TODO: If request is bad, send error message instead of closing
    stream.readable().await?;
    let mut buf = [0; 8192]; // Apparently this is the standard size for most HTTP servers
    stream.try_read(&mut buf)?;

    println!("Parsing request ...");
    let trimmed = buf.split(|byte| *byte == 0).next().ok_or("could not trim request")?;
    let request = http::HttpRequest::new(trimmed)?;

    println!("REQUEST DATA:\n{:?}", request);
    // Only accept GET request
    // Handle range header
    // Send date

    stream.write(RESPONSE.as_bytes()).await?;
    stream.shutdown().await?;
    Ok(())
}
