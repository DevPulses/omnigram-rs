//! The stored form of a `QDateTime`.
//!
//! Source of truth: `dateTimeSize()` in `storage/serialize_common.h`, which
//! fixes the shape at `qint64 julianDay + quint32 msecsSinceStartOfDay +
//! qint8 timeSpec` — 13 bytes, big-endian.
//!
//! `QDateTime` is not ported yet, so the value travels as its three fields.
//! [`SerializedDateTime::from_unix_millis`] builds the same values Qt writes for
//! a local-time stamp.

/// `dateTimeSize()` — the exact serialized length.
pub const DATE_TIME_SIZE: usize = 8 + 4 + 1;

/// `Qt::LocalTime` — the spec a `currentDateTime()` carries.
pub const LOCAL_TIME: i8 = 0;
/// `Qt::UTC`.
pub const UTC: i8 = 1;
/// `Qt::OffsetFromUTC`.
pub const OFFSET_FROM_UTC: i8 = 2;
/// `Qt::TimeZone`.
pub const TIME_ZONE: i8 = 3;

/// The Julian day of `1970-01-01`, the epoch of Unix time.
const JULIAN_DAY_OF_UNIX_EPOCH: i64 = 2_440_588;
/// Milliseconds in one day.
const MSECS_PER_DAY: i64 = 86_400_000;

/// A `QDateTime` as the storage layer writes it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SerializedDateTime {
    /// `QDate::toJulianDay()`.
    pub julian_day: i64,
    /// `QTime::msecsSinceStartOfDay()`.
    pub msecs: u32,
    /// `Qt::TimeSpec`.
    pub spec: i8,
}

impl SerializedDateTime {
    /// The values Qt writes for `QDateTime::fromMSecsSinceEpoch(millis, Qt::LocalTime)`.
    #[must_use]
    pub fn from_unix_millis(millis: i64) -> Self {
        let day = millis.div_euclid(MSECS_PER_DAY);
        let msecs = millis.rem_euclid(MSECS_PER_DAY);
        Self {
            julian_day: JULIAN_DAY_OF_UNIX_EPOCH + day,
            msecs: msecs as u32,
            spec: LOCAL_TIME,
        }
    }

    /// The same instant back in Unix milliseconds, ignoring the spec.
    #[must_use]
    pub fn unix_millis(self) -> i64 {
        (self.julian_day - JULIAN_DAY_OF_UNIX_EPOCH) * MSECS_PER_DAY + i64::from(self.msecs)
    }
}
