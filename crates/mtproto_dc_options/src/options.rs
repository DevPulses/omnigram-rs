//! `DcOptions` — the connect options of every data center.
//!
//! Source of truth: `mtproto/mtproto_dc_options.{h,cpp}`.
//!
//! This is the data half of the class: the built-in endpoints, adding and
//! merging options, the endpoints per dc, the CDN id set and the serialized
//! blob. What needs the TL scheme (`setFromList` / `setCDNConfig` with real
//! `MTPDcOption` values), the RSA store (`getDcRSAKey`) and `rpl` change
//! streams is not here; see the crate docs for the exact seam.

use std::collections::{BTreeMap, BTreeSet};

use crate::blob::{BlobReader, BlobWriter, MAX_IP_SIZE, MAX_SECRET_SIZE, VERSION};
use crate::endpoint::Endpoint;
use crate::environment::Environment;
use crate::flags::{Flags, CDN, IPV6, STATIC};
use crate::ids::{self, DcId, DcType, ShiftedDcId};
use crate::lookup::Variants;

/// One CDN public key as stored in the blob: modulus and exponent.
///
/// The C++ store keys these by RSA fingerprint and drops keys that do not
/// parse; fingerprinting belongs to the RSA module, so the entries travel
/// verbatim and in file order, which is the order a C++ build writes them in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdnPublicKey {
    pub n: Vec<u8>,
    pub e: Vec<u8>,
}

/// `DcOptions::Endpoint` arguments as a TL-free tuple, used by
/// `setFromList` callers that already decoded the scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionEntry {
    pub dc_id: DcId,
    pub flags: Flags,
    pub ip: String,
    pub port: i32,
    pub secret: Vec<u8>,
}

struct BuiltInDc {
    id: i32,
    ip: &'static str,
    port: i32,
}

/// `kBuiltInDcs`.
const BUILT_IN: [BuiltInDc; 6] = [
    BuiltInDc { id: 1, ip: "149.154.175.50", port: 443 },
    BuiltInDc { id: 2, ip: "149.154.167.51", port: 443 },
    BuiltInDc { id: 2, ip: "95.161.76.100", port: 443 },
    BuiltInDc { id: 3, ip: "149.154.175.100", port: 443 },
    BuiltInDc { id: 4, ip: "149.154.167.91", port: 443 },
    BuiltInDc { id: 5, ip: "149.154.171.5", port: 443 },
];

/// `kBuiltInDcsIPv6`.
const BUILT_IN_IPV6: [BuiltInDc; 5] = [
    BuiltInDc { id: 1, ip: "2001:0b28:f23d:f001:0000:0000:0000:000a", port: 443 },
    BuiltInDc { id: 2, ip: "2001:067c:04e8:f002:0000:0000:0000:000a", port: 443 },
    BuiltInDc { id: 3, ip: "2001:0b28:f23d:f003:0000:0000:0000:000a", port: 443 },
    BuiltInDc { id: 4, ip: "2001:067c:04e8:f004:0000:0000:0000:000a", port: 443 },
    BuiltInDc { id: 5, ip: "2001:0b28:f23f:f005:0000:0000:0000:000a", port: 443 },
];

/// `kBuiltInDcsTest`.
const BUILT_IN_TEST: [BuiltInDc; 3] = [
    BuiltInDc { id: 1, ip: "149.154.175.10", port: 443 },
    BuiltInDc { id: 2, ip: "149.154.167.40", port: 443 },
    BuiltInDc { id: 3, ip: "149.154.175.117", port: 443 },
];

/// `kBuiltInDcsIPv6Test`.
const BUILT_IN_IPV6_TEST: [BuiltInDc; 3] = [
    BuiltInDc { id: 1, ip: "2001:0b28:f23d:f001:0000:0000:0000:000e", port: 443 },
    BuiltInDc { id: 2, ip: "2001:067c:04e8:f002:0000:0000:0000:000e", port: 443 },
    BuiltInDc { id: 3, ip: "2001:0b28:f23d:f003:0000:0000:0000:000e", port: 443 },
];

/// `MTP::DcOptions`.
#[derive(Debug, Clone)]
pub struct DcOptions {
    environment: Environment,
    data: BTreeMap<DcId, Vec<Endpoint>>,
    cdn_dc_ids: BTreeSet<DcId>,
    cdn_public_keys: BTreeMap<DcId, Vec<CdnPublicKey>>,
    immutable: bool,
}

impl DcOptions {
    /// `DcOptions(Environment)` — built-in endpoints for that environment.
    #[must_use]
    pub fn new(environment: Environment) -> Self {
        let mut result = Self {
            environment,
            data: BTreeMap::new(),
            cdn_dc_ids: BTreeSet::new(),
            cdn_public_keys: BTreeMap::new(),
            immutable: false,
        };
        result.construct_from_built_in();
        result
    }

