//! Fixtures for the unit tests.
//!
//! The built-in Telegram keys are the `kTestPublicRSAKeys` and
//! `kPublicRSAKeys` entries of
//! `tdesktop/Telegram/SourceFiles/mtproto/mtproto_dc_options.cpp`; their
//! SPKI twins and all fingerprints come from an independent Python oracle
//! written straight from the C++ sources (lib_tl serialization,
//! `BN_bn2bin` minimality, SHA-1 through hashlib). The generated key and
//! the raw and OAEP vectors come from the OpenSSL 3 CLI — the same
//! library the original links against: `C_RAW_HEX` is its
//! `rsa_padding_mode:none` encryption of `M_RAW_HEX`, and `OAEP_C_HEX`
//! decrypts back to `OAEP_MSG_HEX` under `rsa_padding_mode:oaep` with
//! SHA-1.

pub const TEST_PKCS1_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEAyMEdY1aR+sCR3ZSJrtztKTKqigvO/vBfqACJLZtS7QMgCGXJ6XIR\nyy7mx66W0/sOFa7/1mAZtEoIokDP3ShoqF4fVNb6XeqgQfaUHd8wJpDWHcR2OFwv\nplUUI1PLTktZ9uW2WE23b+ixNwJjJGwBDJPQEQFBE+vfmH0JP503wr5INS1poWg/\nj25sIWeYPHYeOrFp/eXaqhISP6G+q2IeTaWTXpwZj4LzXq5YOpk4bYEQ6mvRq7D1\naHWfYmlEGepfaYR8Q0YqvvhYtMte3ITnuSJs171+GDqpdKcSwHnd6FudwGO4pcCO\nj4WcDuXc2CTHgH8gFTNhp/Y8/SpDOhvn9QIDAQAB\n-----END RSA PUBLIC KEY-----";
pub const TEST_SPKI_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyMEdY1aR+sCR3ZSJrtzt\nKTKqigvO/vBfqACJLZtS7QMgCGXJ6XIRyy7mx66W0/sOFa7/1mAZtEoIokDP3Sho\nqF4fVNb6XeqgQfaUHd8wJpDWHcR2OFwvplUUI1PLTktZ9uW2WE23b+ixNwJjJGwB\nDJPQEQFBE+vfmH0JP503wr5INS1poWg/j25sIWeYPHYeOrFp/eXaqhISP6G+q2Ie\nTaWTXpwZj4LzXq5YOpk4bYEQ6mvRq7D1aHWfYmlEGepfaYR8Q0YqvvhYtMte3ITn\nuSJs171+GDqpdKcSwHnd6FudwGO4pcCOj4WcDuXc2CTHgH8gFTNhp/Y8/SpDOhvn\n9QIDAQAB\n-----END PUBLIC KEY-----";
pub const PROD_PKCS1_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA6LszBcC1LGzyr992NzE0ieY+BSaOW622Aa9Bd4ZHLl+TuFQ4lo4g\n5nKaMBwK/BIb9xUfg0Q29/2mgIR6Zr9krM7HjuIcCzFvDtr+L0GQjae9H0pRB2OO\n62cECs5HKhT5DZ98K33vmWiLowc621dQuwKWSQKjWf50XYFw42h21P2KXUGyp2y/\n+aEyZ+uVgLLQbRA1dEjSDZ2iGRy12Mk5gpYc397aYp438fsJoHIgJ2lgMv5h7WY9\nt6N/byY9Nw9p21Og3AoXSL2q/2IJ1WRUhebgAdGVMlV1fkuOQoEzR7EdpqtQD9Cs\n5+bfo3Nhmcyvk5ftB0WkJ9z6bNZ7yxrP8wIDAQAB\n-----END RSA PUBLIC KEY-----";
pub const PROD_SPKI_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA6LszBcC1LGzyr992NzE0\nieY+BSaOW622Aa9Bd4ZHLl+TuFQ4lo4g5nKaMBwK/BIb9xUfg0Q29/2mgIR6Zr9k\nrM7HjuIcCzFvDtr+L0GQjae9H0pRB2OO62cECs5HKhT5DZ98K33vmWiLowc621dQ\nuwKWSQKjWf50XYFw42h21P2KXUGyp2y/+aEyZ+uVgLLQbRA1dEjSDZ2iGRy12Mk5\ngpYc397aYp438fsJoHIgJ2lgMv5h7WY9t6N/byY9Nw9p21Og3AoXSL2q/2IJ1WRU\nhebgAdGVMlV1fkuOQoEzR7EdpqtQD9Cs5+bfo3Nhmcyvk5ftB0WkJ9z6bNZ7yxrP\n8wIDAQAB\n-----END PUBLIC KEY-----";
pub const GEN_PKCS1_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA1TZbYjtZVFCwSqUX1eAnZZLWbYfti9zmrEfVKM89tNItK7xk1w3n\nDfHFPn9It7Wz9u+6AScLWrirdjiRL6y776ojHuRZ3vv3JX8y4lf5S8Z4Tz5Jhz2S\ncmdkl0v4yE/N9ESMdq5ObnPWAQLkLvEZ+UbF8lzU3ZM1407XwcvnRMijz6MiwO0F\noiFShS6CnwzObKb0cSLfxih3Zuid4pHRH9X2mG3LJ/69EwzvJeB5ZGVw8024bWXF\n9aasAekKeXwmQBPV8fwyBO2i4b/ccdJL4n7IYWW8aC+2B4f28cfkYLACF1yUSk1a\ntpzJZKmViZZib0E+scmxe7MeJJpyQrpqYwIDAQAB\n-----END RSA PUBLIC KEY-----";
pub const GEN_SPKI_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1TZbYjtZVFCwSqUX1eAn\nZZLWbYfti9zmrEfVKM89tNItK7xk1w3nDfHFPn9It7Wz9u+6AScLWrirdjiRL6y7\n76ojHuRZ3vv3JX8y4lf5S8Z4Tz5Jhz2Scmdkl0v4yE/N9ESMdq5ObnPWAQLkLvEZ\n+UbF8lzU3ZM1407XwcvnRMijz6MiwO0FoiFShS6CnwzObKb0cSLfxih3Zuid4pHR\nH9X2mG3LJ/69EwzvJeB5ZGVw8024bWXF9aasAekKeXwmQBPV8fwyBO2i4b/ccdJL\n4n7IYWW8aC+2B4f28cfkYLACF1yUSk1atpzJZKmViZZib0E+scmxe7MeJJpyQrpq\nYwIDAQAB\n-----END PUBLIC KEY-----";

