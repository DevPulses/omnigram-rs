#!/usr/bin/env python3
"""Oracle generator for the tdata_key_data crate.

Re-implements the C++ path of
`tdesktop/Telegram/SourceFiles/storage/details/storage_file_utilities.cpp`
from scratch, with no code shared with the Rust port:

  * hashing and PBKDF2 come from Python's `hashlib`;
  * AES-ECB comes from `cryptography` and AES-IGE is chained by hand here,
    following `AES_ige_encrypt` semantics (iv = iv0 || iv1);
  * the passcode KDFs are taken from `openssl kdf`, i.e. the very library the
    C++ code calls (ARGON2ID / SCRYPT with the recorded cost numbers).

Output: `tests/vectors/mod.rs`, a Rust module of hex constants consumed by the
integration tests. Run from anywhere:

    python3 crates/tdata_key_data/tests/oracle/gen_oracle.py
"""

import hashlib
import pathlib
import subprocess

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent / "vectors" / "mod.rs"

APP_VERSION = 7004002
TDF_MAGIC = b"TDF$"


# --- primitives -------------------------------------------------------------


def xor(a: bytes, b: bytes) -> bytes:
    return bytes(x ^ y for x, y in zip(a, b))


def aes_ecb_encrypt_block(key: bytes, block: bytes) -> bytes:
    enc = Cipher(algorithms.AES(key), modes.ECB()).encryptor()
    return enc.update(block) + enc.finalize()


def aes_ecb_decrypt_block(key: bytes, block: bytes) -> bytes:
    dec = Cipher(algorithms.AES(key), modes.ECB()).decryptor()
    return dec.update(block) + dec.finalize()


def aes_ige_encrypt(data: bytes, key: bytes, iv: bytes) -> bytes:
    assert len(data) % 16 == 0
    iv1, iv2 = iv[:16], iv[16:]
    out = b""
    for i in range(0, len(data), 16):
        p = data[i : i + 16]
        c = xor(aes_ecb_encrypt_block(key, xor(p, iv1)), iv2)
        iv1, iv2 = c, p
        out += c
    return out


def aes_ige_decrypt(data: bytes, key: bytes, iv: bytes) -> bytes:
    assert len(data) % 16 == 0
    iv1, iv2 = iv[:16], iv[16:]
    out = b""
    for i in range(0, len(data), 16):
        c = data[i : i + 16]
        p = xor(aes_ecb_decrypt_block(key, xor(c, iv2)), iv1)
        iv1, iv2 = c, p
        out += p
    return out


def prepare_aes_oldmtp(auth_key: bytes, msg_key: bytes, send: bool):
    """`AuthKey::prepareAES_oldmtp` — MTProto 1.0 derivation, `send == False`."""
    x = 0 if send else 8
    a = hashlib.sha1(msg_key + auth_key[x : x + 32]).digest()
    b = hashlib.sha1(auth_key[32 + x : 48 + x] + msg_key + auth_key[48 + x : 64 + x]).digest()
    c = hashlib.sha1(auth_key[64 + x : 96 + x] + msg_key).digest()
    d = hashlib.sha1(msg_key + auth_key[96 + x : 128 + x]).digest()
    key = a[:8] + b[8:20] + c[4:16]
    iv = a[8:20] + b[:8] + c[16:20] + d[:8]
    return key, iv


def create_local_key(passcode: bytes, salt: bytes) -> bytes:
    hashed = hashlib.sha512(salt + passcode + salt).digest()
    rounds = 1 if not passcode else 100_000
    return hashlib.pbkdf2_hmac("sha512", hashed, salt, rounds, 256)


def create_legacy_local_key(passcode: bytes, salt: bytes) -> bytes:
    rounds = 4 if not passcode else 4_000
    return hashlib.pbkdf2_hmac("sha1", passcode, salt, rounds, 256)