    /// `environment()`.
    #[must_use]
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// `isTestMode()`.
    #[must_use]
    pub fn is_test_mode(&self) -> bool {
        self.environment.is_test()
    }

    /// `ValidateSecret` — the proxy secret shapes a TCP connection accepts.
    ///
    /// See `TcpConnection::Protocol::Create`: an obfuscated secret with the
    /// `0xEE` head, a padded one with the `0xDD` head, a bare 16-byte secret,
    /// or none at all.
    #[must_use]
    pub fn validate_secret(secret: &[u8]) -> bool {
        let first = secret.first().copied();
        (secret.len() >= 21 && first == Some(0xEE))
            || (secret.len() == 17 && first == Some(0xDD))
            || secret.len() == 16
            || secret.is_empty()
    }

    /// `constructFromBuiltIn` — resets to the built-in endpoints.
    pub fn construct_from_built_in(&mut self) {
        self.data.clear();
        let (list, list_v6) = if self.is_test_mode() {
            (&BUILT_IN_TEST[..], &BUILT_IN_IPV6_TEST[..])
        } else {
            (&BUILT_IN[..], &BUILT_IN_IPV6[..])
        };
        for entry in list {
            let flags = Flags::from_raw(STATIC);
            self.apply_one_guarded(entry.id, flags, entry.ip, entry.port, &[]);
        }
        for entry in list_v6 {
            let flags = Flags::from_raw(STATIC).with(IPV6);
            self.apply_one_guarded(entry.id, flags, entry.ip, entry.port, &[]);
        }
        self.compute_cdn_dc_ids();
    }

    /// `constructAddOne` — adds one endpoint, skipping a duplicate.
    pub fn construct_add_one(
        &mut self,
        id: i32,
        flags: Flags,
        ip: &str,
        port: i32,
        secret: &[u8],
    ) {
        self.apply_one_guarded(ids::bare_dc_id(id), flags, ip, port, secret);
        self.compute_cdn_dc_ids();
    }

    fn apply_one_guarded(
        &mut self,
        dc_id: DcId,
        flags: Flags,
        ip: &str,
        port: i32,
        secret: &[u8],
    ) -> bool {
        apply_one_option(&mut self.data, dc_id, flags, ip, port, secret)
    }

    /// `addFromOther` — merges another store into this one.
    ///
    /// Returns the data centers whose endpoints changed, which is what the
    /// C++ code fires on `_changed`.
    pub fn add_from_other(&mut self, other: &mut Self) -> Vec<DcId> {
        if self.immutable || other.data.is_empty() {
            return Vec::new();
        }
        let mut changed = Vec::new();
        for (dc_id, endpoints) in std::mem::take(&mut other.data) {
            let mut any = false;
            for endpoint in endpoints {
                if self.apply_one_guarded(
                    endpoint.id,
                    endpoint.flags,
                    &endpoint.ip,
                    endpoint.port,
                    &endpoint.secret,
                ) {
                    any = true;
                }
            }
            if any {
                changed.push(dc_id);
            }
        }
        for (dc_id, keys) in std::mem::take(&mut other.cdn_public_keys) {
            self.cdn_public_keys.entry(dc_id).or_default().extend(keys);
        }
        self.compute_cdn_dc_ids();
        changed
    }

    /// `setFromList` / `addFromList` — applies a decoded option list.
    ///
    /// Returns the data centers that changed. The list is the TL-free shape of
    /// `MTPDcOption`; `overwrite` clears the current endpoints first.
    pub fn process_from_list(&mut self, options: &[OptionEntry], overwrite: bool) -> Vec<DcId> {
        if options.is_empty() || self.immutable {
            return Vec::new();
        }
        let mut data = if overwrite {
            BTreeMap::new()
        } else {
            self.data.clone()
        };
        for option in options {
            apply_one_option(
                &mut data,
                ids::bare_dc_id(option.dc_id),
                option.flags,
                &option.ip,
                option.port,
                &option.secret,
            );
        }
        let difference = count_options_difference(&self.data, &data);
        if !difference.is_empty() {
            self.data = data;
            self.compute_cdn_dc_ids();
        }
        difference
    }

    /// Whether the endpoints come from an override file and must not be
    /// replaced by server lists.
    #[must_use]
    pub fn is_immutable(&self) -> bool {
        self.immutable
    }

    /// Marks the store as file-overridden (or not), as `loadFromFile` does.
    pub fn set_immutable(&mut self, immutable: bool) {
        self.immutable = immutable;
    }

    /// Every dc id with its endpoints, in dc order.
    #[must_use]
    pub fn all(&self) -> impl Iterator<Item = (&DcId, &Vec<Endpoint>)> {
        self.data.iter()
    }

