use std::io::Result;
use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket};
use std::time::Duration;

use crate::addr::*;

pub struct OpenDNS;

// resolver1.opendns.com
const OPENDNS_V4: (Ipv4Addr, u16) = (Ipv4Addr::new(208, 67, 222, 222), 53);
const OPENDNS_V6: (Ipv6Addr, u16) = (Ipv6Addr::new(0x2620, 0x119, 0x35, 0, 0, 0, 0, 0x35), 53);

fn invalid_data(inner: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, inner)
}

impl OpenDNS {
    /// Build a DNS query for myip.opendns.com with the given QTYPE.
    fn build_query(qtype: u16) -> Vec<u8> {
        let mut query = Vec::with_capacity(64);
        // Header: ID=0xABCD, flags=0x0100 (standard query, RD=1)
        // QDCOUNT=1, ANCOUNT=0, NSCOUNT=0, ARCOUNT=0
        query.extend_from_slice(&[
            0xAB, 0xCD, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        for label in ["myip", "opendns", "com"] {
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0);
        query.extend_from_slice(&qtype.to_be_bytes());
        query.extend_from_slice(&[0x00, 0x01]); // QCLASS=IN
        query
    }

    /// Parse the first answer RDATA from a DNS response.
    fn parse_rdata(buf: &[u8], len: usize) -> Result<&[u8]> {
        if len < 12 || buf[0] != 0xAB || buf[1] != 0xCD {
            return Err(invalid_data("invalid DNS response header"));
        }
        let ancount = u16::from_be_bytes([buf[6], buf[7]]);
        if ancount == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "no address in DNS response",
            ));
        }
        // Skip header (12 bytes) then question QNAME
        let mut pos = 12;
        while pos < len && buf[pos] != 0 {
            pos += buf[pos] as usize + 1;
        }
        pos += 1 + 4; // null terminator + QTYPE + QCLASS

        // Skip answer NAME
        if pos >= len {
            return Err(invalid_data("truncated DNS answer"));
        }
        if buf[pos] & 0xC0 == 0xC0 {
            pos += 2;
        } else {
            while pos < len && buf[pos] != 0 {
                pos += buf[pos] as usize + 1;
            }
            pos += 1;
        }

        // TYPE(2) + CLASS(2) + TTL(4) + RDLENGTH(2)
        if pos + 10 > len {
            return Err(invalid_data("truncated DNS answer record"));
        }
        let rdlength = u16::from_be_bytes([buf[pos + 8], buf[pos + 9]]) as usize;
        pos += 10;

        if pos + rdlength > len {
            return Err(invalid_data("RDATA extends past response"));
        }
        Ok(&buf[pos..pos + rdlength])
    }
}

impl super::ResolvePublicAddress<Ipv4Address> for OpenDNS {
    fn public_address(&self) -> Result<Ipv4Address> {
        let query = OpenDNS::build_query(1);
        let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        sock.set_read_timeout(Some(Duration::from_secs(5)))?;
        sock.send_to(&query, OPENDNS_V4)?;

        let mut buf = [0u8; 512];
        let len = sock.recv(&mut buf)?;
        let octets = OpenDNS::parse_rdata(&buf, len)?;
        let octets: [u8; 4] = octets.try_into().map_err(invalid_data)?;
        Ok(Ipv4Address(octets))
    }
}

impl super::ResolvePublicAddress<Ipv6Address> for OpenDNS {
    fn public_address(&self) -> Result<Ipv6Address> {
        let query = OpenDNS::build_query(28);
        let sock = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0))?;
        sock.set_read_timeout(Some(Duration::from_secs(5)))?;
        sock.send_to(&query, OPENDNS_V6)?;

        let mut buf = [0u8; 512];
        let len = sock.recv(&mut buf)?;
        let octets = OpenDNS::parse_rdata(&buf, len)?;
        let octets: [u8; 16] = octets.try_into().map_err(invalid_data)?;
        Ok(Ipv6Address(octets))
    }
}

impl super::Resolver for OpenDNS {
    #[inline(always)]
    fn new() -> Self {
        Self
    }
}
