//! A tiny HTTP/1.1 server on 127.0.0.1 for tests, std only. Shared by the
//! acceptance suite and `services::web`'s unit tests through `#[path]`.
#![allow(dead_code)] // each includer uses a different part

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// What the client sent. Header names are lowercase.
#[derive(Debug, Clone)]
pub struct Request {
    pub path: String,
    pub headers: HashMap<String, String>,
}

/// What to send back. `Content-Length` and `Connection: close` are added.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
    pub body: Vec<u8>,
    /// `Content-Length` to claim, if not the body's real length.
    pub declared_length: Option<usize>,
    /// How long to keep the connection open after sending.
    pub hold_open: Duration,
}

impl Reply {
    pub fn new(status: u16) -> Self {
        Reply { status, headers: Vec::new(), body: Vec::new(), declared_length: None, hold_open: Duration::ZERO }
    }

    pub fn markdown(text: &str) -> Self {
        Reply::new(200).header("Content-Type", "text/markdown; charset=utf-8").body(text)
    }

    pub fn redirect(status: u16, location: &str) -> Self {
        Reply::new(status).header("Location", location)
    }

    pub fn header(self, name: &str, value: &str) -> Self {
        self.raw_header(name, value.as_bytes())
    }

    /// A header value that need not be UTF-8.
    pub fn raw_header(mut self, name: &str, value: &[u8]) -> Self {
        self.headers.push((name.to_string(), value.to_vec()));
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Promise more body than is sent, then wait `hold_open` before hanging up.
    pub fn cut_short(mut self, declared_length: usize, hold_open: Duration) -> Self {
        self.declared_length = Some(declared_length);
        self.hold_open = hold_open;
        self
    }
}

type Handler = dyn Fn(&Request) -> Reply + Send + Sync;

/// Serves every connection on its own thread until the process ends.
pub struct Server {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl Server {
    pub fn start(handler: impl Fn(&Request) -> Reply + Send + Sync + 'static) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a test port");
        let addr = listener.local_addr().expect("test port address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let handler: Arc<Handler> = Arc::new(handler);
        let log = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (handler, log) = (Arc::clone(&handler), Arc::clone(&log));
                thread::spawn(move || serve(stream, &*handler, &log));
            }
        });
        Server { addr, requests }
    }

    /// `http://127.0.0.1:port` + `path`.
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().expect("request log").clone()
    }
}

fn serve(stream: TcpStream, handler: &Handler, log: &Mutex<Vec<Request>>) {
    let Some(request) = read_request(&stream) else { return };
    log.lock().expect("request log").push(request.clone());
    let reply = handler(&request);
    let mut out = format!("HTTP/1.1 {} Test\r\n", reply.status).into_bytes();
    for (name, value) in &reply.headers {
        out.extend_from_slice(format!("{name}: ").as_bytes());
        out.extend_from_slice(value);
        out.extend_from_slice(b"\r\n");
    }
    let length = reply.declared_length.unwrap_or(reply.body.len());
    out.extend_from_slice(format!("Content-Length: {length}\r\nConnection: close\r\n\r\n").as_bytes());
    out.extend_from_slice(&reply.body);
    let _ = (&stream).write_all(&out);
    thread::sleep(reply.hold_open);
}

/// The request line and headers; any body is ignored (tests only send GET).
fn read_request(stream: &TcpStream) -> Option<Request> {
    let mut lines = BufReader::new(stream).lines().map_while(Result::ok);
    let path = lines.next()?.split_whitespace().nth(1)?.to_string();
    let headers = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect();
    Some(Request { path, headers })
}