    /// Every dc id with its endpoints, in dc order.
    #[must_use]
    pub fn endpoints(&self, dc_id: DcId) -> &[Endpoint] {
        self.data.get(&dc_id).map_or(&[], Vec::as_slice)
    }

    /// `configEnumDcIds` — the data centers worth asking for a config: no CDN
    /// ones and no temporary ones, sorted.
    #[must_use]
    pub fn config_enum_dc_ids(&self) -> Vec<DcId> {
        self.data
            .iter()
            .filter(|(dc_id, endpoints)| {
                let front = endpoints.first().is_some_and(|endpoint| !endpoint.flags.is_cdn());
                front && !ids::is_temporary_dc_id(**dc_id)
            })
            .map(|(dc_id, _)| *dc_id)
            .collect()
    }

    /// `dcType` — what a shifted id names.
    #[must_use]
    pub fn dc_type(&self, shifted_dc_id: ShiftedDcId) -> DcType {
        if ids::is_temporary_dc_id(shifted_dc_id) {
            return DcType::Temporary;
        }
        let dc_id = ids::bare_dc_id(shifted_dc_id);
        if self.cdn_dc_ids.contains(&dc_id) {
            return DcType::Cdn;
        }
        if ids::is_media_cluster_dc_id(shifted_dc_id) && self.has_media_only_options_for(dc_id) {
            return DcType::MediaCluster;
        }
        DcType::Regular
    }

    /// `hasMediaOnlyOptionsFor`.
    #[must_use]
    pub fn has_media_only_options_for(&self, dc_id: DcId) -> bool {
        self.data
            .get(&dc_id)
            .is_some_and(|list| list.iter().any(|endpoint| endpoint.flags.has(crate::flags::MEDIA_ONLY)))
    }

    /// Whether any CDN key is known for the data center.
    #[must_use]
    pub fn has_cdn_keys_for_dc(&self, dc_id: DcId) -> bool {
        self.cdn_public_keys.contains_key(&dc_id)
    }

    /// `lookup` — the endpoints a connection may use.
    #[must_use]
    pub fn lookup(&self, dc_id: DcId, dc_type: DcType, through_proxy: bool) -> Variants {
        let Some(endpoints) = self.data.get(&dc_id) else {
            return Variants::default();
        };
        Variants::build(endpoints, dc_type, through_proxy, Self::validate_secret)
    }

    fn compute_cdn_dc_ids(&mut self) {
        self.cdn_dc_ids.clear();
        for (dc_id, endpoints) in &self.data {
            if endpoints
                .first()
                .is_some_and(|endpoint| endpoint.flags.has(CDN))
            {
                self.cdn_dc_ids.insert(ids::bare_dc_id(*dc_id));
            }
        }
    }

    /// `serialize` — the options blob.
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        if self.immutable {
            // Overridden endpoints must not reach the settings file.
            return Self::new(self.environment).serialize();
        }
        let mut out = BlobWriter::new();
        out.write_i32(-VERSION);

        let mut options_count = 0;
        for (dc_id, endpoints) in &self.data {
            if ids::is_temporary_dc_id(*dc_id) {
                continue;
            }
            options_count += endpoints.len() as i32;
        }
        out.write_i32(options_count);
        for (dc_id, endpoints) in &self.data {
            if ids::is_temporary_dc_id(*dc_id) {
                continue;
            }
            for endpoint in endpoints {
                out.write_i32(endpoint.id);
                out.write_i32(endpoint.flags.bits());
                out.write_i32(endpoint.port);
                out.write_i32(endpoint.ip.len() as i32);
                out.write_raw(endpoint.ip.as_bytes());
                out.write_i32(endpoint.secret.len() as i32);
                out.write_raw(&endpoint.secret);
            }
        }

