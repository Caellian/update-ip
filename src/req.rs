//! Minimal HTTPS client. Writes raw HTTP/1.1 over a `native-tls` TLS stream.

use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Response {
    pub status: u16,
    pub body: String,
}

fn request(method: &str, host: &str, path: &str, headers: &[(&str, &str)], body: Option<&str>) -> Option<Response> {
    let tcp = TcpStream::connect((host, 443)).ok()?;
    let connector = TlsConnector::new().ok()?;
    let mut stream = connector.connect(host, tcp).ok()?;

    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
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
