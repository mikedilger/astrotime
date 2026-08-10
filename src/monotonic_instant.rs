use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::Duration;

/// A precise, opaque point on a monotonic clock.
///
/// Unlike [`crate::DatedInstant`], a `MonotonicInstant` has no epoch and cannot
/// be converted to a calendar date. It is useful for measuring elapsed time,
/// setting deadlines, and ordering events within one monotonic clock domain.
///
/// The underlying clock is [`std::time::Instant`]. Consequently, the clock is
/// monotonic but its precision and range are platform-dependent, and durations
/// added to or subtracted from this type are truncated to nanoseconds.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MonotonicInstant(std::time::Instant);

impl MonotonicInstant {
    /// Return the current point on the platform's monotonic clock.
    #[must_use]
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    /// Return the elapsed time since this instant.
    #[must_use]
    pub fn elapsed(self) -> Duration {
        Self::now() - self
    }

    /// Return the signed duration from `earlier` to this instant.
    #[must_use]
    pub fn duration_since(self, earlier: Self) -> Duration {
        self - earlier
    }

    /// Return the non-negative duration from `earlier` to this instant.
    #[must_use]
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        if self < earlier {
            None
        } else {
            Some(self - earlier)
        }
    }

    /// Return the non-negative duration from `earlier` to this instant,
    /// returning zero when `earlier` is later than this instant.
    #[must_use]
    pub fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or(Duration::new(0, 0))
    }

    /// Return this instant shifted by `duration`, or `None` if the platform
    /// instant cannot represent the result.
    #[must_use]
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        let (negative, duration) = duration_to_std(duration)?;
        let instant = if negative {
            self.0.checked_sub(duration)
        } else {
            self.0.checked_add(duration)
        }?;
        Some(Self(instant))
    }

    /// Return this instant shifted backwards by `duration`, or `None` if the
    /// platform instant cannot represent the result.
    #[must_use]
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        let (negative, duration) = duration_to_std(duration)?;
        let instant = if negative {
            self.0.checked_add(duration)
        } else {
            self.0.checked_sub(duration)
        }?;
        Some(Self(instant))
    }

    /// Return the wrapped standard-library monotonic instant.
    #[must_use]
    pub const fn into_std(self) -> std::time::Instant {
        self.0
    }
}

impl From<std::time::Instant> for MonotonicInstant {
    fn from(instant: std::time::Instant) -> Self {
        Self(instant)
    }
}

impl From<MonotonicInstant> for std::time::Instant {
    fn from(instant: MonotonicInstant) -> Self {
        instant.0
    }
}

impl Add<Duration> for MonotonicInstant {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        self.checked_add(duration)
            .expect("monotonic instant addition overflowed")
    }
}

impl AddAssign<Duration> for MonotonicInstant {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub<Duration> for MonotonicInstant {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        self.checked_sub(duration)
            .expect("monotonic instant subtraction overflowed")
    }
}

impl SubAssign<Duration> for MonotonicInstant {
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

impl Sub<Self> for MonotonicInstant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        if self >= rhs {
            duration_from_std(self.0.duration_since(rhs.0))
        } else {
            -duration_from_std(rhs.0.duration_since(self.0))
        }
    }
}

fn duration_to_std(duration: Duration) -> Option<(bool, std::time::Duration)> {
    let negative = duration.is_negative();
    let seconds = if negative {
        duration.seconds_part().checked_neg()?
    } else {
        duration.seconds_part()
    };
    let attoseconds = if negative {
        duration.attos_part().checked_neg()?
    } else {
        duration.attos_part()
    };

    Some((
        negative,
        std::time::Duration::new(
            u64::try_from(seconds).ok()?,
            u32::try_from(attoseconds / 1_000_000_000).ok()?,
        ),
    ))
}

fn duration_from_std(duration: std::time::Duration) -> Duration {
    Duration::try_from(duration).expect("standard duration must fit astrotime")
}

#[cfg(test)]
mod tests {
    use super::MonotonicInstant;
    use crate::Duration;

    #[test]
    fn now_is_ordered_and_elapsed_is_non_negative() {
        let first = MonotonicInstant::now();
        let second = MonotonicInstant::now();

        assert!(second >= first);
        assert!(!first.elapsed().is_negative());
        assert!(!second.duration_since(first).is_negative());
    }

    #[test]
    fn subtraction_preserves_signed_duration() {
        let first = MonotonicInstant::now();
        let second = first + Duration::new(2, 500_000_000_000_000_000);

        assert_eq!(second - first, Duration::new(2, 500_000_000_000_000_000));
        assert_eq!(first - second, Duration::new(-2, -500_000_000_000_000_000));
        assert_eq!(second.duration_since(first), second - first);
        assert_eq!(first.checked_duration_since(second), None);
        assert_eq!(first.saturating_duration_since(second), Duration::new(0, 0));
    }

    #[test]
    fn addition_and_subtraction_round_trip() {
        let start = MonotonicInstant::now();
        let offset = Duration::new(3, 250_000_000_000_000_000);

        assert_eq!(start + offset - offset, start);
        assert_eq!(start - offset + offset, start);
        assert!(start.checked_add(Duration::new(i64::MIN, 0)).is_none());
        assert!(start.checked_sub(Duration::new(i64::MIN, 0)).is_none());
    }

    #[test]
    fn standard_instant_conversion_preserves_value() {
        let standard = std::time::Instant::now();
        let monotonic = MonotonicInstant::from(standard);

        assert_eq!(monotonic.into_std(), standard);
        assert_eq!(std::time::Instant::from(monotonic), standard);
    }

    #[test]
    fn subnanosecond_arithmetic_is_truncated_at_the_platform_boundary() {
        let start = MonotonicInstant::now();
        let shifted = start + Duration::new(0, 1);

        assert_eq!(shifted - start, Duration::new(0, 0));
    }
}
