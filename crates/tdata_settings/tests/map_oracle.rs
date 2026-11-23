//! Oracle for the settings map: the Python writer emits the bytes, the port
//! must read them back into the same context and the same effect sequence.

mod vectors;

use tdata_settings::apply::{apply_read_fallback_config, FallbackConfigAction};
use tdata_settings::blocks as id;
use tdata_settings::context::{LegacyDcEndpoint, LegacyDcOptions};
use tdata_settings::map::read_settings_map;
use tdata_settings::sink::{
    CallSettings, LegacyEffect, ProxyData, ProxySettings, ProxyType, RecordingSink, WindowPosition,
};
use tdata_settings::stream::{StreamReader, StreamWriter};
use tdata_settings::write::{write_mtp_data, write_settings_map, CacheLimits, SettingsSnapshot};
use tdata_settings::ReadSettingsContext;

fn read_map(hexed: &str) -> (ReadSettingsContext, RecordingSink, bool) {
    let bytes = hex::decode(hexed).expect("vector is hex");
    let mut reader = StreamReader::new(&bytes);
    let mut context = ReadSettingsContext::new();
    let mut sink = RecordingSink::new();
    let ok = read_settings_map(&mut reader, vectors::VERSION, &mut context, &mut sink);
    (context, sink, ok)
}

#[test]
fn context_matches_the_oracle() {
    let (context, _, ok) = read_map(vectors::MAP_HEX);
    assert!(ok, "the oracle map reads");

    assert_eq!(
        context.session_settings.serialized,
        Some((
            hex::decode(vectors::SESSION_SETTINGS_HEX).unwrap(),
            vectors::VERSION
        ))
    );
    assert_eq!(context.cache_total_size_limit, 100 * 1024 * 1024);
    assert_eq!(context.cache_total_time_limit, 3600);
    assert_eq!(context.cache_big_file_total_size_limit, 200 * 1024 * 1024);
    assert_eq!(context.cache_big_file_total_time_limit, 0);

    assert_eq!(context.theme_key_day, 0x1111_1111_1111_1111);
    assert_eq!(context.theme_key_night, 0x2222_2222_2222_2222);
    assert!(context.night_mode, "the theme block said night mode");
    assert!(context.tile_day);
    assert!(!context.tile_night, "the newer block overrode the legacy one");
    assert!(context.tile_read);

    assert_eq!(context.background_key_day, 0x3333_3333_3333_3333);
    assert_eq!(context.background_key_night, 0x4444_4444_4444_4444);
    assert!(context.background_keys_read);
    assert_eq!(context.lang_pack_key, 0x5555_5555_5555_5555);
    assert_eq!(context.languages_key, 0x6666_6666_6666_6666);

    assert_eq!(
        context.mtp_authorization,
        hex::decode(vectors::MTP_AUTHORIZATION_HEX).unwrap()
    );
    assert!(context.mtp_legacy_keys.is_empty());

    assert_eq!(
        context.fallback_config_legacy_dc_options,
        LegacyDcOptions::Endpoints(vec![LegacyDcEndpoint {
            dc_id: 2,
            flags: 0x10,
            ip: "149.154.167.50".to_string(),
            port: 443,
        }])
    );
    assert_eq!(context.fallback_config_legacy_chat_size_max, 100);
    assert_eq!(context.fallback_config_legacy_stickers_faved_limit, 200);
    assert_eq!(
        context.fallback_config_legacy_txt_domain_string,
        "telegram.org"
    );
    assert!(context.fallback_config.is_empty());

    assert_eq!(context.session_settings.dialogs_filters_enabled, Some(true));
    assert_eq!(
        context.session_settings.hidden_pinned_messages,
        vec![(100, 5), (200, 7)]
    );
    assert!(context.legacy_read);
}

