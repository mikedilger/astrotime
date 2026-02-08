use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::calendar::Calendar;
use crate::date_time::DateTime;
use crate::duration::Duration;
use crate::epoch::Epoch;
use crate::error::Error;
use crate::standard::Standard;
use crate::{ATTOS_PER_SEC_F64, ATTOS_PER_SEC_I64};

/// An `Instant` is a precise moment in time.
///
/// Internally this is stored as a Duration (which is 128 bits in size) offset from
/// an opaque internally chosen epoch.
///
/// This represents the same thing that a `DateTime` does, but has advantages:
/// * It is easier to work with Durations by avoiding the complexity of the `Calendar`
/// * Spans a much larger time span, able to handle times from about 20 times as old
///   as the age of the/ universe backwards, and the same distance forwards
/// * Provides attosecond (10^-18) precision.
//
// Internally, Instants are Duration offsets from `Epoch::TimeStandard`, which is
// January 1st, 1977 CE Gregorian, 00:00:00.000 TAI
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Instant(pub(crate) Duration);

impl Instant {
    /// Create from a Julian Day (low precision)
    ///
    /// This is not as precise as `from_julian_day_parts`(), and much less precise than
    /// `from_julian_day_precise`()
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_julian_day_f64(jd: f64) -> Self {
        let fsecs = jd * 86400.0;
        let whole_secs = fsecs.trunc() as i64;
        let attos = (fsecs.fract() * ATTOS_PER_SEC_F64) as i64;
        Epoch::JulianPeriod.as_instant() + Duration::new(whole_secs, attos)
    }

    /// Create from a Julian Day (medium precision)
    ///
    /// This is more precise than `from_julian_day_f64`() but not as precise as
    /// `from_julian_day_precise`()
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_julian_day_parts(day: i64, day_fraction: f64) -> Self {
        // FIXME - range bound this
        let fsecs = day_fraction * 86400.;
        let mut whole_secs = fsecs.trunc() as i64;
        let attos = (fsecs.fract() * ATTOS_PER_SEC_F64) as i64;
        whole_secs += day * 86400;
        Epoch::JulianPeriod.as_instant() + Duration::new(whole_secs, attos)
    }

    /// Create from a Julian Day (maximum precision)
    ///
    /// This is more precise than `from_julian_day_f64`() but not as precise as
    /// `from_julian_day_precise`()
    ///
    /// # Errors
    ///
    /// This will throw an `Error::RangeError` if the seconds are out of
    /// bounds (`0` <= `seconds` < `86_400`) or the attoseconds are out of bounds
    /// (`0` <= `attoseconds` < `ATTOS_PER_SEC_U64`)
    #[allow(clippy::manual_range_contains)]
    pub fn from_julian_day_precise(
        day: i64,
        seconds: u32,
        attoseconds: i64,
    ) -> Result<Self, Error> {
        if seconds >= 86400 {
            return Err(Error::RangeError);
        }
        if attoseconds < 0 || attoseconds >= ATTOS_PER_SEC_I64 {
            return Err(Error::RangeError);
        }
        let secs = day * 86400 + i64::from(seconds);
        Ok(Epoch::JulianPeriod.as_instant() + Duration::new(secs, attoseconds))
    }

    /// As Julian day (low precision)
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_julian_day_f64(&self) -> f64 {
        let since = *self - Epoch::JulianPeriod.as_instant();
        (since.secs as f64 + since.attos as f64 / ATTOS_PER_SEC_F64) / 86400.
    }

    /// As Julian day (medium precision)
    ///
    /// This returns a day number and a day fraction.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_julian_day_parts(&self) -> (i64, f64) {
        let since = *self - Epoch::JulianPeriod.as_instant();
        let day = since.secs / 86400;
        let rem = since.secs % 86400;
        let frac = (rem as f64 + since.attos as f64 / ATTOS_PER_SEC_F64) / 86400.;
        (day, frac)
    }

    /// As Julian day (high precision)
    ///
    /// This returns a day number, a second number, and an attoseconds number
    #[must_use]
    pub fn as_julian_day_precise(&self) -> (i64, i64, i64) {
        let since = *self - Epoch::JulianPeriod.as_instant();
        let day = since.secs / 86400;
        let secs = since.secs % 86400;
        (day, secs, since.attos)
    }

    /// As julian day (formatted as a string)
    #[must_use]
    pub fn as_julian_day_formatted(&self) -> String {
        let (day, frac) = self.as_julian_day_parts();
        let fraction = format!("{frac}").trim_start_matches(['-', '0']).to_owned();
        format!("JD {day}{fraction}")
    }

    /// As an NTP date, which is seconds and attoseconds.
    ///
    /// This value is not returned as a Duration because NTP dates are
    /// incorrect durations from the NTP epoch (in that they don't include
    /// leap seconds).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_ntp_date(&self) -> (i64, i64) {
        let ntp_dur = *self - Epoch::Ntp.as_instant();
        let leaps = crate::leaps::leap_seconds_elapsed_at(*self);
        (ntp_dur.secs - leaps, ntp_dur.attos)
    }

    /// From an NTP date
    #[must_use]
    pub fn from_ntp_date(ntp_secs: i64, ntp_attos: i64) -> Self {
        let close = Epoch::Ntp.as_instant() + Duration::new(ntp_secs, ntp_attos);
        let approx_leaps = crate::leaps::leap_seconds_elapsed_at(close);
        let actual_leaps =
            crate::leaps::leap_seconds_elapsed_at(close + Duration::new(approx_leaps + 1, 0));
        close + Duration::new(actual_leaps, 0)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(self.0.add(rhs))
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs;
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self(self.0.sub(rhs))
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs;
    }
}

