use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

/// OPC/UA DateTime: number of 100ns intervals since 1601-01-01T00:00:00Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(i64);

impl Default for DateTime {
    /// OPC/UA `DateTime.MinValue` (1601-01-01T00:00:00Z).
    fn default() -> Self {
        DateTime::MIN
    }
}

impl binary::Encode for DateTime {
    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64_le(self.0);
    }
}

impl binary::Decode for DateTime {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 8)?;
        Ok(DateTime::from_timestamp(buf.get_i64_le()))
    }
}

const UNIX_EPOCH_OFFSET_SECS: i64 = (1970 - 1601) * 365 * 24 * 60 * 60 + 89 * 24 * 60 * 60;
const TICKS_PER_SEC: i64 = 10_000_000;
// 9999-12-31T23:59:59Z, in ticks since 1601-01-01
const MAX_TICKS: i64 = 2_650_467_743_990_000_000;

impl DateTime {
    /// OPC/UA minimum value: 1601-01-01T00:00:00Z.
    pub const MIN: DateTime = DateTime(0);
    /// OPC/UA maximum value: 9999-12-31T23:59:59Z and later, encoded as `i64::MAX`.
    pub const MAX: DateTime = DateTime(i64::MAX);

    /// Number of 100ns ticks since 1601-01-01T00:00:00Z.
    pub const fn timestamp(&self) -> i64 {
        self.0
    }

    /// Builds a DateTime from 100ns ticks since 1601-01-01T00:00:00Z.
    /// Values before the epoch give `MIN`; values at or after 9999-12-31T23:59:59Z give `MAX`.
    pub const fn from_timestamp(ticks: i64) -> Self {
        if ticks <= 0 {
            Self::MIN
        } else if ticks >= MAX_TICKS {
            Self::MAX
        } else {
            DateTime(ticks)
        }
    }

    /// Current time from the system clock (100ns resolution).
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unix_ticks = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => (d.as_nanos() / 100) as i64,
            Err(e) => -((e.duration().as_nanos() / 100) as i64),
        };
        DateTime::from_timestamp(UNIX_EPOCH_OFFSET_SECS * TICKS_PER_SEC + unix_ticks)
    }
}

const DAYS_1601_TO_1970: i64 = 134_774;

/// Error returned when parsing or converting an ISO 8601 date-time fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid ISO 8601 date-time: {0}")]
pub struct ParseDateTimeError(&'static str);

/// Broken-down components of an ISO 8601 date-time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeParts {
    pub year: i64,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Fraction of a second, in 100ns ticks (0..10_000_000).
    pub ticks: u32,
    /// UTC offset in minutes (0 for `Z`).
    pub offset_minutes: i32,
}

// Days since 1970-01-01 for a proleptic Gregorian date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_in_month(y: i64, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        _ => 28,
    }
}

impl From<DateTime> for DateTimeParts {
    /// Breaks a DateTime down into UTC components.
    fn from(dt: DateTime) -> Self {
        let secs = dt.0.div_euclid(TICKS_PER_SEC);
        let (year, month, day) = civil_from_days(secs.div_euclid(86_400) - DAYS_1601_TO_1970);
        let sod = secs.rem_euclid(86_400);
        DateTimeParts {
            year,
            month: month as u8,
            day: day as u8,
            hour: (sod / 3600) as u8,
            minute: (sod % 3600 / 60) as u8,
            second: (sod % 60) as u8,
            ticks: dt.0.rem_euclid(TICKS_PER_SEC) as u32,
            offset_minutes: 0,
        }
    }
}

impl TryFrom<DateTimeParts> for DateTime {
    type Error = ParseDateTimeError;

    /// Validates the components and converts them (applying the offset) to a DateTime.
    fn try_from(p: DateTimeParts) -> Result<Self, Self::Error> {
        let err = ParseDateTimeError;
        if !(1..=12).contains(&p.month) || p.day < 1 || p.day > days_in_month(p.year, p.month) {
            return Err(err("date out of range"));
        }
        // 60 allows a leap second
        if p.hour > 23 || p.minute > 59 || p.second > 60 || i64::from(p.ticks) >= TICKS_PER_SEC {
            return Err(err("time out of range"));
        }
        let days =
            days_from_civil(p.year, i64::from(p.month), i64::from(p.day)) + DAYS_1601_TO_1970;
        let secs = i64::from(p.hour) * 3600 + i64::from(p.minute) * 60 + i64::from(p.second)
            - i64::from(p.offset_minutes) * 60;
        days.checked_mul(86_400)
            .and_then(|v| v.checked_add(secs))
            .and_then(|v| v.checked_mul(TICKS_PER_SEC))
            .and_then(|v| v.checked_add(i64::from(p.ticks)))
            .map(DateTime::from_timestamp)
            .ok_or(err("out of range"))
    }
}