#[test]
fn effects_match_the_oracle() {
    let (_, sink, ok) = read_map(vectors::MAP_HEX);
    assert!(ok);

    let tcp_proxy = ProxyData {
        kind: Some(ProxyType::Socks5),
        host: "proxy.local".to_string(),
        port: 1080,
        user: "user".to_string(),
        password: "pass".to_string(),
    };
    let expected = vec![
        LegacyEffect::ApplicationSettings {
            serialized: hex::decode(vectors::APP_SETTINGS_HEX).unwrap(),
            version: vectors::VERSION,
        },
        LegacyEffect::ThemeNightMode(true),
        LegacyEffect::RecentStickersPreload(vec![(11, 3), (22, 7)]),
        LegacyEffect::AutoStart(true),
        LegacyEffect::Scale(150),
        LegacyEffect::DialogLastPath("/home/loth".to_string()),
        LegacyEffect::PowerSaving(3),
        LegacyEffect::SoundNotify(false),
        LegacyEffect::FlashBounceNotify(true),
        LegacyEffect::WorkMode(0),
        LegacyEffect::NotifyView(2),
        LegacyEffect::DialogsWidthRatio(0.75),
        LegacyEffect::WindowPosition(WindowPosition {
            x: 10,
            y: 20,
            w: 800,
            h: 600,
            moncrc: 0xABCD,
            maximized: false,
        }),
        LegacyEffect::SendSubmitWay(1),
        LegacyEffect::CallSettings(CallSettings {
            output_device_id: "speakers".to_string(),
            output_volume: 100,
            input_device_id: "microphone".to_string(),
            input_volume: 50,
            ducking_enabled: 1,
        }),
        LegacyEffect::ProxySelection {
            list: vec![tcp_proxy.clone()],
            selected: Some(tcp_proxy),
            settings: ProxySettings::Enabled,
            use_for_calls: None,
            try_ipv6: None,
        },
        LegacyEffect::SongVolume(1.0),
        LegacyEffect::VideoVolume(1.0),
        LegacyEffect::LegacyRecentEmojiById(vec![("smile".to_string(), 4)]),
        LegacyEffect::LegacyEmojiVariants(vec![(0x1F600, 2)]),
        LegacyEffect::LegacyEmojiVariantsById(vec![("smile".to_string(), 1)]),
    ];
    assert_eq!(sink.effects.len(), expected.len());
    for (index, (read, want)) in sink.effects.iter().zip(expected.iter()).enumerate() {
        assert_eq!(read, want, "effect {index} differs");
    }
}