/// `openssl rsa -modulus` of the generated key.
pub const GEN_N_HEX: &str = "d5365b623b595450b04aa517d5e0276592d66d87ed8bdce6ac47d528cf3db4d22d2bbc64d70de70df1c53e7f48b7b5b3f6efba01270b5ab8ab7638912facbbefaa231ee459defbf7257f32e257f94bc6784f3e49873d92726764974bf8c84fcdf4448c76ae4e6e73d60102e42ef119f946c5f25cd4dd9335e34ed7c1cbe744c8a3cfa322c0ed05a22152852e829f0cce6ca6f47122dfc6287766e89de291d11fd5f6986dcb27febd130cef25e079646570f34db86d65c5f5a6ac01e90a797c264013d5f1fc3204eda2e1bfdc71d24be27ec86165bc682fb60787f6f1c7e460b002175c944a4d5ab69cc964a9958996626f413eb1c9b17bb31e249a7242ba6a63";
pub const GEN_E_HEX: &str = "010001";
/// Fingerprint of the generated key from the Python oracle.
pub const GEN_FINGERPRINT: u64 = 0x4e5d9c0f5b6fe880;

/// Raw 256-byte input below the modulus.
pub const M_RAW_HEX: &str = "288bf1f544a73f22e20332a58ae554761d9ecc464c25d9ab9778dedd079ae50af0416b94748e3c8235038a73bdb8381a45cb4682d1826200b98f22c8722ffe5e31e2420917df79c782dcbff953c5b3fcc55785c34d1fac827e8370bb356fdcd4581c30a6e16ba9221ba37a02d932c92398f3e9df6137a3f66a82960e676be1035b71a53669c0a82d970be4deafad804db5b917aba4b645f36556f9d97619dae9cf6bf4a5f9aaf67a6e402e7cb59e2da94594ecade9f74ba13269d346c107f203f9cc25d9c5364988b7382538de6edcb1054cb0ba9d95f336dc9c45079b379b261d24bdb759342ef46102b0e8986449dd78a8cf00929cd2e373d005dc7255ee4f";
/// `openssl pkeyutl -encrypt -pubin -pkeyopt rsa_padding_mode:none`
/// of `M_RAW_HEX`.
pub const C_RAW_HEX: &str = "44d4881225622527b33556c8f14ba4da445ec95c42349870bbc0bc6005b7ad3a387dd108aa1339dc4ff40e46577d1456545e773929f5eb2f5aab3b3df3666752f81e37780fdc9673dc7131323b11482619892a43dc5223b2e6ea14f9d75280843fb353a7418bb60aeefc55dabc662a3dc8e01c94f481b097cdee50ca6eca7ddda1735c9cb2231ad85f40e80f9e35b1805eb4111f6bbead33b38c24d4148032d77f53a575b665df2c8b94754427139704c18d1148296fe19c64df6b56826b5023be93b180964c853eb8509f200ba777b4ae0bec246b41091940bfb5925bc719485bc631d3dac0795bb447c49b3d463b78da183156eae3690715c10ed75983f558";
/// Raw 256-byte input below the modulus, for the decrypt direction.
pub const S_RAW_HEX: &str = "36841c918a83f8350bff97af9a79308ecb789cbf4f1e529be9c5c8d47c4ccf5c79e2bde3e5a8efe7e50cb6e37284b945486fdda4a61253563bda8518eb0bc833b785af2970dc03bc253533e7c57052d69b34baff253ebcd5057d410a4b6ff5a6a1c6c9cc326233eb478827ea3353e008a96725ec848cd4824128aa597f9d5f5e670fcb51e3a2a2cab6d98e90895cbde1ff8ac46b38a16428020b2f2c246a1f5c7667fdb783bc7915901a41477ee49a2f7d6483e87431eaefd7bddbd5bf0605bf3967e0e5ab7a0e7e943046c76bb90462a6a2c45af1bd3f9840dbf621f9d7dccee692af94b45a6accd4c913cf92b69b47da89a1d3bb3883bfb48a9e3f35920fde";
/// `S_RAW_HEX ^ e mod n`, with the arithmetic cross-checked against
/// OpenSSL raw encryption.
pub const M_EXP_HEX: &str = "48f999fe3b7cfe90a082f65b605984d55f8062758e9fbf829c5f37c78806208dac08de324e9a34f84213caa751e851906f7d4e479473a240c0a25be7c3ed307093f707ca0d03f9808d628eabe5fd973bc4c5841ccb0ca8484d8f541dd32b8d06e9864f938672141310395af3266b78cd5c38eb0b9ac5f43b2b185693590e8ffc35bb57f12c97feb734b94f1612b0a63a44c062aaa9cdd39de46cf23f92205acd47c56b15cb2bf38b50effdb71419b1ade67b7e9ab6271c5edc480336a1085f422d916b7fef0ad5d65e67c6636bffc289ee4bda785961c3c932b6cf5e201ba7232d10bb353e98a159c56fb24617f89d0e00abec8d62347e508c4ba00ba3e050f8";

