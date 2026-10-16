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

    /// The longest positive `Duration`.
    pub const MAX: Self = Self {
        secs: i64::MAX,
        attos: ATTOS_PER_SEC_I64 - 1,
    };

    /// The longest negative `Duration`.
    pub const MIN: Self = Self {
        secs: i64::MIN,
        attos: -(ATTOS_PER_SEC_I64 - 1),
    };

    /// Make a new `Duration` with given number of seconds and attoseconds.
    #[must_use]
    pub const fn new(secs: i64, attos: i64) -> Self {
        let mut d = Self { secs, attos };
        d.normalize();
        d
    }

    /// Make a new `Duration` from seconds as `f64`.
    ///
    /// # Panics
    ///
    /// Panics if `secs` is not a number, or is out of range.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn from_seconds(secs: f64) -> Self {
        // 2^63, the first whole second beyond i64
        const LIMIT: f64 = 9_223_372_036_854_775_808.0;
        assert!(
            secs >= -LIMIT && secs < LIMIT,
            "Duration seconds out of range"
        );
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

    /// Checked multiply by a `u64`.
    ///
    /// Returns `None` on overflow
    #[must_use]
    pub fn checked_mul(&self, multiplier: u64) -> Option<Self> {
        let total_attos = i128::from(self.secs)
            .checked_mul(crate::ATTOS_PER_SEC_I128)?
            .checked_add(i128::from(self.attos))?
            .checked_mul(i128::from(multiplier))?;

        let secs = total_attos.div_euclid(crate::ATTOS_PER_SEC_I128);
        let attos = total_attos.rem_euclid(crate::ATTOS_PER_SEC_I128);

        Some(Self {
            secs: i64::try_from(secs).ok()?,
            attos: i64::try_from(attos).ok()?,
        })
    }

    /// Checked addition. Returns `None` on overflow.
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        Self::from_attos_i128(self.as_attos_i128() + rhs.as_attos_i128())
    }

    /// Checked subtraction. Returns `None` on overflow.
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Self::from_attos_i128(self.as_attos_i128() - rhs.as_attos_i128())
    }

    /// Checked negation. Returns `None` on overflow.
    #[must_use]
    pub fn checked_neg(self) -> Option<Self> {
        Self::from_attos_i128(-self.as_attos_i128())
    }

    /// Saturating addition, stopping at `MAX` or `MIN`.
    #[must_use]
    pub fn saturating_add(self, rhs: Self) -> Self {
        match self.checked_add(rhs) {
            Some(d) => d,
            None if rhs.is_negative() => Self::MIN,
            None => Self::MAX,
        }
    }

    /// Saturating subtraction, stopping at `MAX` or `MIN`.
    #[must_use]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        match self.checked_sub(rhs) {
            Some(d) => d,
            None if rhs.is_negative() => Self::MAX,
            None => Self::MIN,
        }
    }

    /// Multiply by an `f64`, to `f64` precision. Returns `None` on overflow,
    /// or if the result is not a number.
    #[must_use]
    pub fn checked_mul_f64(self, rhs: f64) -> Option<Self> {
        self.scale_f64(|x| x * rhs)
    }

    /// Divide by an `f64`, to `f64` precision. Returns `None` on overflow,
    /// or if the result is not a number (such as dividing by zero).
    #[must_use]
    pub fn checked_div_f64(self, rhs: f64) -> Option<Self> {
        self.scale_f64(|x| x / rhs)
    }

    // Scale the seconds and attoseconds separately, and add them up in i128
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn scale_f64(self, f: impl Fn(f64) -> f64) -> Option<Self> {
        let secs = f(self.secs as f64);
        let attos = f(self.attos as f64);
        // beyond this the seconds don't fit anyway
        if !(secs.abs() < 1e19 && attos.abs() < 1e37) {
            return None;
        }
        let total = secs.trunc() as i128 * crate::ATTOS_PER_SEC_I128
            + (secs.fract() * ATTOS_PER_SEC_F64) as i128
            + attos as i128;
        Self::from_attos_i128(total)
    }

    /// The full value expressed in attoseconds, which always fits an `i128`.
    #[must_use]
    pub fn as_attos_i128(self) -> i128 {
        i128::from(self.secs) * crate::ATTOS_PER_SEC_I128 + i128::from(self.attos)
    }

    /// Make a new `Duration` from attoseconds. Returns `None` if out of range.
    #[must_use]
    pub fn from_attos_i128(attos: i128) -> Option<Self> {
        // Truncating division keeps attos the same sign as secs
        Some(Self {
            secs: i64::try_from(attos / crate::ATTOS_PER_SEC_I128).ok()?,
            attos: i64::try_from(attos % crate::ATTOS_PER_SEC_I128).ok()?,
        })
    }

    /// Checked division by a `u64`.
    ///
    /// The result is truncated toward zero at attosecond precision. Returns
    /// `None` when `divisor` is zero.
    #[must_use]
    pub fn checked_div(&self, divisor: u64) -> Option<Self> {
        if divisor == 0 {
            return None;
        }

        let total_attos = i128::from(self.secs)
            .checked_mul(crate::ATTOS_PER_SEC_I128)?
            .checked_add(i128::from(self.attos))?;
        let quotient = total_attos / i128::from(divisor);

        // `/` on signed integers truncates toward zero. Splitting the result
        // with `/` and `%` preserves that behavior and the Duration sign
        // invariant (unlike div_euclid/rem_euclid for negative values).
        let secs = quotient / crate::ATTOS_PER_SEC_I128;
        let attos = quotient % crate::ATTOS_PER_SEC_I128;

        Some(Self {
            secs: i64::try_from(secs).ok()?,
            attos: i64::try_from(attos).ok()?,
        })
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // herein we reflect through 0, so no div_modulo.
        // We only show a negative sign at the front
        if self.is_negative() {
            write!(f, "-P")?; // negative period designator
        } else {
            write!(f, "P")?; // period designator
        }

        let mut s = self.secs.unsigned_abs();
        let a = self.attos_part().unsigned_abs();

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
        self.checked_neg().expect("overflow when negating duration")
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl AddAssign<Self> for Duration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs)
            .expect("overflow when subtracting durations")
    }
}

