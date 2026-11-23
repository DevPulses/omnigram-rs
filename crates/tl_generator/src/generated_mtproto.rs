// Сгенерировано tl_generator из .tl-схемы (LAYER 0).
// Схема MTProto.
// НЕ редактировать вручную.
#![allow(clippy::all, unused_variables, dead_code)]


use tl_reader::{read_double, read_int, read_int128, read_int256, read_long, read_string,
    read_vector, Eof, Int128, Int256, Reader};
use tl_writer::{write_double, write_int, write_int128, write_int256, write_long, write_string,
    Writer};

/// Bare-чтение значения из TL-потока.
pub trait TlRead: Sized {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof>;
}

/// Bare-запись значения в TL-поток.
pub trait TlWrite {
    fn tl_write(&self, w: &mut Writer);
}

/// Boxed-чтение: конструктор уже прочитан и передан аргументом.
pub trait TlBoxedRead: Sized {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof>;
}

/// Boxed-запись: конструктор пишет реализация.
pub trait TlBoxedWrite {
    fn tl_write_boxed(&self, w: &mut Writer);
}

impl TlRead for i32 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int(r) }
}
impl TlWrite for i32 {
    fn tl_write(&self, w: &mut Writer) { write_int(w, *self) }
}
impl TlRead for u32 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { r.get() }
}
impl TlWrite for u32 {
    fn tl_write(&self, w: &mut Writer) { w.put(*self) }
}
impl TlRead for u64 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_long(r) }
}
impl TlWrite for u64 {
    fn tl_write(&self, w: &mut Writer) { write_long(w, *self) }
}
impl TlRead for f64 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_double(r) }
}
impl TlWrite for f64 {
    fn tl_write(&self, w: &mut Writer) { write_double(w, *self) }
}
impl TlRead for Vec<u8> {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_string(r) }
}
impl TlWrite for Vec<u8> {
    fn tl_write(&self, w: &mut Writer) { write_string(w, self) }
}
impl TlRead for Int128 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int128(r) }
}
impl TlWrite for Int128 {
    fn tl_write(&self, w: &mut Writer) { write_int128(w, *self) }
}
impl TlRead for Int256 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int256(r) }
}
impl TlWrite for Int256 {
    fn tl_write(&self, w: &mut Writer) { write_int256(w, *self) }
}
impl TlRead for bool {
    /// Bare bool: слово 0/1.
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { Ok(read_int(r)? != 0) }
}
impl TlWrite for bool {
    fn tl_write(&self, w: &mut Writer) { write_int(w, i32::from(*self)) }
}
impl<T: TlRead> TlRead for Vec<T> {
    /// Bare vector: счётчик и элементы, без конструктора 0x1cb5c415
    /// (как `Vector<long>` в схеме).
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        read_vector(r, T::tl_read)
    }
}
impl<T: TlWrite> TlWrite for Vec<T> {
    fn tl_write(&self, w: &mut Writer) {
        w.put(self.len() as u32);
        for x in self {
            x.tl_write(w);
        }
    }
}
impl<T: TlBoxedRead> TlBoxedRead for Vec<T> {
    /// Boxed vector: конструктор уже прочитан; счётчик, затем
    /// boxed-элементы (у каждого свой конструктор).
    fn tl_read_boxed(r: &mut Reader, _cons: u32) -> Result<Self, Eof> {
        let count = r.get()? as usize;
        if !r.has(count) {
            return Err(Eof);
        }
        let mut v = Vec::with_capacity(count.min(r.remaining_primes()));
        for _ in 0..count {
            let cons = r.get()?;
            v.push(T::tl_read_boxed(r, cons)?);
        }
        Ok(v)
    }
}
impl<T: TlRead> TlRead for Box<T> {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { Ok(Box::new(T::tl_read(r)?)) }
}
impl<T: TlWrite> TlWrite for Box<T> {
    fn tl_write(&self, w: &mut Writer) { T::tl_write(self, w) }
}
impl<T: TlBoxedRead> TlBoxedRead for Box<T> {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Ok(Box::new(T::tl_read_boxed(r, cons)?))
    }
}
impl<T: TlBoxedWrite> TlBoxedWrite for Box<T> {
    fn tl_write_boxed(&self, w: &mut Writer) { T::tl_write_boxed(self, w) }
}
impl<T: TlBoxedWrite> TlBoxedWrite for Vec<T> {
    fn tl_write_boxed(&self, w: &mut Writer) {
        w.put(tl_reader::ID_VECTOR);
        w.put(self.len() as u32);
        for x in self {
            x.tl_write_boxed(w);
        }
    }
}

