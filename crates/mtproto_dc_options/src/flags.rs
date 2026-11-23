//! `MTPDdcOption::Flags` — the option bits of one endpoint.
//!
//! Source of truth: the `dcOption#18b7a10d` constructor in
//! `mtproto/scheme/api.tl`, which fixes the bit each flag occupies.

/// `f_ipv6` — the address is IPv6.
pub const IPV6: i32 = 1 << 0;
/// `f_media_only` — the endpoint serves media only.
pub const MEDIA_ONLY: i32 = 1 << 1;
/// `f_tcpo_only` — TCP with obfuscation only, no plain HTTP.
pub const TCPO_ONLY: i32 = 1 << 2;
/// `f_cdn` — a CDN endpoint.
pub const CDN: i32 = 1 << 3;
/// `f_static` — an endpoint that is always available (a built-in one).
pub const STATIC: i32 = 1 << 4;
/// `f_this_port_only` — the port is fixed, no probing of others.
pub const THIS_PORT_ONLY: i32 = 1 << 5;
/// `f_secret` — the endpoint carries a proxy secret.
pub const SECRET: i32 = 1 << 10;

/// A set of `dcOption` flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Flags(i32);

impl Flags {
    /// No flags.
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }

    /// `Flags::from_raw`.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// The raw bit set, as stored in files and sent over TL.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self.0
    }

    /// Whether every bit of `flag` is set.
    #[must_use]
    pub const fn has(self, flag: i32) -> bool {
        self.0 & flag == flag
    }

    /// The same set with `flag` added.
    #[must_use]
    pub const fn with(self, flag: i32) -> Self {
        Self(self.0 | flag)
    }

    /// `isCdnDc` — the endpoint is a CDN one.
    #[must_use]
    pub const fn is_cdn(self) -> bool {
        self.0 & CDN != 0
    }
}

impl std::ops::BitOr for Flags {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