def prepare_encrypted(payload: bytes, local_key: bytes, pad: int) -> bytes:
    """`PrepareEncrypted` with a deterministic padding byte."""
    size = len(payload)
    full = size
    body = bytearray(payload)
    if full % 16:
        full += 16 - (full % 16)
        body += bytes([pad]) * (full - size)
    body[0:4] = size.to_bytes(4, "little")
    key128 = hashlib.sha1(bytes(body)).digest()[:16]
    key, iv = prepare_aes_oldmtp(local_key, key128, False)
    return key128 + aes_ige_encrypt(bytes(body), key, iv)


def file_block(data: bytes) -> bytes:
    return len(data).to_bytes(4, "big") + data


def sign_file(body: bytes, version: int) -> bytes:
    md5 = hashlib.md5()
    md5.update(body)
    md5.update(len(body).to_bytes(4, "little", signed=True))
    md5.update(version.to_bytes(4, "little", signed=True))
    md5.update(TDF_MAGIC)
    return md5.digest()


def to_file_part(val: int) -> str:
    out = ""
    for _ in range(0x10):
        v = val & 0x0F
        out += chr(ord("0") + v) if v < 0x0A else chr(ord("A") + (v - 0x0A))
        val >>= 4
    return out


# --- openssl-backed KDFs ----------------------------------------------------


def openssl_kdf(algorithm: str, options: list[str], keylen: int) -> bytes:
    args = ["openssl", "kdf", "-keylen", str(keylen)]
    for option in options:
        args += ["-kdfopt", option]
    args.append(algorithm)
    result = subprocess.run(args, check=True, capture_output=True, text=True)
    hexed = result.stdout.strip().replace(":", "")
    return bytes.fromhex(hexed)


# --- fixtures ---------------------------------------------------------------


LOCAL_KEY = bytes(range(256))
SALT_32 = bytes(range(0x20))
SALT_16 = bytes(range(0x10))
PASSCODE = b"passcode"
PAD_BYTE = 0xEE

BLOCK_ONE = b"hello tdata"
BLOCK_THREE = bytes((i * 7 + 3) & 0xFF for i in range(37))

PAYLOAD = (
    (0).to_bytes(4, "little")
    + file_block(BLOCK_ONE)
    + (0xFFFFFFFF).to_bytes(4, "big")  # null block
    + file_block(BLOCK_THREE)
)
ENCRYPTED = prepare_encrypted(PAYLOAD, LOCAL_KEY, PAD_BYTE)
FILE_BODY = file_block(ENCRYPTED)
FILE = (
    TDF_MAGIC
    + APP_VERSION.to_bytes(4, "little", signed=True)
    + FILE_BODY
    + sign_file(FILE_BODY, APP_VERSION)
)


