//! DNS record management with a domain provider.
//!
//! Implementations create and update DNS records (A/AAAA) via a provider's API.

use crate::addr::{Address, Ipv4Address, Ipv6Address};
use crate::util::{cat_stderr, write_cat};

#[cfg(feature = "provider-cloudflare")]
mod cloudflare;

#[cfg(feature = "provider-cloudflare")]
pub type Default = cloudflare::Cloudflare;

#[cfg(not(feature = "provider-cloudflare"))]
compile_error!("no domain provider enabled");

pub trait HandleRecord<A: Address> {
    type RecordId;

    fn get_record_id(&self, record_name: &str) -> Option<Self::RecordId>;
    fn update_dns_record(&self, record_id: Self::RecordId, record_name: &str, ip: A) -> bool;
    fn create_dns_record(&self, record_name: &str, ip: A) -> bool;

    fn upsert_record(&self, record_name: &str, ip: A) {
        let mut stderr = std::io::stderr();
        match HandleRecord::<A>::get_record_id(self, record_name) {
            Some(record_id) => {
                if self.update_dns_record(record_id, record_name, ip) {
                    let _ = write_cat(&mut stderr, &[b"Updated ", A::RECORD_TYPE.as_bytes(), b" record to "]);
                    let _ = ip.write_str(&mut stderr);
                    let _ = write_cat(&mut stderr, &[b".\n"]);
                } else {
                    let _ = cat_stderr(&[b"Failed to update ", A::RECORD_TYPE.as_bytes(), b" record.\n"]);
                }
            }
            None => {
                if self.create_dns_record(record_name, ip) {
                    let _ = write_cat(&mut stderr, &[b"Created ", A::RECORD_TYPE.as_bytes(), b" record with "]);
                    let _ = ip.write_str(&mut stderr);
                    let _ = write_cat(&mut stderr, &[b".\n"]);
                } else {
                    let _ = cat_stderr(&[b"Failed to create ", A::RECORD_TYPE.as_bytes(), b" record.\n"]);
                }
            }
        }
    }
}

pub trait DnsProvider:
    HandleRecord<Ipv4Address> + HandleRecord<Ipv6Address> + Sized
{
    fn new() -> Self;
}