/// Конструкторы (аналог mtpc_*).
pub const RES_PQ: u32 = 0x05162463;
pub const P_Q_INNER_DATA: u32 = 0x83c95aec;
pub const P_Q_INNER_DATA_DC: u32 = 0xa9f55f95;
pub const P_Q_INNER_DATA_TEMP: u32 = 0x3c6a84d4;
pub const P_Q_INNER_DATA_TEMP_DC: u32 = 0x56fddf88;
pub const BIND_AUTH_KEY_INNER: u32 = 0x75a3f765;
pub const SERVER_DH_PARAMS_FAIL: u32 = 0x79cb045d;
pub const SERVER_DH_PARAMS_OK: u32 = 0xd0e8075c;
pub const SERVER_DH_INNER_DATA: u32 = 0xb5890dba;
pub const CLIENT_DH_INNER_DATA: u32 = 0x6643b654;
pub const DH_GEN_OK: u32 = 0x3bcbf734;
pub const DH_GEN_RETRY: u32 = 0x46dc1fb9;
pub const DH_GEN_FAIL: u32 = 0xa69dae02;
pub const DESTROY_AUTH_KEY_OK: u32 = 0xf660e1d4;
pub const DESTROY_AUTH_KEY_NONE: u32 = 0x0a9f2259;
pub const DESTROY_AUTH_KEY_FAIL: u32 = 0xea109b13;
pub const REQ_PQ: u32 = 0x60469778;
pub const REQ_PQ_MULTI: u32 = 0xbe7e8ef1;
pub const REQ_DH_PARAMS: u32 = 0xd712e4be;
pub const SET_CLIENT_DH_PARAMS: u32 = 0xf5045f1f;
pub const DESTROY_AUTH_KEY: u32 = 0xd1435160;
pub const MSGS_ACK: u32 = 0x62d6b459;
pub const BAD_MSG_NOTIFICATION: u32 = 0xa7eff811;
pub const BAD_SERVER_SALT: u32 = 0xedab447b;
pub const MSGS_STATE_REQ: u32 = 0xda69fb52;
pub const MSGS_STATE_INFO: u32 = 0x04deb57d;
pub const MSGS_ALL_INFO: u32 = 0x8cc0d131;
pub const MSG_DETAILED_INFO: u32 = 0x276d3ec6;
pub const MSG_NEW_DETAILED_INFO: u32 = 0x809db6df;
pub const MSG_RESEND_REQ: u32 = 0x7d861a08;
pub const RPC_ERROR: u32 = 0x2144ca19;
pub const RPC_ANSWER_UNKNOWN: u32 = 0x5e2ad36e;
pub const RPC_ANSWER_DROPPED_RUNNING: u32 = 0xcd78e586;
pub const RPC_ANSWER_DROPPED: u32 = 0xa43ad8b7;
pub const FUTURE_SALT: u32 = 0x0949d9dc;
pub const FUTURE_SALTS: u32 = 0xae500895;
pub const PONG: u32 = 0x347773c5;
pub const DESTROY_SESSION_OK: u32 = 0xe22045fc;
pub const DESTROY_SESSION_NONE: u32 = 0x62d350c9;
pub const NEW_SESSION_CREATED: u32 = 0x9ec20908;
pub const HTTP_WAIT: u32 = 0x9299359f;
pub const IP_PORT: u32 = 0xd433ad73;
pub const IP_PORT_SECRET: u32 = 0x37982646;
pub const ACCESS_POINT_RULE: u32 = 0x4679b65f;
pub const HELP_CONFIG_SIMPLE: u32 = 0x5a592a6c;
pub const TLS_CLIENT_HELLO: u32 = 0x6c52c484;
pub const TLS_BLOCK_STRING: u32 = 0x4218a164;
pub const TLS_BLOCK_RANDOM: u32 = 0x4d4dc41e;
pub const TLS_BLOCK_ZERO: u32 = 0x09333afb;
pub const TLS_BLOCK_DOMAIN: u32 = 0x10e8636f;
pub const TLS_BLOCK_GREASE: u32 = 0xe675a1c1;
pub const TLS_BLOCK_PUBLIC_KEY: u32 = 0x9eb95b5c;
pub const TLS_BLOCK_SCOPE: u32 = 0xe725d44f;
pub const TLS_BLOCK_PERMUTATION: u32 = 0x4cd9354d;
pub const TLS_BLOCK_M: u32 = 0x9de0538b;
pub const TLS_BLOCK_E: u32 = 0x9bec2575;
pub const TLS_BLOCK_PADDING: u32 = 0xa4357218;
pub const RPC_DROP_ANSWER: u32 = 0x58e4a740;
pub const GET_FUTURE_SALTS: u32 = 0xb921bd04;
pub const PING: u32 = 0x7abe77ec;
pub const PING_DELAY_DISCONNECT: u32 = 0xf3427b8c;
pub const DESTROY_SESSION: u32 = 0xe7512126;

#[derive(Debug, Clone, PartialEq)]
pub enum ResPQ {
    ResPQ { nonce: Int128, server_nonce: Int128, pq: Vec<u8>, server_public_key_fingerprints: Vec<u64>, },
}