def main() -> None:
    argon2id = openssl_kdf(
        "ARGON2ID",
        [
            "pass:" + PASSCODE.decode(),
            "hexsalt:" + SALT_16.hex(),
            "memcost:65536",
            "iter:8",
            "lanes:1",
            "threads:1",
        ],
        256,
    )
    scrypt_key = openssl_kdf(
        "SCRYPT",
        [
            "pass:" + PASSCODE.decode(),
            "hexsalt:" + SALT_16.hex(),
            "n:131072",
            "r:8",
            "p:1",
            "maxmem_bytes:201326592",
        ],
        256,
    )
    scrypt_small = openssl_kdf(
        "SCRYPT",
        [
            "pass:" + PASSCODE.decode(),
            "hexsalt:" + SALT_16.hex(),
            "n:65536",
            "r:4",
            "p:1",
            "maxmem_bytes:201326592",
        ],
        256,
    )

    lines = [
        "// @generated by tests/oracle/gen_oracle.py — do not edit by hand.",
        "//",
        "// Oracle: an independent Python implementation of the C++ path in",
        "// `storage/details/storage_file_utilities.cpp` (hashlib + hand-chained",
        "// AES-IGE over AES-ECB), with the passcode KDFs produced by `openssl kdf`,",
        "// the same library the C++ code calls.",
        "//",
        "// Every test binary includes the whole module and uses part of it.",
        "#![allow(dead_code)]",
        "",
        f"pub const APP_VERSION: i32 = {APP_VERSION};",
        "",
        "/// 256-byte fixed local key: bytes 0x00..0xFF.",
        f'pub const LOCAL_KEY_HEX: &str = "{LOCAL_KEY.hex()}";',
        "/// Salt used by the key derivations: bytes 0x00..0x1F.",
        f'pub const SALT_32_HEX: &str = "{SALT_32.hex()}";',
        "/// Passcode used by every derivation below.",
        f'pub const PASSCODE: &[u8] = b"{PASSCODE.decode()}";',
        "",
        "/// `CreateLocalKey` with a passcode (100000 PBKDF2-SHA512 rounds).",
        f'pub const LOCAL_KEY_WITH_PASSCODE_HEX: &str = "{create_local_key(PASSCODE, SALT_32).hex()}";',
        "/// `CreateLocalKey` without a passcode (a single round).",
        f'pub const LOCAL_KEY_NO_PASSCODE_HEX: &str = "{create_local_key(b"", SALT_32).hex()}";',
        "/// `CreateLegacyLocalKey` with a passcode (4000 PBKDF2-SHA1 rounds).",
        f'pub const LEGACY_KEY_WITH_PASSCODE_HEX: &str = "{create_legacy_local_key(PASSCODE, SALT_32).hex()}";',
        "/// `CreateLegacyLocalKey` without a passcode (4 rounds).",
        f'pub const LEGACY_KEY_NO_PASSCODE_HEX: &str = "{create_legacy_local_key(b"", SALT_32).hex()}";',
        "",
        "/// Salt for the passcode wraps: bytes 0x00..0x0F.",
        f'pub const SALT_16_HEX: &str = "{SALT_16.hex()}";',
        "/// Argon2id, memcost 65536 KiB, 8 passes, 1 lane, 256-byte output.",
        f'pub const PASSCODE_ARGON2ID_HEX: &str = "{argon2id.hex()}";',
        "/// Scrypt, N 131072, r 8, p 1, 256-byte output.",
        f'pub const PASSCODE_SCRYPT_HEX: &str = "{scrypt_key.hex()}";',
        "/// Scrypt, N 65536, r 4, p 1 — the Windows x86 read path.",
        f'pub const PASSCODE_SCRYPT_SMALL_HEX: &str = "{scrypt_small.hex()}";',
        "",
        "/// First block written into the payload descriptor.",
        f'pub const BLOCK_ONE_HEX: &str = "{BLOCK_ONE.hex()}";',
        "/// Third block: 37 bytes, so the payload needs random padding.",
        f'pub const BLOCK_THREE_HEX: &str = "{BLOCK_THREE.hex()}";',
        "/// Padding byte the tests feed through `prepare_encrypted_with`.",
        f"pub const PAD_BYTE: u8 = {PAD_BYTE};",
        "/// Payload: size prefix, block, null block, block.",
        f'pub const PAYLOAD_HEX: &str = "{PAYLOAD.hex()}";',
        "/// `PrepareEncrypted` of that payload with the fixed padding byte.",
        f'pub const ENCRYPTED_HEX: &str = "{ENCRYPTED.hex()}";',
        "",
        "/// The block as stored in a file: big-endian length, then the blob.",
        f'pub const FILE_BODY_HEX: &str = "{FILE_BODY.hex()}";',
        "/// The whole container: magic, version, block, MD5.",
        f'pub const FILE_HEX: &str = "{FILE.hex()}";',
        "",
        f'pub const FILE_PART_OF_ABCDEF_HEX: &str = "{to_file_part(0x0123456789ABCDEF)}";',
        f'pub const FILE_PART_OF_ONE_HEX: &str = "{to_file_part(1)}";',
        f'pub const FILE_PART_OF_MAX_HEX: &str = "{to_file_part(0xFFFFFFFFFFFFFFFF)}";',
        "",
    ]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines))
    print(f"wrote {OUT}")
    print(f"payload {len(PAYLOAD)} bytes, encrypted {len(ENCRYPTED)} bytes, file {len(FILE)} bytes")


if __name__ == "__main__":
    main()