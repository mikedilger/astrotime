
use std::convert::TryFrom;
use std::ops::{Add, Sub};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::calendar::Calendar;
use crate::date_time::DateTime;
use crate::duration::Duration;
use crate::error::Error;
use crate::epoch::Epoch;
use crate::standard::Standard;

/// An `Instant` is a precise moment in time according to a particular time `Standard`.
///
/// Internally this is stored as a Duration (which is 128 bits in size) offset from
/// an internally chosen epoch.
///
/// This represents the same thing that a `DateTime` does, but it makes it easier to work
/// with Durations, avoids the complexity of the `Calendar`, and spans a much larger time
/// span, able to handle times from about 20 times as old as the age of the
/// universe backwards, and the same distance forwards, with attosecond (10^-18) precision.
//
// Internally, Instants are Duration offsets from `Epoch::TimeStandard`, which is
// January 1st, 1977 CE gregorian, 00:00:32.184 Tt
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
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
        let attos = (fsecs.fract() * 1_000_000_000_000_000_000.) as i64;
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
        let attos = (fsecs.fract() * 1_000_000_000_000_000_000.) as i64;
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
    /// (`0` <= `attoseconds` < `1_000_000_000_000_000_000`)
    #[allow(clippy::manual_range_contains)]
    pub fn from_julian_day_precise(day: i64, seconds: u32, attoseconds: i64) -> Result<Self, Error>
    {
        if seconds >= 86400 { return Err(Error::RangeError); }
        if attoseconds < 0 || attoseconds >= 1_000_000_000_000_000_000 { return Err(Error::RangeError); }
        let secs = day * 86400 + i64::from(seconds);
        Ok(Epoch::JulianPeriod.as_instant() + Duration::new(secs, attoseconds))
    }

    /// As Julian day (low precision)
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_julian_day_f64(&self) -> f64 {
        let since = *self - Epoch::JulianPeriod.as_instant();
        (since.secs as f64 + since.attos as f64 / 1_000_000_000_000_000_000.) / 86400.
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
        let frac = (rem as f64 + since.attos as f64 / 1_000_000_000_000_000_000.) / 86400.;
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
        let fraction = format!("{}", frac).trim_start_matches(|c| c=='-' || c=='0')
            .to_owned();
        format!("JD {}{}", day, fraction)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(self.0.add(rhs))
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        Self(self.0.sub(rhs))
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
        // Conversion between time standards
        let dur: Duration = S::from_tt(i.0);

        // NOTE: if we ever move the epoch that Durations are based on
        //       away from TimeStandard, then replace `C::epoch()` below
        //       with `C::epoch() - Epoch::TimeStandard.as_instant()`
        Self::from_duration_from_epoch(dur - C::epoch().0)
    }
}

impl<C: Calendar, S: Standard> From<DateTime<C, S>> for Instant {
    fn from(dt: DateTime<C, S>) -> Self {
        // NOTE: if we ever move the epoch that Durations are based on
        //       away from TimeStandard, then replace `C::epoch()` below
        //       with `C::epoch() - Epoch::TimeStandard.as_instant()`
        let dur: Duration = dt.duration_from_epoch() + C::epoch().0;

        // Conversion between time standards
        Self(S::to_tt(dur))
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

        let since_unix_epoch_less_leaps: Duration =
            match s.duration_since(std::time::UNIX_EPOCH)
        {
            Ok(std_dur) => TryFrom::try_from(std_dur)?,
            Err(std_time_error) => {
                // we can handle negative durations ;-P
                let d: Duration = TryFrom::try_from(std_time_error.duration())?;
                -d
            }
        };

        let time_less_leaps = Epoch::Unix.as_instant() + since_unix_epoch_less_leaps;

        // Add the missing leap seconds in two passes
        let leap_seconds_elapsed_try1 = crate::standard::leap_seconds_elapsed(time_less_leaps);
        let time_maybe_missing_one_leap = time_less_leaps + Duration::new(leap_seconds_elapsed_try1, 0);

        // Check that we didn't hit another leap second in the process
        let leap_seconds_elapsed_try2 = crate::standard::leap_seconds_elapsed(time_maybe_missing_one_leap);

        Ok(if leap_seconds_elapsed_try2 >leap_seconds_elapsed_try1 {
            time_maybe_missing_one_leap + Duration::new(1,0)
        } else {
            time_maybe_missing_one_leap
        })
    }
}

#[cfg(test)]
mod test {
    use super::Instant;
    use crate::calendar::Gregorian;
    use crate::date_time::DateTime;
    use crate::epoch::Epoch;
    use crate::standard::{Utc, Tai};

    #[test]
    fn test_instant_julian_day_conversions() {
        crate::setup_logging();

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
            Instant::from_julian_day_precise(2440587, 43200 + 41, 184_000_000_000_000_000)
                .unwrap(),
            Epoch::Unix.as_instant()
        );
        assert_eq!(
            Instant::from_julian_day_precise(2451544, 43200 + 64, 184_000_000_000_000_000)
                .unwrap(),
            Epoch::Y2k.as_instant()
        );

        assert_eq!(
            Instant::from_julian_day_precise(2443144, 43200 + 32, 184_000_000_000_000_000)
                .unwrap(),
            Epoch::TimeStandard.as_instant()
        );
    }

    #[test]
    fn test_time_standard_conversions() {
        crate::setup_logging();

        let p: Instant = From::from( DateTime::<Gregorian, Tai>::new(1993, 6, 30, 0, 0, 27, 0).unwrap());
        let q: DateTime<Gregorian, Utc> = From::from(p);
        assert_eq!(q,
                   DateTime::<Gregorian, Utc>::new(1993, 6, 30, 0, 0, 0, 0).unwrap());

        let p: Instant = Epoch::Unix.as_instant();
        let q: DateTime<Gregorian, Utc> = From::from(p);
        assert_eq!(q,
                   DateTime::<Gregorian, Utc>::new(1970, 1, 1, 0, 0, 0, 0).unwrap());

        let y2k: Instant = Epoch::Y2k.as_instant();
        let q: DateTime<Gregorian, Utc> = From::from(y2k);
        assert_eq!(q,
                   DateTime::<Gregorian, Utc>::new(2000, 1, 1, 0, 0, 0, 0).unwrap());
    }
}
