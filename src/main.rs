use crate::addr::*;
use crate::provider::*;
use crate::resolver::*;

mod provider;
mod req;
mod resolver;
mod ssl;
mod addr;


fn main() {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    let record_names = util::env("DNS_RECORD_NAME");
    let resolver = resolver::Default::new();
    let provider = provider::Default::new();

    let ipv4: Option<Ipv4Address> = match resolver.public_address() {
        Ok(ip) => Some(ip),
        Err(no_addr) if no_addr.kind() == std::io::ErrorKind::AddrNotAvailable => None,
        Err(_) => {
            let _ = stderr.write_all(b"IPv4 resolution failed\n");
            None
        }
    };
    let ipv6: Option<Ipv6Address> = match resolver.public_address() {
        Ok(ip) => Some(ip),
        Err(no_addr) if no_addr.kind() == std::io::ErrorKind::AddrNotAvailable => None,
        Err(_) => {
            let _ = stderr.write_all(b"IPv6 resolution failed\n");
            None
        }
    };

    if ipv4.is_none() && ipv6.is_none() {
        let _ = stderr.write_all(b"Could not fetch any public IP.\n");
        std::process::exit(1);
    }

    for record_name in record_names.split(';').filter(|s| !s.is_empty()) {
        if let Some(ip) = ipv4 {
            provider.upsert_record(record_name, ip);
        }
        if let Some(ip) = ipv6 {
            provider.upsert_record(record_name, ip);
        }
    }
}

pub mod util {
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;

    pub fn cat(parts: &[&str]) -> String {
        let len: usize = parts.iter().map(|s| s.len()).sum();
        let mut out = String::with_capacity(len);
        for part in parts {
            out.push_str(part);
        }
        out
    }

    pub fn write_cat<W: Write>(w: &mut W, parts: &[&[u8]]) -> std::io::Result<()> {
        for part in parts {
            w.write_all(part)?;
        }
        Ok(())
    }

    pub fn cat_stderr(parts: &[&[u8]]) -> std::io::Result<()> {
        let mut stderr = std::io::stderr();
        write_cat(&mut stderr, parts)?;
        stderr.flush()
    }

    pub fn u8_to_str<const N: usize>(mut n: u8, buf: &mut [u8; N]) -> &str {
        if n == 0 {
            buf[N - 1] = b'0';
            return unsafe { std::str::from_utf8_unchecked(&buf[(N - 1)..]) };
        }
        let mut i = N;
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10);
            n /= 10;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[i..]) }
    }

    pub fn usize_to_str<const N: usize>(mut n: usize, buf: &mut [u8; N]) -> &str {
        if n == 0 {
            buf[N - 1] = b'0';
            return unsafe { std::str::from_utf8_unchecked(&buf[(N - 1)..]) };
        }
        let mut i = N;
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[i..]) }
    }

    pub fn parse_u16(s: &str) -> Option<u16> {
        let mut n: u16 = 0;
        for &b in s.as_bytes() {
            if !b.is_ascii_digit() {
                return None;
            }
            n = n.checked_mul(10)?.checked_add((b - b'0') as u16)?;
        }
        Some(n)
    }

    pub fn env<K: AsRef<std::ffi::OsStr> + ?Sized>(var: &'static K) -> String {
        let var = var.as_ref();
        match std::env::var(var) {
            Ok(it) => return it,
            Err(std::env::VarError::NotPresent) => {
              let mut stderr = std::io::stderr();
              let _ = stderr.write_all(b"missing '");
              let _ = stderr.write_all(var.as_bytes());
              let _ = stderr.write_all(b"' environment variable");
              let _ = stderr.flush();
            }
            Err(std::env::VarError::NotUnicode(_)) => {
              let mut stderr = std::io::stderr();
              let _ = stderr.write_all(b"'");
              let _ = stderr.write_all(var.as_bytes());
              let _ = stderr.write_all(b"' environment variable value is not unicode");
              let _ = stderr.flush();
            },
        }
        std::process::exit(1);
    }
}