impl std::fmt::Display for DateTimeParts {
    /// `YYYY-MM-DDTHH:MM:SS[.fffffff]` followed by `Z` or `±HH:MM`; trailing zeros of the fraction are trimmed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.year < 0 {
            write!(f, "-{:04}", -self.year)?;
        } else {
            write!(f, "{:04}", self.year)?;
        }
        write!(
            f,
            "-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.month, self.day, self.hour, self.minute, self.second
        )?;
        if self.ticks != 0 {
            let digits = format!("{:07}", self.ticks);
            write!(f, ".{}", digits.trim_end_matches('0'))?;
        }
        if self.offset_minutes == 0 {
            write!(f, "Z")
        } else {
            let sign = if self.offset_minutes < 0 { '-' } else { '+' };
            let abs = self.offset_minutes.abs();
            write!(f, "{sign}{:02}:{:02}", abs / 60, abs % 60)
        }
    }
}

fn take_num(s: &mut &str, min: usize, max: usize) -> Option<i64> {
    let n = s.bytes().take(max).take_while(u8::is_ascii_digit).count();
    if n < min {
        return None;
    }
    let (num, rest) = s.split_at(n);
    *s = rest;
    num.parse().ok()
}

fn eat(s: &mut &str, c: char) -> bool {
    match s.strip_prefix(c) {
        Some(rest) => {
            *s = rest;
            true
        }
        None => false,
    }
}

impl std::str::FromStr for DateTimeParts {
    type Err = ParseDateTimeError;

    /// Parses `[-]YYYY-MM-DD[THH:MM[:SS[.f+]][Z|±HH[:MM]]]`. Fractional digits beyond 100ns
    /// are truncated. Components are range-checked only by conversion to `DateTime`.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let err = ParseDateTimeError;
        let mut s = input;
        let neg = eat(&mut s, '-');
        let year = take_num(&mut s, 4, 9).ok_or(err("year"))?;
        let year = if neg { -year } else { year };
        if !eat(&mut s, '-') {
            return Err(err("expected '-' after year"));
        }
        let month = take_num(&mut s, 2, 2).ok_or(err("month"))? as u8;
        if !eat(&mut s, '-') {
            return Err(err("expected '-' after month"));
        }
        let day = take_num(&mut s, 2, 2).ok_or(err("day"))? as u8;

        let mut p = DateTimeParts {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
            ticks: 0,
            offset_minutes: 0,
        };
        if eat(&mut s, 'T') {
            p.hour = take_num(&mut s, 2, 2).ok_or(err("hour"))? as u8;
            if !eat(&mut s, ':') {
                return Err(err("expected ':' after hour"));
            }
            p.minute = take_num(&mut s, 2, 2).ok_or(err("minute"))? as u8;
            if eat(&mut s, ':') {
                p.second = take_num(&mut s, 2, 2).ok_or(err("second"))? as u8;
                if eat(&mut s, '.') || eat(&mut s, ',') {
                    let n = s.bytes().take_while(u8::is_ascii_digit).count();
                    if n == 0 {
                        return Err(err("fraction"));
                    }
                    let (digits, rest) = s.split_at(n);
                    s = rest;
                    let mut scale = TICKS_PER_SEC as u32 / 10;
                    for b in digits.bytes().take(7) {
                        p.ticks += u32::from(b - b'0') * scale;
                        scale /= 10;
                    }
                }
            }
            if !eat(&mut s, 'Z') && s.starts_with(['+', '-']) {
                let sign = if eat(&mut s, '+') {
                    1
                } else {
                    s = &s[1..];
                    -1
                };
                let oh = take_num(&mut s, 2, 2).ok_or(err("offset hour"))?;
                eat(&mut s, ':');
                let om = if s.is_empty() {
                    0
                } else {
                    take_num(&mut s, 2, 2).ok_or(err("offset minute"))?
                };
                if oh > 23 || om > 59 {
                    return Err(err("offset out of range"));
                }
                p.offset_minutes = sign * (oh * 60 + om) as i32;
            }
        }
        if !s.is_empty() {
            return Err(err("trailing characters"));
        }
        Ok(p)
    }
}

