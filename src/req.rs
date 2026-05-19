//! Minimal HTTPS client. Raw OpenSSL FFI over std TCP.

use crate::ssl::SslStream;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct Response {
    pub status: u16,
    pub body: String,
}

const TIMEOUT: Duration = Duration::from_secs(10);

fn request(method: &str, host: &str, path: &str, headers: &[(&str, &str)], body: Option<&str>) -> Option<Response> {
    let addr = (host, 443).to_socket_addrs().ok()?.next()?;
    let tcp = TcpStream::connect_timeout(&addr, TIMEOUT).ok()?;
    tcp.set_read_timeout(Some(TIMEOUT)).ok()?;
    let mut stream = SslStream::connect(host, tcp)?;

    let mut req = format!("{method} {path} HTTP/1.0\r\nHost: {host}\r\n");
    for (k, v) in headers {
        req.push_str(k);
        req.push_str(": ");
        req.push_str(v);
        req.push_str("\r\n");
    }
    if let Some(body) = body {
        req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("\r\n");
    if let Some(body) = body {
        req.push_str(body);
    }

    stream.write_all(req.as_bytes()).ok()?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    let raw = String::from_utf8_lossy(&buf);

    let status_line = raw.lines().next()?;
    let status: u16 = status_line.split(' ').nth(1)?.parse().ok()?;

    let body = raw.split_once("\r\n\r\n").map(|(_, b)| b.to_string()).unwrap_or_default();

    Some(Response { status, body })
}

pub fn get(host: &str, path: &str, headers: &[(&str, &str)]) -> Option<Response> {
    request("GET", host, path, headers, None)
}

pub fn put(host: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Option<Response> {
    request("PUT", host, path, headers, Some(body))
}

pub fn post(host: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Option<Response> {
    request("POST", host, path, headers, Some(body))
}