#[test]
fn writer_reproduces_the_oracle_map() {
    let mut out = StreamWriter::new();

    // dbiApplicationSettings
    out.write_u32(id::DBI_APPLICATION_SETTINGS);
    out.write_byte_array(&hex::decode(vectors::APP_SETTINGS_HEX).unwrap());
    // dbiSessionSettings
    out.write_u32(id::DBI_SESSION_SETTINGS);
    out.write_byte_array(&hex::decode(vectors::SESSION_SETTINGS_HEX).unwrap());
    // dbiCacheSettings
    out.write_u32(id::DBI_CACHE_SETTINGS);
    out.write_i64(100 * 1024 * 1024);
    out.write_i32(3600);
    out.write_i64(200 * 1024 * 1024);
    out.write_i32(0);
    // dbiTileBackgroundOld
    out.write_u32(id::DBI_TILE_BACKGROUND_OLD);
    out.write_i32(1);
    // dbiThemeKey
    out.write_u32(id::DBI_THEME_KEY);
    out.write_u64(0x1111_1111_1111_1111);
    out.write_u64(0x2222_2222_2222_2222);
    out.write_u32(1);
    // dbiTileBackground
    out.write_u32(id::DBI_TILE_BACKGROUND);
    out.write_i32(2);
    out.write_i32(0);
    // dbiBackgroundKey
    out.write_u32(id::DBI_BACKGROUND_KEY);
    out.write_u64(0x3333_3333_3333_3333);
    out.write_u64(0x4444_4444_4444_4444);
    // dbiLangPackKey, dbiLanguagesKey
    out.write_u32(id::DBI_LANG_PACK_KEY);
    out.write_u64(0x5555_5555_5555_5555);
    out.write_u32(id::DBI_LANGUAGES_KEY);
    out.write_u64(0x6666_6666_6666_6666);
    // dbiMtpAuthorization
    out.write_u32(id::DBI_MTP_AUTHORIZATION);
    out.write_byte_array(&hex::decode(vectors::MTP_AUTHORIZATION_HEX).unwrap());
    // dbiRecentStickers
    out.write_u32(id::DBI_RECENT_STICKERS);
    out.write_pairs_u64_u16(&[(11, 3), (22, 7)]);
    // dbiAutoStart, dbiScalePercent, dbiDialogLastPath, dbiPowerSaving
    out.write_u32(id::DBI_AUTO_START);
    out.write_i32(1);
    out.write_u32(id::DBI_SCALE_PERCENT);
    out.write_i32(150);
    out.write_u32(id::DBI_DIALOG_LAST_PATH);
    out.write_string("/home/loth");
    out.write_u32(id::DBI_POWER_SAVING);
    out.write_i32(3);
    // dbiSoundFlashBounceNotifyOld, dbiWorkModeOld, dbiNotifyViewOld
    out.write_u32(id::DBI_SOUND_FLASH_BOUNCE_NOTIFY_OLD);
    out.write_i32(0x00);
    out.write_u32(id::DBI_WORK_MODE_OLD);
    out.write_i32(5);
    out.write_u32(id::DBI_NOTIFY_VIEW_OLD);
    out.write_i32(2);
    // dbiDialogsWidthRatioOld
    out.write_u32(id::DBI_DIALOGS_WIDTH_RATIO_OLD);
    out.write_i32(750_000);
    // dbiDcOptionOld
    out.write_u32(id::DBI_DC_OPTION_OLD);
    out.write_u32(2);
    out.write_i32(0x10);
    out.write_string("149.154.167.50");
    out.write_u32(443);
    // dbiChatSizeMaxOld, dbiStickersFavedLimitOld, dbiTxtDomainStringOld
    out.write_u32(id::DBI_CHAT_SIZE_MAX_OLD);
    out.write_i32(100);
    out.write_u32(id::DBI_STICKERS_FAVED_LIMIT_OLD);
    out.write_i32(200);
    out.write_u32(id::DBI_TXT_DOMAIN_STRING_OLD);
    out.write_string("telegram.org");
    // dbiFallbackProductionConfig (empty)
    out.write_u32(id::DBI_FALLBACK_PRODUCTION_CONFIG);
    out.write_byte_array(&[]);
    // dbiWindowPositionOld
    out.write_u32(id::DBI_WINDOW_POSITION_OLD);
    out.write_i32(10);
    out.write_i32(20);
    out.write_i32(800);
    out.write_i32(600);
    out.write_u64(0xABCD);
    out.write_bool(false);
    // dbiSendKeyOld
    out.write_u32(id::DBI_SEND_KEY_OLD);
    out.write_i32(1);
    // dbiCallSettingsOld
    out.write_u32(id::DBI_CALL_SETTINGS_OLD);
    out.write_byte_array(&hex::decode(vectors::CALL_SETTINGS_HEX).unwrap());
    // dbiConnectionTypeOldOld
    out.write_u32(id::DBI_CONNECTION_TYPE_OLD_OLD);
    out.write_i32(3);
    out.write_string("proxy.local");
    out.write_i32(1080);
    out.write_string("user");
    out.write_string("pass");
    // dbiHiddenPinnedMessagesOld
    out.write_u32(id::DBI_HIDDEN_PINNED_MESSAGES_OLD);
    out.write_u32(2);
    out.write_u64(100);
    out.write_i32(5);
    out.write_u64(200);
    out.write_i32(7);
    // dbiDialogsFiltersOld
    out.write_u32(id::DBI_DIALOGS_FILTERS_OLD);
    out.write_i32(1);
    // dbiSongVolumeOld, dbiVideoVolumeOld
    out.write_u32(id::DBI_SONG_VOLUME_OLD);
    out.write_i32(1_000_000);
    out.write_u32(id::DBI_VIDEO_VOLUME_OLD);
    out.write_i32(2_000_000);
    // dbiRecentEmojiOld
    out.write_u32(id::DBI_RECENT_EMOJI_OLD);
    out.write_pairs_string_u16(&[("smile".to_string(), 4)]);
    // dbiEmojiVariantsOldOld
    out.write_u32(id::DBI_EMOJI_VARIANTS_OLD_OLD);
    out.write_u32(1);
    out.write_u32(0x1F600);
    out.write_u64(2);
    // dbiEmojiVariantsOld
    out.write_u32(id::DBI_EMOJI_VARIANTS_OLD);
    out.write_u32(1);
    out.write_string("smile");
    out.write_i32(1);

    assert_eq!(hex::encode(out.data()), vectors::MAP_HEX);
}

#[test]
fn unknown_block_fails_the_read() {
    let (_, _, ok) = read_map(vectors::MAP_UNKNOWN_BLOCK_HEX);
    assert!(!ok, "an unknown block id fails the whole read");
}

