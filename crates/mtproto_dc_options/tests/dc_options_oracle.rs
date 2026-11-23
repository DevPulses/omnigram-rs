//! Oracle for the data center options: the Python writer produces the blob,
//! the port must parse it, reproduce it byte for byte and answer lookups the
//! same way the C++ store does.

mod vectors;

use mtproto_dc_options::flags::{CDN, Flags, IPV6, MEDIA_ONLY, STATIC, TCPO_ONLY};
use mtproto_dc_options::ids::{
    bare_dc_id, download_dc_id, is_media_cluster_dc_id, is_temporary_dc_id, shift_dc_id,
    DcType, DC_SHIFT,
};
use mtproto_dc_options::lookup::{HTTP, IPV4, IPV6_INDEX, TCP};
use mtproto_dc_options::{DcOptions, Environment};

fn parse(hexed: &str) -> DcOptions {
    let mut options = DcOptions::new(Environment::Production);
    assert!(
        options.construct_from_serialized(&hex::decode(hexed).unwrap()),
        "the oracle blob parses"
    );
    options
}

#[test]
fn built_in_endpoints_match_the_oracle() {
    let production = DcOptions::new(Environment::Production);
    assert_eq!(
        hex::encode(production.serialize()),
        vectors::BUILT_IN_BLOB_HEX
    );
    assert!(!production.is_test_mode());
    assert_eq!(production.endpoints(1).len(), 2);
    assert_eq!(production.endpoints(2).len(), 3);
    assert_eq!(production.endpoints(2)[0].ip, "149.154.167.51");
    assert_eq!(production.endpoints(2)[2].ip, "2001:067c:04e8:f002:0000:0000:0000:000a");
    assert!(production.endpoints(2)[2].is_ipv6());
    assert!(production.endpoints(2)[2].flags.has(STATIC));

    let test = DcOptions::new(Environment::Test);
    assert_eq!(hex::encode(test.serialize()), vectors::BUILT_IN_TEST_BLOB_HEX);
    assert!(test.is_test_mode());
    assert_eq!(test.endpoints(3)[0].ip, "149.154.175.117");
}

#[test]
fn fixture_parses_and_reserializes_byte_for_byte() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    assert_eq!(
        hex::encode(options.serialize()),
        vectors::FIXTURE_BLOB_HEX,
        "reading then writing the blob must not change a byte"
    );

    assert_eq!(options.endpoints(2).len(), 3);
    assert_eq!(options.endpoints(3).len(), 1);
    assert_eq!(options.endpoints(4).len(), 2);
    assert_eq!(options.endpoints(5)[0].port, 8443);
    assert_eq!(
        hex::encode(&options.endpoints(3)[0].secret),
        vectors::FIXTURE_SECRET_HEX
    );
    assert!(options.endpoints(3)[0].flags.has(TCPO_ONLY));
    assert!(options.has_cdn_keys_for_dc(4));
}

#[test]
fn duplicate_endpoints_are_dropped() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    let before = options.endpoints(2).len();
    let blob = {
        // The same dp again, twice, before reading: `ApplyOneOption` dedupes by
        // ip and port.
        let mut options = options.clone();
        options.construct_add_one(2, Flags::from_raw(STATIC), "149.154.167.51", 443, &[]);
        options.serialize()
    };
    let reparsed = {
        let mut copy = DcOptions::new(Environment::Production);
        assert!(copy.construct_from_serialized(&blob));
        copy
    };
    assert_eq!(reparsed.endpoints(2).len(), before);
}

#[test]
fn lookup_splits_by_address_and_protocol() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    let variants = options.lookup(2, DcType::Regular, false);
    assert_eq!(variants.data[IPV4][TCP].len(), 2);
    assert_eq!(variants.data[IPV4][HTTP].len(), 2);
    assert_eq!(variants.data[IPV6_INDEX][TCP].len(), 1);
    assert_eq!(variants.data[IPV6_INDEX][HTTP].len(), 1);

    // Through a proxy only the static endpoints stay.
    let proxied = options.lookup(2, DcType::Regular, true);
    assert_eq!(proxied.data[IPV4][TCP].len(), 1);
    assert_eq!(proxied.data[IPV4][TCP][0].ip, "149.154.167.51");
}

