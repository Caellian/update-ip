//! DNS record management with a domain provider.
//!
//! Implementations create and update DNS records (A/AAAA) via a provider's API.

use crate::Address;

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
        match HandleRecord::<A>::get_record_id(self, record_name) {
            Some(record_id) => {
                if self.update_dns_record(record_id, record_name, ip) {
                    eprintln!("Updated {} record to {ip}.", A::RECORD_TYPE);
                } else {
                    eprintln!("Failed to update {} record.", A::RECORD_TYPE);
                }
            }
            None => {
                if self.create_dns_record(record_name, ip) {
                    eprintln!("Created {} record for {ip}.", A::RECORD_TYPE);
                } else {
                    eprintln!("Failed to create {} record.", A::RECORD_TYPE);
                }
            }
        }
    }
}

pub trait DnsProvider:
    HandleRecord<std::net::Ipv4Addr> + HandleRecord<std::net::Ipv6Addr> + Sized
{
    fn new() -> Self;
}