#[test]
fn truncated_block_fails_the_read() {
    let (_, _, ok) = read_map(vectors::MAP_TRUNCATED_HEX);
    assert!(!ok, "a short payload fails the block");
}

#[test]
fn invalid_legacy_send_key_fails_the_read() {
    let (_, _, ok) = read_map(vectors::MAP_BAD_SEND_KEY_HEX);
    assert!(!ok, "dbiSendKeyOld accepts only Enter and Ctrl+Enter");
}

#[test]
fn cache_limits_below_the_minimum_fail_the_read() {
    let (_, _, ok) = read_map(vectors::MAP_SMALL_CACHE_HEX);
    assert!(!ok);
}

#[test]
fn no_time_limit_sentinels_become_zero() {
    let (context, _, ok) = read_map(vectors::MAP_NO_TIME_LIMIT_HEX);
    assert!(ok);
    assert_eq!(context.cache_total_size_limit, 64 * 1024 * 1024);
    assert_eq!(context.cache_total_time_limit, 0);
    assert_eq!(context.cache_big_file_total_time_limit, 0);
    assert!(tdata_settings::no_time_limit(0));
    assert!(tdata_settings::no_time_limit(i32::MAX));
    assert!(tdata_settings::no_time_limit(-1));
    assert!(!tdata_settings::no_time_limit(60));
}

#[test]
fn window_position_keeps_negative_coordinates() {
    let (_, sink, ok) = read_map(vectors::MAP_WINDOW_MAXIMIZED_HEX);
    assert!(ok);
    assert_eq!(
        sink.effects,
        vec![LegacyEffect::WindowPosition(WindowPosition {
            x: -5,
            y: -6,
            w: 1280,
            h: 720,
            moncrc: 0xFEED,
            maximized: true,
        })]
    );
}

#[test]
fn proxy_list_selects_the_indexed_entry() {
    let (_, sink, ok) = read_map(vectors::MAP_PROXY_LIST_HEX);
    assert!(ok);
    let [LegacyEffect::ProxySelection {
        list,
        selected,
        settings,
        use_for_calls,
        ..
    }] = sink.effects.as_slice()
    else {
        panic!("one proxy effect");
    };
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].host, "first.local");
    assert_eq!(list[1].host, "second.local");
    assert_eq!(list[1].kind, Some(ProxyType::Socks5));
    assert_eq!(selected.as_ref().map(|proxy| proxy.host.as_str()), Some("second.local"));
    assert_eq!(*settings, ProxySettings::Enabled);
    assert_eq!(*use_for_calls, Some(true));
}

#[test]
fn oldest_proxy_list_derives_the_settings() {
    let (_, sink, ok) = read_map(vectors::MAP_PROXY_LIST_OLD_HEX);
    assert!(ok);
    let [LegacyEffect::ProxySelection {
        list,
        selected,
        settings,
        use_for_calls,
        ..
    }] = sink.effects.as_slice()
    else {
        panic!("one proxy effect");
    };
    assert_eq!(list.len(), 1);
    assert_eq!(selected.as_ref().map(|proxy| proxy.host.as_str()), Some("only.local"));
    assert_eq!(*settings, ProxySettings::Enabled);
    assert_eq!(*use_for_calls, Some(false));
}

#[test]
fn fallback_config_merges_legacy_pieces() {
    let (mut context, _, ok) = read_map(vectors::MAP_HEX);
    assert!(ok);
    let action = apply_read_fallback_config(&mut context);
    let FallbackConfigAction::Legacy(config) = action else {
        panic!("no serialized config in this map");
    };
    assert_eq!(
        config.dc_options,
        LegacyDcOptions::Endpoints(vec![LegacyDcEndpoint {
            dc_id: 2,
            flags: 0x10,
            ip: "149.154.167.50".to_string(),
            port: 443,
        }])
    );
    assert_eq!(config.chat_size_max, Some(100));
    // The saved-gifs limit is read but never applied, and so is the faving
    // limit; only the recent-stickers limit travels on.
    assert_eq!(config.stickers_recent_limit, None);
    assert_eq!(config.megagroup_size_max, None);
    assert_eq!(config.txt_domain_string.as_deref(), Some("telegram.org"));
    // The pieces were moved out of the context.
    assert_eq!(context.fallback_config_legacy_dc_options, LegacyDcOptions::Empty);
}