        let keys_count: usize = self.cdn_public_keys.values().map(Vec::len).sum();
        out.write_i32(keys_count as i32);
        for (dc_id, keys) in &self.cdn_public_keys {
            for key in keys {
                out.write_i32(*dc_id);
                out.write_bytes(&key.n);
                out.write_bytes(&key.e);
            }
        }
        out.into_data()
    }

    /// `constructFromSerialized` — replaces the endpoints from a blob.
    ///
    /// Returns `false` for any shape the C++ reader rejects: an unreadable
    /// header, an IP size outside `1..=45`, a secret size outside `0..=32`, a
    /// short payload, or an unreadable CDN key section.
    pub fn construct_from_serialized(&mut self, serialized: &[u8]) -> bool {
        let mut reader = BlobReader::new(serialized);
        let Some(minus_version) = reader.read_i32() else {
            return false;
        };
        let version = if minus_version < 0 { -minus_version } else { 0 };
        let count = if version > 0 {
            match reader.read_i32() {
                Some(count) => count,
                None => return false,
            }
        } else {
            minus_version
        };

        self.data.clear();
        for _ in 0..count {
            let (Some(id), Some(flags), Some(port), Some(ip_size)) = (
                reader.read_i32(),
                reader.read_i32(),
                reader.read_i32(),
                reader.read_i32(),
            ) else {
                return false;
            };
            if ip_size <= 0 || ip_size > MAX_IP_SIZE {
                return false;
            }
            let Some(ip_bytes) = reader.read_raw(ip_size as usize) else {
                return false;
            };
            let Ok(ip) = std::str::from_utf8(ip_bytes) else {
                // The C++ reader keeps raw bytes; an address is ASCII by
                // construction, so a non-UTF-8 one is bad data.
                return false;
            };

            let mut secret = Vec::new();
            if version > 0 {
                let Some(secret_size) = reader.read_i32() else {
                    return false;
                };
                if !(0..=MAX_SECRET_SIZE).contains(&secret_size) {
                    return false;
                } else if secret_size > 0 {
                    match reader.read_raw(secret_size as usize) {
                        Some(bytes) => secret = bytes.to_vec(),
                        None => return false,
                    }
                }
            }
            if !reader.is_ok() {
                return false;
            }
            self.apply_one_guarded(
                ids::bare_dc_id(id),
                Flags::from_raw(flags),
                ip,
                port,
                &secret,
            );
        }

        if !reader.at_end() && version > 1 {
            let Some(keys_count) = reader.read_i32() else {
                return false;
            };
            for _ in 0..keys_count {
                let (Some(dc_id), n, e) = (
                    reader.read_i32(),
                    read_serialized_bytes(&mut reader),
                    read_serialized_bytes(&mut reader),
                ) else {
                    return false;
                };
                let (Some(n), Some(e)) = (n, e) else {
                    return false;
                };
                self.cdn_public_keys
                    .entry(dc_id)
                    .or_default()
                    .push(CdnPublicKey { n, e });
            }
        }
        self.compute_cdn_dc_ids();
        true
    }
}

/// `Serialize::bytes(n)` on the read side: a `qint32` length, then the bytes.
fn read_serialized_bytes(reader: &mut BlobReader<'_>) -> Option<Vec<u8>> {
    let size = reader.read_i32()?;
    if size < 0 {
        return None;
    }
    reader.read_raw(size as usize).map(<[u8]>::to_vec)
}

/// `DcOptions::ApplyOneOption` — appends unless the same ip and port exist.
fn apply_one_option(
    data: &mut BTreeMap<DcId, Vec<Endpoint>>,
    dc_id: DcId,
    flags: Flags,
    ip: &str,
    port: i32,
    secret: &[u8],
) -> bool {
    match data.get_mut(&dc_id) {
        Some(list) => {
            if list
                .iter()
                .any(|endpoint| endpoint.ip == ip && endpoint.port == port)
            {
                return false;
            }
            list.push(Endpoint::new(dc_id, flags, ip.to_string(), port, secret.to_vec()));
        }
        None => {
            data.insert(
                dc_id,
                vec![Endpoint::new(dc_id, flags, ip.to_string(), port, secret.to_vec())],
            );
        }
    }
    true
}

/// `DcOptions::CountOptionsDifference` — the ids whose endpoint sets differ.
fn count_options_difference(
    a: &BTreeMap<DcId, Vec<Endpoint>>,
    b: &BTreeMap<DcId, Vec<Endpoint>>,
) -> Vec<DcId> {
    let equal = |m: &Vec<Endpoint>, n: &Vec<Endpoint>| {
        if m.len() != n.len() {
            return false;
        }
        m.iter().all(|endpoint| {
            n.iter()
                .any(|other| other.ip == endpoint.ip && other.port == endpoint.port)
        })
    };
    let mut result = Vec::new();
    let mut left = a.iter().peekable();
    let mut right = b.iter().peekable();
    while left.peek().is_some() || right.peek().is_some() {
        match (left.peek(), right.peek()) {
            (Some((a_id, a_list)), Some((b_id, b_list))) => {
                if a_id < b_id {
                    result.push(**a_id);
                    left.next();
                } else if b_id < a_id {
                    result.push(**b_id);
                    right.next();
                } else {
                    if !equal(a_list, b_list) {
                        result.push(**a_id);
                    }
                    left.next();
                    right.next();
                }
            }
            (Some((a_id, _)), None) => {
                result.push(**a_id);
                left.next();
            }
            (None, Some((b_id, _))) => {
                result.push(**b_id);
                right.next();
            }
            (None, None) => break,
        }
    }
    result
}