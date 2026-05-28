use crate::{ATTOS_PER_SEC_F64, ATTOS_PER_SEC_I64};
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Duration is an interval of time
///
/// Durations can handle lengths of time about 40 times as long as the age of the
/// universe, and have attosecond (10^-18) precision.
///
/// Negative values are supported.
///
/// Stored in 128 bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
pub struct Duration {
    pub(crate) secs: i64,

    // attos are normalized such that
    // -ATTOS_PER_SEC_I64 < attos < ATTOS_PER_SEC_I64
    // and maintain the same sign as secs.
    pub(crate) attos: i64,
}

impl Duration {
    pub(crate) const fn normalize(&mut self) {
        // This doesn't need divmod_i64 euclidean modulus because we reflect
        // negatives through zero
        self.secs += self.attos / ATTOS_PER_SEC_I64;
        self.attos %= ATTOS_PER_SEC_I64;
        if self.secs < 0 && self.attos > 0 {
            self.attos -= ATTOS_PER_SEC_I64;
            self.secs += 1;
        } else if self.secs > 0 && self.attos < 0 {
            self.attos += ATTOS_PER_SEC_I64;
            self.secs -= 1;
        }
    }

    /// Make a new `Duration` with given number of seconds and attoseconds.
    #[must_use]
    pub const fn new(secs: i64, attos: i64) -> Self {
        let mut d = Self { secs, attos };
        d.normalize();
        d
    }

    /// Make a new `Duration` from seconds as `f64`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn from_seconds(secs: f64) -> Self {
        Self {
            secs: secs.trunc() as i64,
            attos: (secs.fract() * ATTOS_PER_SEC_F64) as i64,
        }
    }

    /// The seconds part
    #[inline]
    #[must_use]
    pub const fn seconds_part(&self) -> i64 {
        self.secs
    }

    /// The sub-second attoseconds part
    #[inline]
    #[must_use]
    pub const fn attos_part(&self) -> i64 {
        self.attos
    }

    /// The full value expressed in attoseconds. Returns None on overflow.
    ///
    /// This overflows on durations more than about 18 seconds.
    #[must_use]
    pub const fn as_attos(&self) -> Option<i64> {
        let Some(sec_part) = self.secs.checked_mul(ATTOS_PER_SEC_I64) else {
            return None;
        };
        sec_part.checked_add(self.attos)
    }

    /// As number of seconds expressed as an `f64`.
    ///
    /// Precision is limited by the `f64` representation, especially for
    /// durations with a large whole-seconds component.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn as_f64_seconds(&self) -> f64 {
        self.seconds_part() as f64 + self.attos_part() as f64 / ATTOS_PER_SEC_F64
    }

    /// Determine if the duration is zero
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.attos == 0
    }

    /// Determine if the duration is negative.
    ///
    /// `Duration` values are normalized, so this is equivalent to testing
    /// whether the seconds component is negative or the sub-second component
    /// is negative when the seconds component is zero.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.secs < 0 || (self.secs == 0 && self.attos < 0)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // herein we reflect through 0, so no div_modulo.
        // We only show a negative sign at the front
        if self.secs < 0 {
            write!(f, "-P")?; // negative period designator
        } else {
            write!(f, "P")?; // period designator
        }

        let mut s = self.secs.abs();
        let a = self.attos_part().abs();

        let days = s / 86400;
        s %= 86400; // only days should show any negative values
        if days != 0 {
            write!(f, "{days}D")?;
        }

        if s != 0 || a != 0 {
            write!(f, "T")?;
        }

        let hours = s / 3600;
        s %= 3600;
        if hours != 0 {
            write!(f, "{hours}H")?;
        }

        let minutes = s / 60;
        s %= 60;
        if minutes != 0 {
            write!(f, "{minutes}M")?;
        }
        if s != 0 || a != 0 {
            if a == 0 {
                write!(f, "{s}S")?;
            } else {
                write!(f, "{s}.{a:018}S")?;
            }
        }
        Ok(())
    }
}