impl ResPQ {
    pub const RES_PQ: u32 = 0x05162463;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::RES_PQ => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let pq = read_string(r)?;
            let server_public_key_fingerprints = read_vector(r, |r| read_long(r))?;
            Ok(Self::ResPQ {
                nonce,
                server_nonce,
                pq,
                server_public_key_fingerprints,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::ResPQ { nonce: f0, server_nonce: f1, pq: f2, server_public_key_fingerprints: f3 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_string(w, f2);
                w.put(f3.len() as u32);
                for _elem in f3.iter() { write_long(w, *_elem); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::ResPQ { .. } => { w.put(Self::RES_PQ); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for ResPQ {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for ResPQ {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for ResPQ {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::RES_PQ)
    }
}
impl TlWrite for ResPQ {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PQInnerData {
    PQInnerData { pq: Vec<u8>, p: Vec<u8>, q: Vec<u8>, nonce: Int128, server_nonce: Int128, new_nonce: Int256, },
    PQInnerDataDc { pq: Vec<u8>, p: Vec<u8>, q: Vec<u8>, nonce: Int128, server_nonce: Int128, new_nonce: Int256, dc: i32, },
    PQInnerDataTemp { pq: Vec<u8>, p: Vec<u8>, q: Vec<u8>, nonce: Int128, server_nonce: Int128, new_nonce: Int256, expires_in: i32, },
    PQInnerDataTempDc { pq: Vec<u8>, p: Vec<u8>, q: Vec<u8>, nonce: Int128, server_nonce: Int128, new_nonce: Int256, dc: i32, expires_in: i32, },
}

impl PQInnerData {
    pub const P_Q_INNER_DATA: u32 = 0x83c95aec;
    pub const P_Q_INNER_DATA_DC: u32 = 0xa9f55f95;
    pub const P_Q_INNER_DATA_TEMP: u32 = 0x3c6a84d4;
    pub const P_Q_INNER_DATA_TEMP_DC: u32 = 0x56fddf88;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::P_Q_INNER_DATA => {
            let pq = read_string(r)?;
            let p = read_string(r)?;
            let q = read_string(r)?;
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce = read_int256(r)?;
            Ok(Self::PQInnerData {
                pq,
                p,
                q,
                nonce,
                server_nonce,
                new_nonce,
            })
        }
            Self::P_Q_INNER_DATA_DC => {
            let pq = read_string(r)?;
            let p = read_string(r)?;
            let q = read_string(r)?;
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce = read_int256(r)?;
            let dc = read_int(r)?;
            Ok(Self::PQInnerDataDc {
                pq,
                p,
                q,
                nonce,
                server_nonce,
                new_nonce,
                dc,
            })
        }
            Self::P_Q_INNER_DATA_TEMP => {
            let pq = read_string(r)?;
            let p = read_string(r)?;
            let q = read_string(r)?;
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce = read_int256(r)?;
            let expires_in = read_int(r)?;
            Ok(Self::PQInnerDataTemp {
                pq,
                p,
                q,
                nonce,
                server_nonce,
                new_nonce,
                expires_in,
            })
        }
            Self::P_Q_INNER_DATA_TEMP_DC => {
            let pq = read_string(r)?;
            let p = read_string(r)?;
            let q = read_string(r)?;
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce = read_int256(r)?;
            let dc = read_int(r)?;
            let expires_in = read_int(r)?;
            Ok(Self::PQInnerDataTempDc {
                pq,
                p,
                q,
                nonce,
                server_nonce,
                new_nonce,
                dc,
                expires_in,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::PQInnerData { pq: f0, p: f1, q: f2, nonce: f3, server_nonce: f4, new_nonce: f5 } => {
                write_string(w, f0);
                write_string(w, f1);
                write_string(w, f2);
                write_int128(w, *f3);
                write_int128(w, *f4);
                write_int256(w, *f5);
            }
            Self::PQInnerDataDc { pq: f0, p: f1, q: f2, nonce: f3, server_nonce: f4, new_nonce: f5, dc: f6 } => {
                write_string(w, f0);
                write_string(w, f1);
                write_string(w, f2);
                write_int128(w, *f3);
                write_int128(w, *f4);
                write_int256(w, *f5);
                write_int(w, *f6);
            }
            Self::PQInnerDataTemp { pq: f0, p: f1, q: f2, nonce: f3, server_nonce: f4, new_nonce: f5, expires_in: f6 } => {
                write_string(w, f0);
                write_string(w, f1);
                write_string(w, f2);
                write_int128(w, *f3);
                write_int128(w, *f4);
                write_int256(w, *f5);
                write_int(w, *f6);
            }
            Self::PQInnerDataTempDc { pq: f0, p: f1, q: f2, nonce: f3, server_nonce: f4, new_nonce: f5, dc: f6, expires_in: f7 } => {
                write_string(w, f0);
                write_string(w, f1);
                write_string(w, f2);
                write_int128(w, *f3);
                write_int128(w, *f4);
                write_int256(w, *f5);
                write_int(w, *f6);
                write_int(w, *f7);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::PQInnerData { .. } => { w.put(Self::P_Q_INNER_DATA); self.write_unboxed(w); },
            Self::PQInnerDataDc { .. } => { w.put(Self::P_Q_INNER_DATA_DC); self.write_unboxed(w); },
            Self::PQInnerDataTemp { .. } => { w.put(Self::P_Q_INNER_DATA_TEMP); self.write_unboxed(w); },
            Self::PQInnerDataTempDc { .. } => { w.put(Self::P_Q_INNER_DATA_TEMP_DC); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for PQInnerData {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for PQInnerData {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for PQInnerData {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for PQInnerData {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindAuthKeyInner {
    BindAuthKeyInner { nonce: u64, temp_auth_key_id: u64, perm_auth_key_id: u64, temp_session_id: u64, expires_at: i32, },
}

impl BindAuthKeyInner {
    pub const BIND_AUTH_KEY_INNER: u32 = 0x75a3f765;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::BIND_AUTH_KEY_INNER => {
            let nonce = read_long(r)?;
            let temp_auth_key_id = read_long(r)?;
            let perm_auth_key_id = read_long(r)?;
            let temp_session_id = read_long(r)?;
            let expires_at = read_int(r)?;
            Ok(Self::BindAuthKeyInner {
                nonce,
                temp_auth_key_id,
                perm_auth_key_id,
                temp_session_id,
                expires_at,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::BindAuthKeyInner { nonce: f0, temp_auth_key_id: f1, perm_auth_key_id: f2, temp_session_id: f3, expires_at: f4 } => {
                write_long(w, *f0);
                write_long(w, *f1);
                write_long(w, *f2);
                write_long(w, *f3);
                write_int(w, *f4);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::BindAuthKeyInner { .. } => { w.put(Self::BIND_AUTH_KEY_INNER); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for BindAuthKeyInner {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for BindAuthKeyInner {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for BindAuthKeyInner {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::BIND_AUTH_KEY_INNER)
    }
}
impl TlWrite for BindAuthKeyInner {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerDHParams {
    ServerDHParamsFail { nonce: Int128, server_nonce: Int128, new_nonce_hash: Int128, },
    ServerDHParamsOk { nonce: Int128, server_nonce: Int128, encrypted_answer: Vec<u8>, },
}

impl ServerDHParams {
    pub const SERVER_DH_PARAMS_FAIL: u32 = 0x79cb045d;
    pub const SERVER_DH_PARAMS_OK: u32 = 0xd0e8075c;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::SERVER_DH_PARAMS_FAIL => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce_hash = read_int128(r)?;
            Ok(Self::ServerDHParamsFail {
                nonce,
                server_nonce,
                new_nonce_hash,
            })
        }
            Self::SERVER_DH_PARAMS_OK => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let encrypted_answer = read_string(r)?;
            Ok(Self::ServerDHParamsOk {
                nonce,
                server_nonce,
                encrypted_answer,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::ServerDHParamsFail { nonce: f0, server_nonce: f1, new_nonce_hash: f2 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_int128(w, *f2);
            }
            Self::ServerDHParamsOk { nonce: f0, server_nonce: f1, encrypted_answer: f2 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_string(w, f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::ServerDHParamsFail { .. } => { w.put(Self::SERVER_DH_PARAMS_FAIL); self.write_unboxed(w); },
            Self::ServerDHParamsOk { .. } => { w.put(Self::SERVER_DH_PARAMS_OK); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for ServerDHParams {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for ServerDHParams {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for ServerDHParams {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for ServerDHParams {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerDHInnerData {
    ServerDHInnerData { nonce: Int128, server_nonce: Int128, g: i32, dh_prime: Vec<u8>, g_a: Vec<u8>, server_time: i32, },
}

impl ServerDHInnerData {
    pub const SERVER_DH_INNER_DATA: u32 = 0xb5890dba;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::SERVER_DH_INNER_DATA => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let g = read_int(r)?;
            let dh_prime = read_string(r)?;
            let g_a = read_string(r)?;
            let server_time = read_int(r)?;
            Ok(Self::ServerDHInnerData {
                nonce,
                server_nonce,
                g,
                dh_prime,
                g_a,
                server_time,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::ServerDHInnerData { nonce: f0, server_nonce: f1, g: f2, dh_prime: f3, g_a: f4, server_time: f5 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_int(w, *f2);
                write_string(w, f3);
                write_string(w, f4);
                write_int(w, *f5);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::ServerDHInnerData { .. } => { w.put(Self::SERVER_DH_INNER_DATA); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for ServerDHInnerData {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for ServerDHInnerData {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for ServerDHInnerData {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::SERVER_DH_INNER_DATA)
    }
}
impl TlWrite for ServerDHInnerData {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClientDHInnerData {
    ClientDHInnerData { nonce: Int128, server_nonce: Int128, retry_id: u64, g_b: Vec<u8>, },
}

impl ClientDHInnerData {
    pub const CLIENT_DH_INNER_DATA: u32 = 0x6643b654;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::CLIENT_DH_INNER_DATA => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let retry_id = read_long(r)?;
            let g_b = read_string(r)?;
            Ok(Self::ClientDHInnerData {
                nonce,
                server_nonce,
                retry_id,
                g_b,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::ClientDHInnerData { nonce: f0, server_nonce: f1, retry_id: f2, g_b: f3 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_long(w, *f2);
                write_string(w, f3);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::ClientDHInnerData { .. } => { w.put(Self::CLIENT_DH_INNER_DATA); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for ClientDHInnerData {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for ClientDHInnerData {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for ClientDHInnerData {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::CLIENT_DH_INNER_DATA)
    }
}
impl TlWrite for ClientDHInnerData {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetClientDHParamsAnswer {
    DhGenOk { nonce: Int128, server_nonce: Int128, new_nonce_hash1: Int128, },
    DhGenRetry { nonce: Int128, server_nonce: Int128, new_nonce_hash2: Int128, },
    DhGenFail { nonce: Int128, server_nonce: Int128, new_nonce_hash3: Int128, },
}

impl SetClientDHParamsAnswer {
    pub const DH_GEN_OK: u32 = 0x3bcbf734;
    pub const DH_GEN_RETRY: u32 = 0x46dc1fb9;
    pub const DH_GEN_FAIL: u32 = 0xa69dae02;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::DH_GEN_OK => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce_hash1 = read_int128(r)?;
            Ok(Self::DhGenOk {
                nonce,
                server_nonce,
                new_nonce_hash1,
            })
        }
            Self::DH_GEN_RETRY => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce_hash2 = read_int128(r)?;
            Ok(Self::DhGenRetry {
                nonce,
                server_nonce,
                new_nonce_hash2,
            })
        }
            Self::DH_GEN_FAIL => {
            let nonce = read_int128(r)?;
            let server_nonce = read_int128(r)?;
            let new_nonce_hash3 = read_int128(r)?;
            Ok(Self::DhGenFail {
                nonce,
                server_nonce,
                new_nonce_hash3,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::DhGenOk { nonce: f0, server_nonce: f1, new_nonce_hash1: f2 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_int128(w, *f2);
            }
            Self::DhGenRetry { nonce: f0, server_nonce: f1, new_nonce_hash2: f2 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_int128(w, *f2);
            }
            Self::DhGenFail { nonce: f0, server_nonce: f1, new_nonce_hash3: f2 } => {
                write_int128(w, *f0);
                write_int128(w, *f1);
                write_int128(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::DhGenOk { .. } => { w.put(Self::DH_GEN_OK); self.write_unboxed(w); },
            Self::DhGenRetry { .. } => { w.put(Self::DH_GEN_RETRY); self.write_unboxed(w); },
            Self::DhGenFail { .. } => { w.put(Self::DH_GEN_FAIL); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for SetClientDHParamsAnswer {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for SetClientDHParamsAnswer {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for SetClientDHParamsAnswer {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for SetClientDHParamsAnswer {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DestroyAuthKeyRes {
    DestroyAuthKeyOk,
    DestroyAuthKeyNone,
    DestroyAuthKeyFail,
}

impl DestroyAuthKeyRes {
    pub const DESTROY_AUTH_KEY_OK: u32 = 0xf660e1d4;
    pub const DESTROY_AUTH_KEY_NONE: u32 = 0x0a9f2259;
    pub const DESTROY_AUTH_KEY_FAIL: u32 = 0xea109b13;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::DESTROY_AUTH_KEY_OK => {
            Ok(Self::DestroyAuthKeyOk {
            })
        }
            Self::DESTROY_AUTH_KEY_NONE => {
            Ok(Self::DestroyAuthKeyNone {
            })
        }
            Self::DESTROY_AUTH_KEY_FAIL => {
            Ok(Self::DestroyAuthKeyFail {
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::DestroyAuthKeyOk => {
            }
            Self::DestroyAuthKeyNone => {
            }
            Self::DestroyAuthKeyFail => {
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::DestroyAuthKeyOk { .. } => { w.put(Self::DESTROY_AUTH_KEY_OK); self.write_unboxed(w); },
            Self::DestroyAuthKeyNone { .. } => { w.put(Self::DESTROY_AUTH_KEY_NONE); self.write_unboxed(w); },
            Self::DestroyAuthKeyFail { .. } => { w.put(Self::DESTROY_AUTH_KEY_FAIL); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for DestroyAuthKeyRes {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for DestroyAuthKeyRes {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for DestroyAuthKeyRes {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for DestroyAuthKeyRes {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgsAck {
    MsgsAck { msg_ids: Vec<u64>, },
}

impl MsgsAck {
    pub const MSGS_ACK: u32 = 0x62d6b459;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSGS_ACK => {
            let msg_ids = read_vector(r, |r| read_long(r))?;
            Ok(Self::MsgsAck {
                msg_ids,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsAck { msg_ids: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { write_long(w, *_elem); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsAck { .. } => { w.put(Self::MSGS_ACK); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgsAck {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgsAck {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgsAck {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::MSGS_ACK)
    }
}
impl TlWrite for MsgsAck {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BadMsgNotification {
    BadMsgNotification { bad_msg_id: u64, bad_msg_seqno: i32, error_code: i32, },
    BadServerSalt { bad_msg_id: u64, bad_msg_seqno: i32, error_code: i32, new_server_salt: u64, },
}

impl BadMsgNotification {
    pub const BAD_MSG_NOTIFICATION: u32 = 0xa7eff811;
    pub const BAD_SERVER_SALT: u32 = 0xedab447b;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::BAD_MSG_NOTIFICATION => {
            let bad_msg_id = read_long(r)?;
            let bad_msg_seqno = read_int(r)?;
            let error_code = read_int(r)?;
            Ok(Self::BadMsgNotification {
                bad_msg_id,
                bad_msg_seqno,
                error_code,
            })
        }
            Self::BAD_SERVER_SALT => {
            let bad_msg_id = read_long(r)?;
            let bad_msg_seqno = read_int(r)?;
            let error_code = read_int(r)?;
            let new_server_salt = read_long(r)?;
            Ok(Self::BadServerSalt {
                bad_msg_id,
                bad_msg_seqno,
                error_code,
                new_server_salt,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::BadMsgNotification { bad_msg_id: f0, bad_msg_seqno: f1, error_code: f2 } => {
                write_long(w, *f0);
                write_int(w, *f1);
                write_int(w, *f2);
            }
            Self::BadServerSalt { bad_msg_id: f0, bad_msg_seqno: f1, error_code: f2, new_server_salt: f3 } => {
                write_long(w, *f0);
                write_int(w, *f1);
                write_int(w, *f2);
                write_long(w, *f3);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::BadMsgNotification { .. } => { w.put(Self::BAD_MSG_NOTIFICATION); self.write_unboxed(w); },
            Self::BadServerSalt { .. } => { w.put(Self::BAD_SERVER_SALT); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for BadMsgNotification {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for BadMsgNotification {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for BadMsgNotification {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for BadMsgNotification {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgsStateReq {
    MsgsStateReq { msg_ids: Vec<u64>, },
}

impl MsgsStateReq {
    pub const MSGS_STATE_REQ: u32 = 0xda69fb52;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSGS_STATE_REQ => {
            let msg_ids = read_vector(r, |r| read_long(r))?;
            Ok(Self::MsgsStateReq {
                msg_ids,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsStateReq { msg_ids: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { write_long(w, *_elem); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsStateReq { .. } => { w.put(Self::MSGS_STATE_REQ); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgsStateReq {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgsStateReq {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgsStateReq {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::MSGS_STATE_REQ)
    }
}
impl TlWrite for MsgsStateReq {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgsStateInfo {
    MsgsStateInfo { req_msg_id: u64, info: Vec<u8>, },
}

impl MsgsStateInfo {
    pub const MSGS_STATE_INFO: u32 = 0x04deb57d;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSGS_STATE_INFO => {
            let req_msg_id = read_long(r)?;
            let info = read_string(r)?;
            Ok(Self::MsgsStateInfo {
                req_msg_id,
                info,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsStateInfo { req_msg_id: f0, info: f1 } => {
                write_long(w, *f0);
                write_string(w, f1);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsStateInfo { .. } => { w.put(Self::MSGS_STATE_INFO); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgsStateInfo {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgsStateInfo {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgsStateInfo {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::MSGS_STATE_INFO)
    }
}
impl TlWrite for MsgsStateInfo {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgsAllInfo {
    MsgsAllInfo { msg_ids: Vec<u64>, info: Vec<u8>, },
}

impl MsgsAllInfo {
    pub const MSGS_ALL_INFO: u32 = 0x8cc0d131;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSGS_ALL_INFO => {
            let msg_ids = read_vector(r, |r| read_long(r))?;
            let info = read_string(r)?;
            Ok(Self::MsgsAllInfo {
                msg_ids,
                info,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsAllInfo { msg_ids: f0, info: f1 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { write_long(w, *_elem); }
                write_string(w, f1);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgsAllInfo { .. } => { w.put(Self::MSGS_ALL_INFO); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgsAllInfo {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgsAllInfo {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgsAllInfo {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::MSGS_ALL_INFO)
    }
}
impl TlWrite for MsgsAllInfo {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgDetailedInfo {
    MsgDetailedInfo { msg_id: u64, answer_msg_id: u64, bytes: i32, status: i32, },
    MsgNewDetailedInfo { answer_msg_id: u64, bytes: i32, status: i32, },
}

impl MsgDetailedInfo {
    pub const MSG_DETAILED_INFO: u32 = 0x276d3ec6;
    pub const MSG_NEW_DETAILED_INFO: u32 = 0x809db6df;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSG_DETAILED_INFO => {
            let msg_id = read_long(r)?;
            let answer_msg_id = read_long(r)?;
            let bytes = read_int(r)?;
            let status = read_int(r)?;
            Ok(Self::MsgDetailedInfo {
                msg_id,
                answer_msg_id,
                bytes,
                status,
            })
        }
            Self::MSG_NEW_DETAILED_INFO => {
            let answer_msg_id = read_long(r)?;
            let bytes = read_int(r)?;
            let status = read_int(r)?;
            Ok(Self::MsgNewDetailedInfo {
                answer_msg_id,
                bytes,
                status,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgDetailedInfo { msg_id: f0, answer_msg_id: f1, bytes: f2, status: f3 } => {
                write_long(w, *f0);
                write_long(w, *f1);
                write_int(w, *f2);
                write_int(w, *f3);
            }
            Self::MsgNewDetailedInfo { answer_msg_id: f0, bytes: f1, status: f2 } => {
                write_long(w, *f0);
                write_int(w, *f1);
                write_int(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgDetailedInfo { .. } => { w.put(Self::MSG_DETAILED_INFO); self.write_unboxed(w); },
            Self::MsgNewDetailedInfo { .. } => { w.put(Self::MSG_NEW_DETAILED_INFO); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgDetailedInfo {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgDetailedInfo {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgDetailedInfo {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for MsgDetailedInfo {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgResendReq {
    MsgResendReq { msg_ids: Vec<u64>, },
}

impl MsgResendReq {
    pub const MSG_RESEND_REQ: u32 = 0x7d861a08;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::MSG_RESEND_REQ => {
            let msg_ids = read_vector(r, |r| read_long(r))?;
            Ok(Self::MsgResendReq {
                msg_ids,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::MsgResendReq { msg_ids: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { write_long(w, *_elem); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::MsgResendReq { .. } => { w.put(Self::MSG_RESEND_REQ); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for MsgResendReq {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for MsgResendReq {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for MsgResendReq {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::MSG_RESEND_REQ)
    }
}
impl TlWrite for MsgResendReq {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RpcError {
    RpcError { error_code: i32, error_message: Vec<u8>, },
}

impl RpcError {
    pub const RPC_ERROR: u32 = 0x2144ca19;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::RPC_ERROR => {
            let error_code = read_int(r)?;
            let error_message = read_string(r)?;
            Ok(Self::RpcError {
                error_code,
                error_message,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::RpcError { error_code: f0, error_message: f1 } => {
                write_int(w, *f0);
                write_string(w, f1);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::RpcError { .. } => { w.put(Self::RPC_ERROR); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for RpcError {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for RpcError {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for RpcError {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::RPC_ERROR)
    }
}
impl TlWrite for RpcError {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RpcDropAnswer {
    RpcAnswerUnknown,
    RpcAnswerDroppedRunning,
    RpcAnswerDropped { msg_id: u64, seq_no: i32, bytes: i32, },
}

impl RpcDropAnswer {
    pub const RPC_ANSWER_UNKNOWN: u32 = 0x5e2ad36e;
    pub const RPC_ANSWER_DROPPED_RUNNING: u32 = 0xcd78e586;
    pub const RPC_ANSWER_DROPPED: u32 = 0xa43ad8b7;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::RPC_ANSWER_UNKNOWN => {
            Ok(Self::RpcAnswerUnknown {
            })
        }
            Self::RPC_ANSWER_DROPPED_RUNNING => {
            Ok(Self::RpcAnswerDroppedRunning {
            })
        }
            Self::RPC_ANSWER_DROPPED => {
            let msg_id = read_long(r)?;
            let seq_no = read_int(r)?;
            let bytes = read_int(r)?;
            Ok(Self::RpcAnswerDropped {
                msg_id,
                seq_no,
                bytes,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::RpcAnswerUnknown => {
            }
            Self::RpcAnswerDroppedRunning => {
            }
            Self::RpcAnswerDropped { msg_id: f0, seq_no: f1, bytes: f2 } => {
                write_long(w, *f0);
                write_int(w, *f1);
                write_int(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::RpcAnswerUnknown { .. } => { w.put(Self::RPC_ANSWER_UNKNOWN); self.write_unboxed(w); },
            Self::RpcAnswerDroppedRunning { .. } => { w.put(Self::RPC_ANSWER_DROPPED_RUNNING); self.write_unboxed(w); },
            Self::RpcAnswerDropped { .. } => { w.put(Self::RPC_ANSWER_DROPPED); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for RpcDropAnswer {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for RpcDropAnswer {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for RpcDropAnswer {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for RpcDropAnswer {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FutureSalt {
    FutureSalt { valid_since: i32, valid_until: i32, salt: u64, },
}

impl FutureSalt {
    pub const FUTURE_SALT: u32 = 0x0949d9dc;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::FUTURE_SALT => {
            let valid_since = read_int(r)?;
            let valid_until = read_int(r)?;
            let salt = read_long(r)?;
            Ok(Self::FutureSalt {
                valid_since,
                valid_until,
                salt,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::FutureSalt { valid_since: f0, valid_until: f1, salt: f2 } => {
                write_int(w, *f0);
                write_int(w, *f1);
                write_long(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::FutureSalt { .. } => { w.put(Self::FUTURE_SALT); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for FutureSalt {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for FutureSalt {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for FutureSalt {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::FUTURE_SALT)
    }
}
impl TlWrite for FutureSalt {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FutureSalts {
    FutureSalts { req_msg_id: u64, now: i32, salts: Vec<FutureSalt>, },
}

impl FutureSalts {
    pub const FUTURE_SALTS: u32 = 0xae500895;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::FUTURE_SALTS => {
            let req_msg_id = read_long(r)?;
            let now = read_int(r)?;
            let salts = read_vector(r, |r| { let cons = r.get()?; <FutureSalt as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::FutureSalts {
                req_msg_id,
                now,
                salts,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::FutureSalts { req_msg_id: f0, now: f1, salts: f2 } => {
                write_long(w, *f0);
                write_int(w, *f1);
                w.put(f2.len() as u32);
                for _elem in f2.iter() { _elem.tl_write_boxed(w); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::FutureSalts { .. } => { w.put(Self::FUTURE_SALTS); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for FutureSalts {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for FutureSalts {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for FutureSalts {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::FUTURE_SALTS)
    }
}
impl TlWrite for FutureSalts {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pong {
    Pong { msg_id: u64, ping_id: u64, },
}

impl Pong {
    pub const PONG: u32 = 0x347773c5;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::PONG => {
            let msg_id = read_long(r)?;
            let ping_id = read_long(r)?;
            Ok(Self::Pong {
                msg_id,
                ping_id,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::Pong { msg_id: f0, ping_id: f1 } => {
                write_long(w, *f0);
                write_long(w, *f1);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::Pong { .. } => { w.put(Self::PONG); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for Pong {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for Pong {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for Pong {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::PONG)
    }
}
impl TlWrite for Pong {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DestroySessionRes {
    DestroySessionOk { session_id: u64, },
    DestroySessionNone { session_id: u64, },
}

impl DestroySessionRes {
    pub const DESTROY_SESSION_OK: u32 = 0xe22045fc;
    pub const DESTROY_SESSION_NONE: u32 = 0x62d350c9;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::DESTROY_SESSION_OK => {
            let session_id = read_long(r)?;
            Ok(Self::DestroySessionOk {
                session_id,
            })
        }
            Self::DESTROY_SESSION_NONE => {
            let session_id = read_long(r)?;
            Ok(Self::DestroySessionNone {
                session_id,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::DestroySessionOk { session_id: f0 } => {
                write_long(w, *f0);
            }
            Self::DestroySessionNone { session_id: f0 } => {
                write_long(w, *f0);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::DestroySessionOk { .. } => { w.put(Self::DESTROY_SESSION_OK); self.write_unboxed(w); },
            Self::DestroySessionNone { .. } => { w.put(Self::DESTROY_SESSION_NONE); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for DestroySessionRes {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for DestroySessionRes {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for DestroySessionRes {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for DestroySessionRes {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewSession {
    NewSessionCreated { first_msg_id: u64, unique_id: u64, server_salt: u64, },
}

impl NewSession {
    pub const NEW_SESSION_CREATED: u32 = 0x9ec20908;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::NEW_SESSION_CREATED => {
            let first_msg_id = read_long(r)?;
            let unique_id = read_long(r)?;
            let server_salt = read_long(r)?;
            Ok(Self::NewSessionCreated {
                first_msg_id,
                unique_id,
                server_salt,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::NewSessionCreated { first_msg_id: f0, unique_id: f1, server_salt: f2 } => {
                write_long(w, *f0);
                write_long(w, *f1);
                write_long(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::NewSessionCreated { .. } => { w.put(Self::NEW_SESSION_CREATED); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for NewSession {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for NewSession {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for NewSession {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::NEW_SESSION_CREATED)
    }
}
impl TlWrite for NewSession {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpWait {
    HttpWait { max_delay: i32, wait_after: i32, max_wait: i32, },
}

impl HttpWait {
    pub const HTTP_WAIT: u32 = 0x9299359f;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::HTTP_WAIT => {
            let max_delay = read_int(r)?;
            let wait_after = read_int(r)?;
            let max_wait = read_int(r)?;
            Ok(Self::HttpWait {
                max_delay,
                wait_after,
                max_wait,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::HttpWait { max_delay: f0, wait_after: f1, max_wait: f2 } => {
                write_int(w, *f0);
                write_int(w, *f1);
                write_int(w, *f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::HttpWait { .. } => { w.put(Self::HTTP_WAIT); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for HttpWait {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for HttpWait {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for HttpWait {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::HTTP_WAIT)
    }
}
impl TlWrite for HttpWait {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpPort {
    IpPort { ipv4: i32, port: i32, },
    IpPortSecret { ipv4: i32, port: i32, secret: Vec<u8>, },
}

impl IpPort {
    pub const IP_PORT: u32 = 0xd433ad73;
    pub const IP_PORT_SECRET: u32 = 0x37982646;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::IP_PORT => {
            let ipv4 = read_int(r)?;
            let port = read_int(r)?;
            Ok(Self::IpPort {
                ipv4,
                port,
            })
        }
            Self::IP_PORT_SECRET => {
            let ipv4 = read_int(r)?;
            let port = read_int(r)?;
            let secret = read_string(r)?;
            Ok(Self::IpPortSecret {
                ipv4,
                port,
                secret,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::IpPort { ipv4: f0, port: f1 } => {
                write_int(w, *f0);
                write_int(w, *f1);
            }
            Self::IpPortSecret { ipv4: f0, port: f1, secret: f2 } => {
                write_int(w, *f0);
                write_int(w, *f1);
                write_string(w, f2);
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::IpPort { .. } => { w.put(Self::IP_PORT); self.write_unboxed(w); },
            Self::IpPortSecret { .. } => { w.put(Self::IP_PORT_SECRET); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for IpPort {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for IpPort {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for IpPort {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for IpPort {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessPointRule {
    AccessPointRule { phone_prefix_rules: Vec<u8>, dc_id: i32, ips: Vec<IpPort>, },
}

impl AccessPointRule {
    pub const ACCESS_POINT_RULE: u32 = 0x4679b65f;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::ACCESS_POINT_RULE => {
            let phone_prefix_rules = read_string(r)?;
            let dc_id = read_int(r)?;
            let ips = read_vector(r, |r| { let cons = r.get()?; <IpPort as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::AccessPointRule {
                phone_prefix_rules,
                dc_id,
                ips,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::AccessPointRule { phone_prefix_rules: f0, dc_id: f1, ips: f2 } => {
                write_string(w, f0);
                write_int(w, *f1);
                w.put(f2.len() as u32);
                for _elem in f2.iter() { _elem.tl_write_boxed(w); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::AccessPointRule { .. } => { w.put(Self::ACCESS_POINT_RULE); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for AccessPointRule {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for AccessPointRule {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for AccessPointRule {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::ACCESS_POINT_RULE)
    }
}
impl TlWrite for AccessPointRule {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HelpConfigSimple {
    HelpConfigSimple { date: i32, expires: i32, rules: Vec<AccessPointRule>, },
}

impl HelpConfigSimple {
    pub const HELP_CONFIG_SIMPLE: u32 = 0x5a592a6c;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::HELP_CONFIG_SIMPLE => {
            let date = read_int(r)?;
            let expires = read_int(r)?;
            let rules = read_vector(r, |r| { let cons = r.get()?; <AccessPointRule as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::HelpConfigSimple {
                date,
                expires,
                rules,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::HelpConfigSimple { date: f0, expires: f1, rules: f2 } => {
                write_int(w, *f0);
                write_int(w, *f1);
                w.put(f2.len() as u32);
                for _elem in f2.iter() { _elem.tl_write_boxed(w); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::HelpConfigSimple { .. } => { w.put(Self::HELP_CONFIG_SIMPLE); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for HelpConfigSimple {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for HelpConfigSimple {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for HelpConfigSimple {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::HELP_CONFIG_SIMPLE)
    }
}
impl TlWrite for HelpConfigSimple {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TlsClientHello {
    TlsClientHello { blocks: Vec<TlsBlock>, },
}

impl TlsClientHello {
    pub const TLS_CLIENT_HELLO: u32 = 0x6c52c484;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::TLS_CLIENT_HELLO => {
            let blocks = read_vector(r, |r| { let cons = r.get()?; <TlsBlock as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::TlsClientHello {
                blocks,
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::TlsClientHello { blocks: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { _elem.tl_write_boxed(w); }
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::TlsClientHello { .. } => { w.put(Self::TLS_CLIENT_HELLO); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for TlsClientHello {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for TlsClientHello {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for TlsClientHello {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        Self::read_unboxed(r, Self::TLS_CLIENT_HELLO)
    }
}
impl TlWrite for TlsClientHello {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TlsBlock {
    TlsBlockString { data: Vec<u8>, },
    TlsBlockRandom { length: i32, },
    TlsBlockZero { length: i32, },
    TlsBlockDomain,
    TlsBlockGrease { seed: i32, },
    TlsBlockPublicKey,
    TlsBlockScope { entries: Vec<Box<TlsBlock>>, },
    TlsBlockPermutation { entries: Vec<Vec<Box<TlsBlock>>>, },
    TlsBlockM,
    TlsBlockE,
    TlsBlockPadding,
}

impl TlsBlock {
    pub const TLS_BLOCK_STRING: u32 = 0x4218a164;
    pub const TLS_BLOCK_RANDOM: u32 = 0x4d4dc41e;
    pub const TLS_BLOCK_ZERO: u32 = 0x09333afb;
    pub const TLS_BLOCK_DOMAIN: u32 = 0x10e8636f;
    pub const TLS_BLOCK_GREASE: u32 = 0xe675a1c1;
    pub const TLS_BLOCK_PUBLIC_KEY: u32 = 0x9eb95b5c;
    pub const TLS_BLOCK_SCOPE: u32 = 0xe725d44f;
    pub const TLS_BLOCK_PERMUTATION: u32 = 0x4cd9354d;
    pub const TLS_BLOCK_M: u32 = 0x9de0538b;
    pub const TLS_BLOCK_E: u32 = 0x9bec2575;
    pub const TLS_BLOCK_PADDING: u32 = 0xa4357218;

    /// Чтение тела по уже известному конструктору.
    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        match cons {
            Self::TLS_BLOCK_STRING => {
            let data = read_string(r)?;
            Ok(Self::TlsBlockString {
                data,
            })
        }
            Self::TLS_BLOCK_RANDOM => {
            let length = read_int(r)?;
            Ok(Self::TlsBlockRandom {
                length,
            })
        }
            Self::TLS_BLOCK_ZERO => {
            let length = read_int(r)?;
            Ok(Self::TlsBlockZero {
                length,
            })
        }
            Self::TLS_BLOCK_DOMAIN => {
            Ok(Self::TlsBlockDomain {
            })
        }
            Self::TLS_BLOCK_GREASE => {
            let seed = read_int(r)?;
            Ok(Self::TlsBlockGrease {
                seed,
            })
        }
            Self::TLS_BLOCK_PUBLIC_KEY => {
            Ok(Self::TlsBlockPublicKey {
            })
        }
            Self::TLS_BLOCK_SCOPE => {
            let entries = read_vector(r, |r| { let cons = r.get()?; <Box<TlsBlock> as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::TlsBlockScope {
                entries,
            })
        }
            Self::TLS_BLOCK_PERMUTATION => {
            let entries = read_vector(r, |r| { let cons = r.get()?; <Vec<Box<TlsBlock>> as TlBoxedRead>::tl_read_boxed(r, cons) })?;
            Ok(Self::TlsBlockPermutation {
                entries,
            })
        }
            Self::TLS_BLOCK_M => {
            Ok(Self::TlsBlockM {
            })
        }
            Self::TLS_BLOCK_E => {
            Ok(Self::TlsBlockE {
            })
        }
            Self::TLS_BLOCK_PADDING => {
            Ok(Self::TlsBlockPadding {
            })
        }
            _ => Err(Eof),
        }
    }

    /// Boxed-чтение: конструктор читается из потока.
    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }

    /// Запись тела без конструктора (bare).
    pub fn write_unboxed(&self, w: &mut Writer) {
        match self {
            Self::TlsBlockString { data: f0 } => {
                write_string(w, f0);
            }
            Self::TlsBlockRandom { length: f0 } => {
                write_int(w, *f0);
            }
            Self::TlsBlockZero { length: f0 } => {
                write_int(w, *f0);
            }
            Self::TlsBlockDomain => {
            }
            Self::TlsBlockGrease { seed: f0 } => {
                write_int(w, *f0);
            }
            Self::TlsBlockPublicKey => {
            }
            Self::TlsBlockScope { entries: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { _elem.tl_write_boxed(w); }
            }
            Self::TlsBlockPermutation { entries: f0 } => {
                w.put(f0.len() as u32);
                for _elem in f0.iter() { _elem.tl_write_boxed(w); }
            }
            Self::TlsBlockM => {
            }
            Self::TlsBlockE => {
            }
            Self::TlsBlockPadding => {
            }
        }
    }

    /// Boxed-запись: конструктор, затем тело.
    pub fn write_boxed(&self, w: &mut Writer) {
        match self {
            Self::TlsBlockString { .. } => { w.put(Self::TLS_BLOCK_STRING); self.write_unboxed(w); },
            Self::TlsBlockRandom { .. } => { w.put(Self::TLS_BLOCK_RANDOM); self.write_unboxed(w); },
            Self::TlsBlockZero { .. } => { w.put(Self::TLS_BLOCK_ZERO); self.write_unboxed(w); },
            Self::TlsBlockDomain { .. } => { w.put(Self::TLS_BLOCK_DOMAIN); self.write_unboxed(w); },
            Self::TlsBlockGrease { .. } => { w.put(Self::TLS_BLOCK_GREASE); self.write_unboxed(w); },
            Self::TlsBlockPublicKey { .. } => { w.put(Self::TLS_BLOCK_PUBLIC_KEY); self.write_unboxed(w); },
            Self::TlsBlockScope { .. } => { w.put(Self::TLS_BLOCK_SCOPE); self.write_unboxed(w); },
            Self::TlsBlockPermutation { .. } => { w.put(Self::TLS_BLOCK_PERMUTATION); self.write_unboxed(w); },
            Self::TlsBlockM { .. } => { w.put(Self::TLS_BLOCK_M); self.write_unboxed(w); },
            Self::TlsBlockE { .. } => { w.put(Self::TLS_BLOCK_E); self.write_unboxed(w); },
            Self::TlsBlockPadding { .. } => { w.put(Self::TLS_BLOCK_PADDING); self.write_unboxed(w); },
        }
    }
}

impl TlBoxedRead for TlsBlock {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Self::read_unboxed(r, cons)
    }
}
impl TlBoxedWrite for TlsBlock {
    fn tl_write_boxed(&self, w: &mut Writer) {
        self.write_boxed(w)
    }
}
impl TlRead for TlsBlock {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        let cons = r.get()?;
        Self::read_unboxed(r, cons)
    }
}
impl TlWrite for TlsBlock {
    fn tl_write(&self, w: &mut Writer) {
        self.write_unboxed(w)
    }
}

/// RPC-методы (запросы).
pub mod functions {
    use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ReqPq {
    pub nonce: Int128,
}

impl ReqPq {
    pub const ID: u32 = 0x60469778;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_int128(w, self.nonce);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReqPqMulti {
    pub nonce: Int128,
}

impl ReqPqMulti {
    pub const ID: u32 = 0xbe7e8ef1;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_int128(w, self.nonce);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReqDHParams {
    pub nonce: Int128,
    pub server_nonce: Int128,
    pub p: Vec<u8>,
    pub q: Vec<u8>,
    pub public_key_fingerprint: u64,
    pub encrypted_data: Vec<u8>,
}

impl ReqDHParams {
    pub const ID: u32 = 0xd712e4be;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_int128(w, self.nonce);
        write_int128(w, self.server_nonce);
        write_string(w, &self.p);
        write_string(w, &self.q);
        write_long(w, self.public_key_fingerprint);
        write_string(w, &self.encrypted_data);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetClientDHParams {
    pub nonce: Int128,
    pub server_nonce: Int128,
    pub encrypted_data: Vec<u8>,
}

impl SetClientDHParams {
    pub const ID: u32 = 0xf5045f1f;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_int128(w, self.nonce);
        write_int128(w, self.server_nonce);
        write_string(w, &self.encrypted_data);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DestroyAuthKey {
}

impl DestroyAuthKey {
    pub const ID: u32 = 0xd1435160;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcDropAnswer {
    pub req_msg_id: u64,
}

impl RpcDropAnswer {
    pub const ID: u32 = 0x58e4a740;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_long(w, self.req_msg_id);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetFutureSalts {
    pub num: i32,
}

impl GetFutureSalts {
    pub const ID: u32 = 0xb921bd04;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_int(w, self.num);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ping {
    pub ping_id: u64,
}

impl Ping {
    pub const ID: u32 = 0x7abe77ec;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_long(w, self.ping_id);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingDelayDisconnect {
    pub ping_id: u64,
    pub disconnect_delay: i32,
}

impl PingDelayDisconnect {
    pub const ID: u32 = 0xf3427b8c;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_long(w, self.ping_id);
        write_int(w, self.disconnect_delay);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DestroySession {
    pub session_id: u64,
}

impl DestroySession {
    pub const ID: u32 = 0xe7512126;

    /// Boxed-запрос: конструктор, затем поля.
    pub fn write_boxed(&self, w: &mut Writer) {
        w.put(Self::ID);
        write_long(w, self.session_id);
    }
}

}