impl std::fmt::Display for DateTime {
    /// ISO 8601 in UTC, e.g. `2024-02-29T12:34:56.789Z`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DateTimeParts::from(*self).fmt(f)
    }
}

impl std::str::FromStr for DateTime {
    type Err = ParseDateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DateTime::try_from(s.parse::<DateTimeParts>()?)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        let seconds = dt.timestamp() + UNIX_EPOCH_OFFSET_SECS;
        let ticks = seconds
            .saturating_mul(TICKS_PER_SEC)
            .saturating_add((dt.timestamp_subsec_nanos() / 100) as i64);
        DateTime::from_timestamp(ticks)
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<DateTime> for chrono::DateTime<chrono::Utc> {
    type Error = DateTimeOutOfRange;

    fn try_from(dt: DateTime) -> Result<Self, Self::Error> {
        let seconds = dt.0.div_euclid(10_000_000) - UNIX_EPOCH_OFFSET_SECS;
        let nanos = dt.0.rem_euclid(10_000_000) as u32 * 100;
        chrono::DateTime::from_timestamp(seconds, nanos).ok_or(DateTimeOutOfRange)
    }
}

/// The OPC/UA DateTime cannot be represented as a chrono DateTime.
#[cfg(feature = "chrono")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("OPC/UA DateTime out of range for chrono")]
pub struct DateTimeOutOfRange;

#[cfg(all(test, feature = "chrono"))]
mod tests {
    use super::*;

    #[test]
    fn now_is_after_2020() {
        let y2020 = DateTime::from(chrono::DateTime::from_timestamp(1_577_836_800, 0).unwrap());
        assert!(DateTime::now() > y2020);
    }

    #[test]
    fn chrono_roundtrip() {
        let unix_epoch = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let dt = DateTime::from(unix_epoch);
        assert_eq!(dt.timestamp(), 116_444_736_000_000_000);
        assert_eq!(chrono::DateTime::try_from(dt).unwrap(), unix_epoch);
    }
}

#[cfg(test)]
mod str_tests {
    use super::*;

    #[test]
    fn display_fromstr() {
        assert_eq!(DateTime::MIN.to_string(), "1601-01-01T00:00:00Z");
        let unix = DateTime::from_timestamp(116_444_736_000_000_000);
        assert_eq!(unix.to_string(), "1970-01-01T00:00:00Z");
        let dt: DateTime = "2024-02-29T12:34:56.789Z".parse().unwrap();
        assert_eq!(dt.to_string(), "2024-02-29T12:34:56.789Z");
        assert_eq!(
            "1970-01-01T01:00:00+01:00".parse::<DateTime>().unwrap(),
            unix
        );
        assert_eq!("1970-01-01".parse::<DateTime>().unwrap(), unix);
        let x = DateTime::from_timestamp(123_456_789_012_345_678);
        assert_eq!(x.to_string().parse::<DateTime>().unwrap(), x);
        assert!("2023-02-29T00:00:00Z".parse::<DateTime>().is_err());
        assert!("garbage".parse::<DateTime>().is_err());
    }

    #[test]
    fn parts() {
        let p: DateTimeParts = "2024-02-29T12:34:56.5-05:30".parse().unwrap();
        assert_eq!(
            (p.year, p.month, p.day, p.hour, p.minute, p.second),
            (2024, 2, 29, 12, 34, 56)
        );
        assert_eq!((p.ticks, p.offset_minutes), (5_000_000, -330));
        assert_eq!(p.to_string(), "2024-02-29T12:34:56.5-05:30");
    }

    #[test]
    fn clamping() {
        assert_eq!(DateTime::from_timestamp(-5), DateTime::MIN);
        assert_eq!(DateTime::from_timestamp(i64::MAX), DateTime::MAX);
        assert_eq!(DateTime::from_timestamp(MAX_TICKS), DateTime::MAX);
        assert_eq!(DateTime::from_timestamp(42).timestamp(), 42);
        let end: DateTime = "9999-12-31T23:59:59Z".parse().unwrap();
        assert_eq!(end, DateTime::MAX);
    }
}
