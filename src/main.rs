mod pool;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use crate::pool::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1024").unwrap();
    let pool = ThreadPool::build(4).unwrap();
    for stream in listener.incoming() {
        let st = stream.unwrap();
        pool.execute(|| handle(st));
    }
}

fn handle(mut con: TcpStream) {
    let reader = BufReader::new(&mut con);
    let rq: Vec<String> = reader
        .lines()
        .map(|x| x.unwrap())
        .take_while(|x| !x.is_empty())
        .collect();

    println!("response: {:?}", rq);
    if let Some(n) = rq.get(0) {
        if "GET / HTTP/1.1".eq(n) {
            let status = "HTTP/1.1 200 OK";
            let rp = fs::read_to_string("hello.html").unwrap();
            let length = rp.len();

            let response =
                format!("{status}\r\nContent-Length: {length}\r\n\r\n{rp}");

            con.write_all(response.as_bytes()).unwrap();
        } else {
            let status = "HTTP/1.1 404";
            let rp = fs::read_to_string("404.html").unwrap();
            let length = rp.len();

            let response =
                format!("{status}\r\nContent-Length: {length}\r\n\r\n{rp}");

            con.write_all(response.as_bytes()).unwrap();
        }
    }
}