impl Neg for Duration {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            secs: -self.secs,
            attos: -self.attos,
        }
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut d = Self {
            secs: self.secs + rhs.secs,
            attos: self.attos + rhs.attos,
        };
        d.normalize();
        d
    }
}

impl AddAssign<Self> for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.secs += rhs.secs;
        self.attos += rhs.attos;
        self.normalize();
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut d = Self {
            secs: self.secs - rhs.secs,
            attos: self.attos - rhs.attos,
        };
        d.normalize();
        d
    }
}

impl SubAssign<Self> for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        self.secs -= rhs.secs;
        self.attos -= rhs.attos;
        self.normalize();
    }
}

impl Mul<f64> for Duration {
    type Output = Self;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn mul(self, rhs: f64) -> Self::Output {
        let newsecs = self.secs as f64 * rhs;
        let secs = newsecs.trunc() as i64;
        let overflow_attos = (newsecs.fract() * ATTOS_PER_SEC_F64) as i64;

        let mut d = Self {
            secs,
            attos: ((self.attos as f64) * rhs) as i64 + overflow_attos,
        };
        d.normalize();
        d
    }
}

impl Mul<Duration> for f64 {
    type Output = Duration;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn mul(self, rhs: Duration) -> Self::Output {
        let newsecs = rhs.secs as Self * self;
        let secs = newsecs.trunc() as i64;
        let overflow_attos = (newsecs.fract() * ATTOS_PER_SEC_F64) as i64;

        let mut d = Duration {
            secs,
            attos: ((rhs.attos as Self) * self) as i64 + overflow_attos,
        };
        d.normalize();
        d
    }
}

impl MulAssign<f64> for Duration {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn mul_assign(&mut self, rhs: f64) {
        let newsecs = self.secs as f64 * rhs;
        self.secs = newsecs.trunc() as i64;

        let overflow_attos = (newsecs.fract() * ATTOS_PER_SEC_F64) as i64;
        self.attos = ((self.attos as f64) * rhs) as i64 + overflow_attos;

        self.normalize();
    }
}

impl Div<f64> for Duration {
    type Output = Self;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn div(self, rhs: f64) -> Self::Output {
        assert!(rhs != 0.0, "cannot divide Duration by zero");
        assert!(rhs.is_finite(), "Duration divisor must be finite");
        let newsecs = self.secs as f64 / rhs;
        let secs = newsecs.trunc() as i64;
        let overflow_attos = (newsecs.fract() * ATTOS_PER_SEC_F64) as i64;

        let mut d = Self {
            secs,
            attos: ((self.attos as f64) / rhs) as i64 + overflow_attos,
        };
        d.normalize();
        d
    }
}

impl DivAssign<f64> for Duration {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn div_assign(&mut self, rhs: f64) {
        assert!(rhs != 0.0, "cannot divide Duration by zero");
        assert!(rhs.is_finite(), "Duration divisor must be finite");
        let newsecs = self.secs as f64 / rhs;
        self.secs = newsecs.trunc() as i64;

        let overflow_attos = (newsecs.fract() * ATTOS_PER_SEC_F64) as i64;
        self.attos = ((self.attos as f64) / rhs) as i64 + overflow_attos;

        self.normalize();
    }
}

impl TryFrom<std::time::Duration> for Duration {
    type Error = crate::error::Error;

    #[allow(clippy::cast_lossless)]
    #[allow(clippy::cast_possible_wrap)]
    fn try_from(d: std::time::Duration) -> Result<Self, Self::Error> {
        if d.as_secs() > i64::MAX as u64 {
            // Duration will not fit! (and is ridiculously long)
            return Err(crate::error::Error::RangeError);
        }
        Ok(Self {
            secs: d.as_secs() as i64,
            attos: d.subsec_nanos() as i64 * 1_000_000_000,
        })
    }
}

