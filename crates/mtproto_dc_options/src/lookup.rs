//! `DcOptions::lookup` — the endpoints of one data center, split by address
//! family and protocol.
//!
//! Source of truth: `DcOptions::lookup` and `DcOptions::FilterIfHasWithFlag`
//! in `mtproto/mtproto_dc_options.cpp`.

use crate::endpoint::Endpoint;
use crate::flags::{CDN, IPV6, MEDIA_ONLY, STATIC, TCPO_ONLY, SECRET};
use crate::ids::DcType;

/// `Variants::address` — IPv4.
pub const IPV4: usize = 0;
/// `Variants::address` — IPv6.
pub const IPV6_INDEX: usize = 1;
/// `Variants::protocol` — TCP.
pub const TCP: usize = 0;
/// `Variants::protocol` — plain HTTP.
pub const HTTP: usize = 1;

/// `DcOptions::Variants` — candidates per address family and protocol.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Variants {
    /// `data[address][protocol]`.
    pub data: [[Vec<Endpoint>; 2]; 2],
}

impl Variants {
    /// `lookup(dc_id, type, through_proxy)` for one data center.
    #[must_use]
    pub fn build(
        endpoints: &[Endpoint],
        dc_type: DcType,
        through_proxy: bool,
        validate_secret: impl Fn(&[u8]) -> bool,
    ) -> Self {
        let mut result = Self::default();
        for endpoint in endpoints {
            let flags = endpoint.flags;
            if dc_type == DcType::Cdn && !flags.has(CDN) {
                continue;
            } else if dc_type != DcType::MediaCluster && flags.has(MEDIA_ONLY) {
                continue;
            } else if !validate_secret(&endpoint.secret) {
                continue;
            }
            let address = if flags.has(IPV6) { IPV6_INDEX } else { IPV4 };
            result.data[address][TCP].push(endpoint.clone());
            if !flags.has(TCPO_ONLY | SECRET) {
                result.data[address][HTTP].push(endpoint.clone());
            }
        }
        if dc_type == DcType::MediaCluster {
            filter_if_has_with_flag(&mut result, MEDIA_ONLY);
        }
        if through_proxy {
            filter_if_has_with_flag(&mut result, STATIC);
        }
        result
    }
}

/// `DcOptions::FilterIfHasWithFlag` — when any endpoint of a list carries the
/// flag, the list keeps only those that do.
fn filter_if_has_with_flag(variants: &mut Variants, flag: i32) {
    for by_address in &mut variants.data {
        for list in by_address {
            if list.iter().any(|endpoint| endpoint.flags.has(flag)) {
                list.retain(|endpoint| endpoint.flags.has(flag));
            }
        }
    }
}
