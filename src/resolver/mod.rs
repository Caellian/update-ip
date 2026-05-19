//! Public IP address resolution.
//!
//! Implementations determine the machine's public IP by any means (DNS, HTTP,
//! STUN, etc.) and return it as a standard address type.

use crate::addr::*;

#[cfg(feature = "resolver-opendns")]
mod opendns;

#[cfg(feature = "resolver-opendns")]
pub type Default = opendns::OpenDNS;

#[cfg(not(feature = "resolver-opendns"))]
compile_error!("no public IP resolver enabled");

pub trait ResolvePublicAddress<A: Address> {
    fn public_address(&self) -> Result<A, std::io::Error>;
}

pub trait Resolver:
    ResolvePublicAddress<Ipv4Address> + ResolvePublicAddress<Ipv6Address> + Sized
{
    fn new() -> Self;
}