impl SubAssign<Self> for Duration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f64> for Duration {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        self.checked_mul_f64(rhs)
            .expect("overflow when multiplying duration by float")
    }
}

impl Mul<Duration> for f64 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<f64> for Duration {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl Div<f64> for Duration {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        assert!(rhs != 0.0, "cannot divide Duration by zero");
        assert!(rhs.is_finite(), "Duration divisor must be finite");
        self.checked_div_f64(rhs)
            .expect("overflow when dividing duration by float")
    }
}

impl DivAssign<f64> for Duration {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
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
    fn test_from_seconds() {
        assert_eq!(
            Duration::from_seconds(-1.5),
            Duration::new(-1, -500_000_000_000_000_000)
        );
        assert_eq!(
            Duration::from_seconds(-9_223_372_036_854_775_808.0).secs,
            i64::MIN
        );
    }

    #[test]
    #[should_panic(expected = "Duration seconds out of range")]
    fn test_from_seconds_out_of_range() {
        let _ = Duration::from_seconds(1e19);
    }

    #[test]
    #[should_panic(expected = "Duration seconds out of range")]
    fn test_from_seconds_nan() {
        let _ = Duration::from_seconds(f64::NAN);
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
        assert_eq!(&*format!("{d}"), "P100DT0.000000000000012000S");
        let d = Duration {
            secs: 86400 + 3600 * 2 + 60 + 1,
            attos: 120,
        };
        assert_eq!(&*format!("{d}"), "P1DT2H1M1.000000000000000120S");
        let d = Duration {
            secs: 60 * 3 + 5,
            attos: 15000,
        };
        assert_eq!(&*format!("{d}"), "PT3M5.000000000000015000S");
        let d = Duration {
            secs: -1,
            attos: -101,
        };
        assert_eq!(&*format!("{d}"), "-PT1.000000000000000101S");
        let d = Duration {
            secs: -86400 * 3,
            attos: 31,
        };
        assert_eq!(&*format!("{d}"), "-P3DT0.000000000000000031S");
        let d = Duration { secs: 0, attos: 31 };
        assert_eq!(&*format!("{d}"), "PT0.000000000000000031S");
        let d = Duration { secs: 0, attos: 0 };
        assert_eq!(&*format!("{d}"), "P");
        assert_eq!(
            &*format!("{}", Duration::MIN),
            "-P106751991167300DT15H30M8.999999999999999999S"
        );
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
    fn test_checked_and_saturating() {
        let one = Duration::new(0, 1);
        assert_eq!(Duration::MAX.checked_add(one), None);
        assert_eq!(Duration::MIN.checked_sub(one), None);
        assert_eq!(Duration::MIN.checked_neg(), None);
        assert!(Duration::MAX.checked_neg().is_some());
        assert_eq!(Duration::MAX.saturating_add(one), Duration::MAX);
        assert_eq!(Duration::MIN.saturating_add(-one), Duration::MIN);
        assert_eq!(Duration::MAX.saturating_sub(-one), Duration::MAX);
        assert_eq!(Duration::MIN.saturating_sub(one), Duration::MIN);
        assert_eq!(
            Duration::MAX.checked_sub(one),
            Some(Duration::new(i64::MAX, ATTOS_PER_SEC_I64 - 2))
        );
        assert_eq!(
            Duration::new(1, 0).checked_sub(Duration::new(0, 1)),
            Some(Duration::new(0, ATTOS_PER_SEC_I64 - 1))
        );
        assert_eq!(
            Duration::new(-1, 0).checked_add(Duration::new(0, 1)),
            Some(Duration::new(0, -(ATTOS_PER_SEC_I64 - 1)))
        );
    }

    #[test]
    #[should_panic(expected = "overflow when adding durations")]
    fn test_add_overflow_panics() {
        let _ = Duration::MAX + Duration::new(0, 1);
    }

    #[test]
    #[should_panic(expected = "overflow when subtracting durations")]
    fn test_sub_overflow_panics() {
        let _ = Duration::MIN - Duration::new(0, 1);
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
    fn test_mul_div_f64_large() {
        let half = Duration::new(0, 500_000_000_000_000_000);
        assert_eq!(half * 100.0, Duration::new(50, 0));
        assert_eq!(half / 0.01, Duration::new(50, 0));
        assert_eq!(-half * 4096.0, Duration::new(-2048, 0));
        let big = Duration::new(10_000_000_000, 0);
        assert_eq!(big.checked_mul_f64(1e10), None);
        assert_eq!(big.checked_div_f64(1e-10), None);
        assert_eq!(big.checked_mul_f64(f64::NAN), None);
        assert_eq!(big.checked_div_f64(0.0), None);
        assert_eq!(
            big.checked_mul_f64(0.5),
            Some(Duration::new(5_000_000_000, 0))
        );
    }

    #[test]
    #[should_panic(expected = "overflow when multiplying duration by float")]
    fn test_mul_f64_overflow_panics() {
        let _ = Duration::new(10_000_000_000, 0) * 1e10;
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
        #[allow(clippy::cast_possible_truncation)]
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

    #[test]
    fn test_mul_u64() {
        let value = Duration {
            secs: 2,
            attos: 500_000_000_000_000_000,
        };
        let result = value.checked_mul(13).unwrap();
        assert_eq!(result.secs, 32);
        assert_eq!(result.attos, 500_000_000_000_000_000);
    }

    #[test]
    fn test_attos_i128() {
        let d = Duration::new(-3, -250_000_000_000_000_000);
        assert_eq!(d.as_attos_i128(), -3_250_000_000_000_000_000);
        assert_eq!(Duration::from_attos_i128(d.as_attos_i128()), Some(d));
        assert_eq!(
            Duration::from_attos_i128(Duration::MAX.as_attos_i128()),
            Some(Duration::MAX)
        );
        assert_eq!(
            Duration::from_attos_i128(Duration::MIN.as_attos_i128()),
            Some(Duration::MIN)
        );
        assert_eq!(
            Duration::from_attos_i128(Duration::MAX.as_attos_i128() + 1),
            None
        );
    }

    #[test]
    fn test_checked_div_u64() {
        let value = Duration::new(5, 0);
        assert_eq!(
            value.checked_div(2),
            Some(Duration::new(2, 500_000_000_000_000_000))
        );
        assert_eq!(
            Duration::new(1, 0).checked_div(3),
            Some(Duration::new(0, 333_333_333_333_333_333))
        );
        assert_eq!(
            Duration::new(-5, 0).checked_div(2),
            Some(Duration::new(-2, -500_000_000_000_000_000))
        );
        assert_eq!(
            Duration::new(-1, 0).checked_div(3),
            Some(Duration::new(0, -333_333_333_333_333_333))
        );
        assert_eq!(value.checked_div(0), None);
    }
}