impl Sub<Self> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        self.0 - rhs.0
    }
}

impl<C: Calendar, S: Standard> From<Instant> for DateTime<C, S> {
    fn from(i: Instant) -> Self {
        Self::from_duration_from_epoch(i - C::epoch())
    }
}

impl<C: Calendar, S: Standard> From<DateTime<C, S>> for Instant {
    fn from(dt: DateTime<C, S>) -> Self {
        C::epoch() + dt.duration_from_epoch()
    }
}

impl TryFrom<std::time::SystemTime> for Instant {
    type Error = Error;

    fn try_from(s: std::time::SystemTime) -> Result<Self, Self::Error> {
        // NOTE: std::time::SystemTime, like UNIX, lies about UTC times
        //       in the past that cross leap seconds. When we compute the
        //       duration_since(UNIX_EPOCH), we get a number that is short
        //       by the total number of leap seconds that have occured.
        //       We correct for this below.
        let since_unix_epoch_less_leaps: Duration = match s.duration_since(std::time::UNIX_EPOCH) {
            Ok(std_dur) => TryFrom::try_from(std_dur)?,
            Err(std_time_error) => {
                // we can handle negative durations ;-P
                let d: Duration = TryFrom::try_from(std_time_error.duration())?;
                -d
            }
        };

        let mut instant = Epoch::Unix.as_instant() + since_unix_epoch_less_leaps;

        // Add the missing leap seconds
        let leaps = crate::leap_seconds_elapsed_at(instant);
        instant += Duration::new(leaps, 0);

        // That correction might have caused the instant to move past yet
        // one more leap second.  If so we should add it.
        let leaps2 = crate::leap_seconds_elapsed_at(instant);
        instant += Duration::new(leaps2 - leaps, 0);

        Ok(instant)
    }
}

#[cfg(test)]
mod test {
    use super::Instant;
    use crate::ATTOS_PER_SEC_I64;
    use crate::calendar::Gregorian;
    use crate::date_time::DateTime;
    use crate::duration::Duration;
    use crate::epoch::Epoch;
    use crate::standard::{Tai, Utc};

    #[test]
    fn test_instant_ntp_conversions() {
        let y1977 = Epoch::Y1977.as_instant();
        let start = y1977 - Duration::new(3, ATTOS_PER_SEC_I64 / 2);
        let end = y1977 - Duration::new(3, ATTOS_PER_SEC_I64 / 2);

        let increment = Duration::new(0, ATTOS_PER_SEC_I64 / 2);
        let mut instant = start;
        while instant < end {
            let (a, b) = instant.as_ntp_date();
            let instant2 = Instant::from_ntp_date(a, b);
            assert_eq!(instant, instant2);

            instant += increment;
        }
    }
    #[test]
    fn test_instant_julian_day_conversions() {
        assert_eq!(
            Instant::from_julian_day_parts(1721425, 0.5),
            Epoch::GregorianCalendar.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(1721423, 0.5),
            Epoch::JulianCalendar.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(0, 0.0),
            Epoch::JulianPeriod.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(2415020, 0.0),
            Epoch::J1900_0.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(2448349, 0.0625),
            Epoch::J1991_25.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(2451545, 0.0),
            Epoch::J2000_0.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(2488070, 0.0),
            Epoch::J2100_0.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_parts(2524595, 0.0),
            Epoch::J2200_0.as_instant()
        );

        assert_eq!(
            Instant::from_julian_day_precise(2440587, 43200 + 41, 184_000_000_000_000_000).unwrap(),
            Epoch::Unix.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_precise(2451544, 43200 + 64, 184_000_000_000_000_000).unwrap(),
            Epoch::Y2k.as_instant()
        );

        assert_eq!(
            Instant::from_julian_day_precise(2443144, 43200 + 32, 184_000_000_000_000_000).unwrap(),
            Epoch::TimeStandard.as_instant()
        );
    }

    #[test]
    fn test_time_standard_conversions() {
        let p: Instant =
            From::from(DateTime::<Gregorian, Tai>::new(1993, 6, 30, 0, 0, 27, 0).unwrap());
        let q: DateTime<Gregorian, Utc> = From::from(p);
        assert_eq!(
            q,
            DateTime::<Gregorian, Utc>::new(1993, 6, 30, 0, 0, 0, 0).unwrap()
        );

        let p: Instant = Epoch::Unix.as_instant();
        let q: DateTime<Gregorian, Utc> = From::from(p);
        assert_eq!(
            q,
            DateTime::<Gregorian, Utc>::new(1970, 1, 1, 0, 0, 0, 0).unwrap()
        );

        let y2k: Instant = Epoch::Y2k.as_instant();
        let q: DateTime<Gregorian, Utc> = From::from(y2k);
        assert_eq!(
            q,
            DateTime::<Gregorian, Utc>::new(2000, 1, 1, 0, 0, 0, 0).unwrap()
        );
    }
}