/// Converts to `std::time::Duration`, truncating any precision finer than
/// nanoseconds. Negative durations return [`crate::Error::RangeError`].
impl TryFrom<Duration> for std::time::Duration {
    type Error = crate::error::Error;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn try_from(d: Duration) -> Result<Self, Self::Error> {
        if d.is_negative() {
            return Err(crate::error::Error::RangeError);
        }
        Ok(Self::new(
            d.seconds_part().cast_unsigned(),
            (d.attos_part() / 1_000_000_000) as u32,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::Duration;
    use crate::ATTOS_PER_SEC_I64;

    #[test]
    fn test_duration_f64_seconds() {
        assert_eq!(
            Duration::new(12, 345_000_000_000_000_000).as_f64_seconds(),
            12.345
        );
        assert_eq!(
            Duration::new(-12, -345_000_000_000_000_000).as_f64_seconds(),
            -12.345
        );
        assert_eq!(Duration::new(0, -1).as_f64_seconds(), -1e-18);
    }

    #[test]
    fn test_duration_is_negative() {
        assert!(!Duration::new(0, 0).is_negative());
        assert!(!Duration::new(1, -1).is_negative());
        assert!(Duration::new(0, -1).is_negative());
        assert!(Duration::new(-1, 1).is_negative());
    }

    #[test]
    fn test_duration_to_std_duration() {
        assert_eq!(
            std::time::Duration::try_from(Duration::new(12, 345_678_901_234_567_890)).unwrap(),
            std::time::Duration::new(12, 345_678_901),
        );
        assert_eq!(
            std::time::Duration::try_from(Duration::new(0, 999_999_999_999_999_999)).unwrap(),
            std::time::Duration::new(0, 999_999_999),
        );
        assert!(matches!(
            std::time::Duration::try_from(Duration::new(0, -1)),
            Err(crate::Error::RangeError),
        ));
        assert!(matches!(
            std::time::Duration::try_from(Duration::new(-1, 0)),
            Err(crate::Error::RangeError),
        ));
    }

    #[test]
    fn test_duration_normalize() {
        let mut d = Duration {
            secs: 12,
            attos: -15,
        };
        d.normalize();
        assert_eq!(d.secs, 11);
        assert_eq!(d.attos, ATTOS_PER_SEC_I64 - 15);

        let mut d = Duration {
            secs: -1,
            attos: 1_100_000_000_000_000_000,
        };
        d.normalize();
        assert_eq!(d.secs, 0);
        assert_eq!(d.attos, 100_000_000_000_000_000);
    }

    #[test]
    fn test_add_duration() {
        let d1 = Duration {
            secs: 8000,
            attos: 12000,
        };
        let d2 = Duration {
            secs: 788,
            attos: 15000,
        };
        let d3 = d1 + d2;
        assert_eq!(d3.secs, 8788);
        assert_eq!(d3.attos, 27000);

        let d1 = Duration {
            secs: -1,
            attos: -101,
        };
        let d2 = Duration { secs: 5, attos: 31 };
        let d3 = d1 + d2;
        assert_eq!(d3.secs, 3);
        assert_eq!(d3.attos, 999_999_999_999_999_930);
    }

    #[test]
    fn test_sub_duration_vs_neg() {
        let d1 = Duration {
            secs: 8000,
            attos: 12000,
        };
        let d2 = Duration {
            secs: 788,
            attos: 15000,
        };
        let d3 = d1 - d2;
        let d4 = d1 + (-d2);
        assert_eq!(d3, d4);
        assert_eq!(d3.secs, 7211);
        assert_eq!(d3.attos, 999_999_999_999_997_000);
    }

    #[test]
    fn test_duration_display() {
        let d = Duration {
            secs: 86400 * 100,
            attos: 12000,
        };
        assert_eq!(&*format!("{}", d), "P100DT0.000000000000012000S");
        let d = Duration {
            secs: 86400 + 3600 * 2 + 60 + 1,
            attos: 120,
        };
        assert_eq!(&*format!("{}", d), "P1DT2H1M1.000000000000000120S");
        let d = Duration {
            secs: 60 * 3 + 5,
            attos: 15000,
        };
        assert_eq!(&*format!("{}", d), "PT3M5.000000000000015000S");
        let d = Duration {
            secs: -1,
            attos: -101,
        };
        assert_eq!(&*format!("{}", d), "-PT1.000000000000000101S");
        let d = Duration {
            secs: -86400 * 3,
            attos: 31,
        };
        assert_eq!(&*format!("{}", d), "-P3DT0.000000000000000031S");
        let d = Duration { secs: 0, attos: 31 };
        assert_eq!(&*format!("{}", d), "PT0.000000000000000031S");
        let d = Duration { secs: 0, attos: 0 };
        assert_eq!(&*format!("{}", d), "P");
    }

    #[test]
    fn test_add_assign() {
        let d1 = Duration {
            secs: 8_640_000,
            attos: 12000,
        };
        let d2 = Duration {
            secs: -16500,
            attos: -999_999_999_999_997_000,
        };

        let mut x = d1;
        x += d2;
        assert_eq!(x.secs, 8_623_499);
        assert_eq!(x.attos, 15_000);
    }

    #[test]
    fn test_mul_f64() {
        let d = Duration {
            secs: 2,
            attos: 500_000_000_000_000_000, // 2.5s
        };
        let m = d * 2.0;
        assert_eq!(m.secs, 5);
        assert_eq!(m.attos, 0);

        let m2 = 2.0 * d;
        assert_eq!(m2.secs, 5);
        assert_eq!(m2.attos, 0);

        let m3 = d * 1.5;
        assert_eq!(m3.secs, 3);
        assert_eq!(m3.attos, 750_000_000_000_000_000);

        let d2 = Duration {
            secs: -2,
            attos: -500_000_000_000_000_000,
        };
        let m4 = d2 * 1.5;
        assert_eq!(m4.secs, -3);
        assert_eq!(m4.attos, -750_000_000_000_000_000);
    }

    #[test]
    fn test_mul_assign_f64() {
        let mut d = Duration {
            secs: 2,
            attos: 500_000_000_000_000_000,
        };
        d *= 1.5;
        assert_eq!(d.secs, 3);
        assert_eq!(d.attos, 750_000_000_000_000_000);
    }

    #[test]
    fn test_div_f64() {
        let d = Duration { secs: 5, attos: 0 };
        let m = d / 2.0;
        assert_eq!(m.secs, 2);
        assert_eq!(m.attos, 500_000_000_000_000_000);

        let d2 = Duration { secs: 1, attos: 0 };
        let m2 = d2 / 3.0;
        assert_eq!(m2.secs, 0);
        // 1/3 = 0.3333... so 333,333,333,333,333,333 attoseconds
        // with precision of f64, we might lose some low bits
        let expected_attos = (1.0f64 / 3.0f64 * crate::ATTOS_PER_SEC_F64) as i64;
        assert_eq!(m2.attos, expected_attos);

        let d3 = Duration { secs: -5, attos: 0 };
        let m3 = d3 / 2.0;
        assert_eq!(m3.secs, -2);
        assert_eq!(m3.attos, -500_000_000_000_000_000);
    }

    #[test]
    fn test_div_assign_f64() {
        let mut d = Duration { secs: 5, attos: 0 };
        d /= 2.0;
        assert_eq!(d.secs, 2);
        assert_eq!(d.attos, 500_000_000_000_000_000);
    }

    #[test]
    #[should_panic(expected = "cannot divide Duration by zero")]
    fn test_div_f64_by_zero_panics() {
        let _ = Duration::new(1, 0) / 0.0;
    }

    #[test]
    #[should_panic(expected = "Duration divisor must be finite")]
    fn test_div_f64_by_infinity_panics() {
        let _ = Duration::new(1, 0) / f64::INFINITY;
    }
}