#[test]
fn serialized_config_replaces_the_legacy_pieces() {
    let (mut context, _, ok) = read_map(vectors::MAP_WITH_CONFIG_HEX);
    assert!(ok);
    assert_eq!(context.fallback_config, b"fallback-config".to_vec());
    assert_eq!(
        apply_read_fallback_config(&mut context),
        FallbackConfigAction::Construct(b"fallback-config".to_vec())
    );
}

#[test]
fn written_settings_map_matches_the_oracle() {
    let snapshot = SettingsSnapshot {
        cache: CacheLimits {
            total_size: 150 * 1024 * 1024,
            total_time: 7200,
            big_file_total_size: 300 * 1024 * 1024,
            big_file_total_time: 60,
        },
        session_settings: hex::decode(vectors::WRITTEN_SESSION_SETTINGS_HEX).unwrap(),
        recent_stickers: vec![(5, 1), (6, 2)],
    };
    assert_eq!(
        hex::encode(write_settings_map(&snapshot)),
        vectors::WRITTEN_SETTINGS_HEX
    );
}

#[test]
fn written_mtp_data_matches_the_oracle() {
    assert_eq!(
        hex::encode(write_mtp_data(
            &hex::decode(vectors::WRITTEN_MTP_AUTHORIZATION_HEX).unwrap()
        )),
        vectors::WRITTEN_MTP_DATA_HEX
    );
}

#[test]
fn a_written_settings_map_reads_back() {
    let snapshot = SettingsSnapshot {
        cache: CacheLimits {
            total_size: 150 * 1024 * 1024,
            total_time: 7200,
            big_file_total_size: 300 * 1024 * 1024,
            big_file_total_time: 60,
        },
        session_settings: b"session_settings".to_vec(),
        recent_stickers: vec![(5, 1), (6, 2)],
    };
    let written = write_settings_map(&snapshot);
    let mut reader = StreamReader::new(&written);
    let mut context = ReadSettingsContext::new();
    let mut sink = RecordingSink::new();
    assert!(read_settings_map(
        &mut reader,
        vectors::VERSION,
        &mut context,
        &mut sink
    ));

    assert_eq!(context.cache_total_size_limit, 150 * 1024 * 1024);
    assert_eq!(context.cache_total_time_limit, 7200);
    assert_eq!(context.cache_big_file_total_size_limit, 300 * 1024 * 1024);
    assert_eq!(context.cache_big_file_total_time_limit, 60);
    assert_eq!(
        context.session_settings.serialized,
        Some((b"session_settings".to_vec(), vectors::VERSION))
    );
    assert_eq!(
        sink.effects,
        vec![LegacyEffect::RecentStickersPreload(vec![(5, 1), (6, 2)])]
    );
    assert!(!context.legacy_read, "no legacy block was written");
}

#[test]
fn an_empty_session_settings_payload_is_not_written() {
    let snapshot = SettingsSnapshot {
        cache: CacheLimits {
            total_size: 64 * 1024 * 1024,
            total_time: 0,
            big_file_total_size: 64 * 1024 * 1024,
            big_file_total_time: 0,
        },
        session_settings: Vec::new(),
        recent_stickers: Vec::new(),
    };
    let written = write_settings_map(&snapshot);
    let mut reader = StreamReader::new(&written);
    let mut context = ReadSettingsContext::new();
    let mut sink = RecordingSink::new();
    assert!(read_settings_map(
        &mut reader,
        vectors::VERSION,
        &mut context,
        &mut sink
    ));
    assert_eq!(context.session_settings.serialized, None);
    // The stickers block is always written, so its (empty) preload is always
    // set, exactly like `cSetRecentStickersPreload(v)` in C++.
    assert_eq!(
        sink.effects,
        vec![LegacyEffect::RecentStickersPreload(Vec::new())]
    );
}

#[test]
fn every_known_id_is_accepted_by_is_known() {
    for block_id in id::KNOWN {
        assert!(id::is_known(block_id), "{block_id:#x} is known");
    }
    assert!(!id::is_known(0x7F));
    assert!(!id::is_known(id::DBI_VERSION));
    assert!(!id::is_known(id::DBI_ENCRYPTED));
    assert!(!id::is_known(id::DBI_ENCRYPTED_WITH_SALT));
}