#[test]
fn tcpo_only_and_secret_endpoints_have_no_http_variant() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    let variants = options.lookup(3, DcType::Regular, false);
    assert_eq!(variants.data[IPV4][TCP].len(), 1);
    assert_eq!(variants.data[IPV4][HTTP].len(), 0);
    assert!(DcOptions::validate_secret(&[0xEE; 21]));
    assert!(DcOptions::validate_secret(&[0xDD; 17]));
    assert!(DcOptions::validate_secret(&[0; 16]));
    assert!(DcOptions::validate_secret(&[]));
    assert!(!DcOptions::validate_secret(&[0; 20]));
    assert!(!DcOptions::validate_secret(&[0xEE; 20]));
    // A bare 16-byte secret is taken whatever it starts with.
    assert!(DcOptions::validate_secret(&[0xDD; 16]));
}

#[test]
fn media_only_endpoints_need_a_media_cluster_lookup() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    // A regular lookup drops the media-only endpoint but keeps the CDN one:
    // only a CDN lookup requires the `f_cdn` flag, no lookup forbids it.
    let regular = options.lookup(4, DcType::Regular, false);
    assert_eq!(regular.data[IPV4][TCP].len(), 1);
    assert!(regular.data[IPV4][TCP][0].flags.has(CDN));

    let cluster = options.lookup(4, DcType::MediaCluster, false);
    assert_eq!(cluster.data[IPV4][TCP].len(), 1);
    assert_eq!(cluster.data[IPV4][TCP][0].ip, "149.154.167.91");
    assert!(options.has_media_only_options_for(4));
    assert!(!options.has_media_only_options_for(2));
}

#[test]
fn cdn_lookup_keeps_only_cdn_endpoints() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    let cdn = options.lookup(4, DcType::Cdn, false);
    assert_eq!(cdn.data[IPV4][TCP].len(), 1);
    assert!(cdn.data[IPV4][TCP][0].flags.has(CDN));
}

#[test]
fn dc_type_follows_the_flag_of_the_first_endpoint() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    // dc 2 has a static endpoint first, dc 4 starts with a media-only one, so
    // neither counts as a CDN data center: only the first endpoint is checked.
    assert_eq!(options.dc_type(2), DcType::Regular);
    assert_eq!(options.dc_type(4), DcType::Regular);
    assert_eq!(
        options.dc_type(download_dc_id(4, 0)),
        DcType::MediaCluster
    );
    assert_eq!(options.dc_type(3), DcType::Regular);

    // A data center whose first endpoint is a CDN one is a CDN data center.
    let mut cdn_first = DcOptions::new(Environment::Production);
    assert!(cdn_first.construct_from_serialized(&hex::decode(vectors::FIXTURE_BLOB_HEX).unwrap()));
    cdn_first.construct_add_one(9, Flags::from_raw(CDN), "10.0.0.1", 443, &[]);
    assert_eq!(cdn_first.dc_type(9), DcType::Cdn);

    assert!(is_temporary_dc_id(1000));
    assert_eq!(options.dc_type(1000), DcType::Temporary);
    assert_eq!(options.dc_type(1001 + DC_SHIFT), DcType::Temporary);
    assert_eq!(bare_dc_id(1001 + DC_SHIFT), 1001);
    assert!(is_media_cluster_dc_id(shift_dc_id(1, 0x06)));
    assert_eq!(shift_dc_id(2, 1), 10_002);
}

#[test]
fn config_enum_lists_regular_data_centers() {
    let options = parse(vectors::FIXTURE_BLOB_HEX);
    assert_eq!(options.config_enum_dc_ids(), vec![2, 3, 4, 5]);
}

#[test]
fn reader_refuses_bad_blobs() {
    let mut options = DcOptions::new(Environment::Production);
    assert!(!options.construct_from_serialized(&hex::decode(vectors::BAD_IP_ZERO_HEX).unwrap()));
    assert!(!options.construct_from_serialized(&hex::decode(vectors::BAD_IP_LONG_HEX).unwrap()));
    assert!(
        !options.construct_from_serialized(&hex::decode(vectors::BAD_SECRET_LONG_HEX).unwrap())
    );
    assert!(!options.construct_from_serialized(&hex::decode(vectors::TRUNCATED_HEX).unwrap()));
    assert!(!options.construct_from_serialized(&[]));
}

