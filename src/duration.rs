use crate::{ATTOS_PER_SEC_F64, ATTOS_PER_SEC_I64};
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    /// Determine if the duration is zero
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.attos == 0
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

impl Mul<f64> for Duration {
    type Output = Self;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn mul(self, rhs: f64) -> Self {
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

#[cfg(test)]
mod test {
    use super::Duration;
    use crate::ATTOS_PER_SEC_I64;

    #[test]
    fn test_duration_normalize() {
        crate::setup_logging();

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
        crate::setup_logging();

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
        crate::setup_logging();

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
        crate::setup_logging();

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
}
