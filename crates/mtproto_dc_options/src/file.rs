//! The `.tdesktop-endpoints` text file: a developer override of the endpoints.
//!
//! Source of truth: `DcOptions::loadFromFile` and `DcOptions::writeToFile` in
//! `mtproto/mtproto_dc_options.cpp`.
//!
//! One endpoint per line: `dcId host port [tcpo_only] [media_only]`, `#` starts
//! a comment. A file that overrides the endpoints makes the store *immutable*:
//! it no longer accepts server lists and serializes the built-in options
//! instead of its own.

use std::net::IpAddr;
use std::path::Path;

use crate::flags::{Flags, IPV6, MEDIA_ONLY, TCPO_ONLY};
use crate::ids::DC_SHIFT;
use crate::options::DcOptions;

impl DcOptions {
    /// `loadFromFile` — replaces the endpoints from a text file.
    ///
    /// Returns `false` on an unreadable file, a line with too few components, a
    /// dc id outside `1..DC_SHIFT`, an address that does not parse, a
    /// non-positive port, an unknown option word, or a file with no endpoints.
    pub fn load_from_file(&mut self, path: &Path) -> bool {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return false;
        };
        let mut options = Vec::new();
        for line in contents.lines() {
            let mut components = line.split_whitespace();
            let Some(first) = components.next() else {
                continue;
            };
            if first.starts_with('#') {
                continue;
            }
            let mut parts = vec![first];
            parts.extend(components);
            if parts.len() < 3 {
                return false;
            }
            let Ok(dc_id) = parts[0].parse::<i32>() else {
                return false;
            };
            let ip = parts[1];
            let Ok(port) = parts[2].parse::<i32>() else {
                return false;
            };
            let Ok(address) = ip.parse::<IpAddr>() else {
                return false;
            };
            if dc_id <= 0 || dc_id >= DC_SHIFT || port <= 0 {
                return false;
            }
            let mut flags = Flags::none();
            if address.is_ipv6() {
                flags = flags.with(IPV6);
            }
            for option in &parts[3..] {
                if option.starts_with('#') {
                    break;
                } else if *option == "tcpo_only" {
                    flags = flags.with(TCPO_ONLY);
                } else if *option == "media_only" {
                    flags = flags.with(MEDIA_ONLY);
                } else {
                    return false;
                }
            }
            options.push(crate::options::OptionEntry {
                dc_id,
                flags,
                ip: ip.to_string(),
                port,
                secret: Vec::new(),
            });
        }
        if options.is_empty() {
            return false;
        }

        self.set_immutable(false);
        self.process_from_list(&options, true);
        self.set_immutable(true);
        true
    }

    /// `writeToFile` — dumps the endpoints in the same text form.
    pub fn write_to_file(&self, path: &Path) -> bool {
        let mut out = String::new();
        for (dc_id, endpoints) in self.all() {
            for endpoint in endpoints {
                out.push_str(&format!("{} {} {}", dc_id, endpoint.ip, endpoint.port));
                if endpoint.flags.has(TCPO_ONLY) {
                    out.push_str(" tcpo_only");
                }
                if endpoint.flags.has(MEDIA_ONLY) {
                    out.push_str(" media_only");
                }
                out.push('\n');
            }
        }
        std::fs::write(path, out).is_ok()
    }
}
