//! One connect option of one data center.
//!
//! Source of truth: `DcOptions::Endpoint` in `mtproto/mtproto_dc_options.h`.

use crate::flags::{Flags, IPV6};
use crate::ids::DcId;

/// One endpoint of one data center.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub id: DcId,
    pub flags: Flags,
    pub ip: String,
    pub port: i32,
    pub secret: Vec<u8>,
}

impl Endpoint {
    #[must_use]
    pub fn new(id: DcId, flags: Flags, ip: String, port: i32, secret: Vec<u8>) -> Self {
        Self {
            id,
            flags,
            ip,
            port,
            secret,
        }
    }

    /// Whether the endpoint was written as IPv6.
    #[must_use]
    pub fn is_ipv6(&self) -> bool {
        self.flags.has(IPV6)
    }
}