/// The OAEP message of the deterministic test, 100 bytes.
pub const OAEP_MSG_HEX: &str = "787b21380f134dcafe6be7f3eefe9845890e33a6f4e3ad9d3c0a4e2d7f8f303d9df162e5aa1d2ffe066aefe229c534cdc8a752e93b6dd59f60a766cce845e9fda081f7d7e000d614b08fc9859fe5e5d2cf56459d97eaa35588a95036d2e466e91626cfb1";
/// The fixed OAEP seed of the deterministic test.
pub const OAEP_SEED_HEX: &str = "000102030405060708090a0b0c0d0e0f10111213";
/// OAEP ciphertext of the message under the fixed seed; OpenSSL
/// decrypts it back to `OAEP_MSG_HEX`.
pub const OAEP_C_HEX: &str = "9d472e1a3811ca6fbca8a75e576c86b053b910314b7251951f6e7cdaf0badf9f44a270ca6c2000024ac0b933d90290685f068fa38ed13f20553b6d97f0e0542261435f0acd84e8ea4cbdd64e18bd478114e0c48dace92e31221d5faea8b6cf3e044e0bfcb2173cb2520486bcb8365fe52a991cea881ba94b2b0c59893c5e23bed6573ef166124ac17c9dad6615c48ba5bddd140e001ffa54e4bdda65227378db186191c221ab96f532b05d0c729eec5d5f88ac4f2a0c9155877374dff00ab54a1f41461a31e4b7d40bcbbac0ac7a2f8a3e9894833c54035121fa7824c77149a85c24bab9243f2b71089006a1370b4bd0fc8b7a744c08b3be1cf8284bbde5f8b4";
