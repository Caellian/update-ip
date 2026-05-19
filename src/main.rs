use crate::provider::*;
use crate::resolver::*;

mod provider;
mod req;
mod resolver;

pub trait Address: std::fmt::Display + Copy {
    const RECORD_TYPE: &'static str;
}
impl Address for std::net::Ipv4Addr {
    const RECORD_TYPE: &'static str = "A";
}
impl Address for std::net::Ipv6Addr {
    const RECORD_TYPE: &'static str = "AAAA";
}

pub fn env<K: AsRef<std::ffi::OsStr> + ?Sized>(var: &'static K) -> String {
    let var = var.as_ref();
    match std::env::var(var) {
        Ok(it) => return it,
        Err(std::env::VarError::NotPresent) => {
            eprintln!("missing '{}' environment variable", var.to_string_lossy())
        }
        Err(std::env::VarError::NotUnicode(_)) => eprintln!(
            "'{}' environment variable value is not unicode",
            var.to_string_lossy()
        ),
    }
    std::process::exit(1);
}

fn main() {
    let record_names = env("DNS_RECORD_NAME");
    let resolver = resolver::Default::new();
    let provider = provider::Default::new();

    let ipv4: Option<std::net::Ipv4Addr> = match resolver.public_address() {
        Ok(ip) => Some(ip),
        Err(no_addr) if no_addr.kind() == std::io::ErrorKind::AddrNotAvailable => None,
        Err(err) => {
            eprintln!("IPv4 resolution failed: {err}");
            None
        }
    };
    let ipv6: Option<std::net::Ipv6Addr> = match resolver.public_address() {
        Ok(ip) => Some(ip),
        Err(no_addr) if no_addr.kind() == std::io::ErrorKind::AddrNotAvailable => None,
        Err(err) => {
            eprintln!("IPv6 resolution failed: {err}");
            None
        }
    };

    if ipv4.is_none() && ipv6.is_none() {
        eprintln!("Could not fetch any public IP.");
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
