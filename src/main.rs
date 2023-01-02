mod pool;

use std::error::Error;
use async_std::fs;
use async_std::net::{TcpListener, TcpStream};
use futures_lite::{AsyncBufReadExt, AsyncWriteExt};
use futures_lite::io::BufReader;
use futures_lite::stream::StreamExt;
// use crate::pool::ThreadPool;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    // let pool = ThreadPool::build(4).unwrap();
    loop {
        let receive = listener.incoming().next().await;
        if let Some(x) = receive {
            handle(x?).await?;
        }
    }
}

async fn handle(mut con: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut rq = BufReader::new(&con)
        .lines()
        .map(|x| x.unwrap_or_default())
        .take_while(|x| !x.is_empty())
        .next()
        .await;

    println!("response: {:?}", rq);

    if let Some(ref n) = rq {
        let response = if "GET / HTTP/1.1".eq(n) {
            let status = "HTTP/1.1 200 OK";
            let rp = fs::read_to_string("hello.html").await?;
            let length = rp.len();

            format!("{status}\r\nContent-Length: {length}\r\n\r\n{rp}")
        } else {
            let status = "HTTP/1.1 404";
            let rp = fs::read_to_string("404.html").await?;
            let length = rp.len();

            format!("{status}\r\nContent-Length: {length}\r\n\r\n{rp}")
        };
        con.write_all(response.as_bytes()).await?;
        con.flush().await?;
    }
    Ok({})
}