#[test]
fn version_zero_blob_has_no_secret_section() {
    let options = parse(vectors::VERSION_ZERO_BLOB_HEX);
    assert_eq!(options.endpoints(2).len(), 1);
    assert_eq!(options.endpoints(2)[0].ip, "1.2.3.4");
    assert!(options.endpoints(2)[0].secret.is_empty());
    assert!(!options.endpoints(2)[0].flags.has(IPV6));
    assert!(!options.endpoints(2)[0].flags.has(MEDIA_ONLY));
}

#[test]
fn merging_keeps_new_endpoints_and_reports_ids() {
    let mut base = DcOptions::new(Environment::Production);
    let mut extra = DcOptions::new(Environment::Production);
    assert!(extra.construct_from_serialized(&hex::decode(vectors::FIXTURE_BLOB_HEX).unwrap()));

    let changed = base.add_from_other(&mut extra);
    assert!(!changed.is_empty());
    assert!(changed.contains(&2));
    // dc 2 already had one of the fixture endpoints, so both are present.
    assert!(base.endpoints(2).len() >= 3);
    assert!(base.has_cdn_keys_for_dc(4));

    // Merging the same store twice changes nothing.
    let mut again = DcOptions::new(Environment::Production);
    assert!(again.construct_from_serialized(&hex::decode(vectors::FIXTURE_BLOB_HEX).unwrap()));
    assert!(base.add_from_other(&mut again).is_empty());
}

#[test]
fn process_from_list_reports_the_changed_data_centers() {
    let mut options = DcOptions::new(Environment::Production);
    let entry = mtproto_dc_options::OptionEntry {
        dc_id: 7,
        flags: Flags::from_raw(STATIC),
        ip: "10.1.2.3".to_string(),
        port: 443,
        secret: Vec::new(),
    };
    assert_eq!(options.process_from_list(&[entry.clone()], false), vec![7]);
    assert_eq!(options.endpoints(7).len(), 1);
    // The same entry again is no change at all.
    assert!(options.process_from_list(&[entry], false).is_empty());
}

#[test]
fn endpoint_file_round_trips_and_freezes_the_store() {
    let dir = std::env::temp_dir().join(format!(
        "mtproto_dc_options_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("endpoints.txt");

    let source = parse(vectors::FIXTURE_BLOB_HEX);
    assert!(source.write_to_file(&path));

    let mut loaded = DcOptions::new(Environment::Production);
    assert!(loaded.load_from_file(&path));
    assert!(loaded.is_immutable());
    assert_eq!(loaded.endpoints(3)[0].secret, Vec::new());
    assert!(loaded.endpoints(3)[0].flags.has(TCPO_ONLY));
    assert!(loaded.endpoints(4)[0].flags.has(MEDIA_ONLY));
    assert!(loaded.endpoints(2)[2].flags.has(IPV6));

    // An override file makes the store refuse server lists and serialize the
    // built-in endpoints instead of its own.
    assert!(
        loaded
            .process_from_list(
                &[mtproto_dc_options::OptionEntry {
                    dc_id: 8,
                    flags: Flags::none(),
                    ip: "10.9.9.9".to_string(),
                    port: 443,
                    secret: Vec::new(),
                }],
                false
            )
            .is_empty()
    );
    assert_eq!(hex::encode(loaded.serialize()), vectors::BUILT_IN_BLOB_HEX);

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("3 149.154.175.100 443 tcpo_only\n"));
    assert!(text.contains("4 149.154.167.91 443 media_only\n"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn endpoint_file_rejects_bad_lines() {
    let dir = std::env::temp_dir().join(format!(
        "mtproto_dc_options_bad_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let cases = [
        ("2 not-an-ip 443\n", "an address that does not parse"),
        ("0 1.2.3.4 443\n", "a dc id of zero"),
        ("2 1.2.3.4 0\n", "a port of zero"),
        ("2 1.2.3.4\n", "too few components"),
        ("2 1.2.3.4 443 wat\n", "an unknown option word"),
    ];
    for (index, (contents, what)) in cases.iter().enumerate() {
        let path = dir.join(format!("bad{index}.txt"));
        std::fs::write(&path, contents).unwrap();
        let mut options = DcOptions::new(Environment::Production);
        assert!(!options.load_from_file(&path), "must reject {what}");
    }

    // A file of comments only is not an override either.
    let empty = dir.join("empty.txt");
    std::fs::write(&empty, "# nothing here\n").unwrap();
    let mut options = DcOptions::new(Environment::Production);
    assert!(!options.load_from_file(&empty));
    assert!(!options.is_immutable());

    let _ = std::fs::remove_dir_all(&dir);
}