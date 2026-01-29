use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::duration::Duration;
use crate::instant::Instant;

/// A standard of time
pub trait Standard: Debug + Sized + Clone {
    /// Short capital-letter abbreviation for the time standard (usually 2 or 3 letters)
    fn abbrev() -> &'static str;

    /// This function is not meant to be called from outside the library.
    ///
    /// But if you wish to create a new Standard that implements this trait, you'll need
    /// to know how to do so.
    ///
    /// It takes a `Duration` from January 1st, 1977 CE gregorian, 00:00:32.184
    /// as defined by this `Standard` and converts it to a Duration from January 1st, 1977 CE
    /// gregorian, 00:00:32.184 TT.
    fn to_tt(dur: Duration) -> Duration;

    /// This function is not meant to be called from outside the library.
    ///
    /// But if you wish to create a new Standard that implements this trait, you'll need
    /// to know how to do so.
    ///
    /// It takes a `Duration` from January 1st, 1977 CE gregorian, 00:00:32.184 TT
    /// and converts it to a `Duration` from January 1st, 1977 CE gregorian, 00:00:32.184
    /// as defined by this `Standard`.
    fn from_tt(dur: Duration) -> Duration;
}

/// Whether a Standard is Continuous or not
pub trait Continuous {}

/// Terrestrial Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// See [Wikipedia](https://en.wikipedia.org/wiki/Terrestrial_Time)
///
/// This type is proleptic. TT was defined in 1976, and changed in 2000 very slightly.
/// All dates before this extrapolate backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tt;
impl Standard for Tt {
    fn abbrev() -> &'static str {
        "TT"
    }

    fn to_tt(dur: Duration) -> Duration {
        dur
    }

    fn from_tt(dur: Duration) -> Duration {
        dur
    }
}
impl Continuous for Tt {}

/// International Atomic Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// realized via atomic clocks.
///
/// This type is proleptic. TAI started on 1 January 1958, but we represent all dates
/// before this as if TAI extends backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tai;
impl Standard for Tai {
    fn abbrev() -> &'static str {
        "TAI"
    }

    fn to_tt(dur: Duration) -> Duration {
        dur + Duration::new(32, 184_000_000_000_000_000)
    }

    fn from_tt(dur: Duration) -> Duration {
        dur - Duration::new(32, 184_000_000_000_000_000)
    }
}
impl Continuous for Tai {}

/// Universal Coordinated Time
///
/// This is civil time as usually reported.  It is discontinuous, having leap
/// seconds inserted from time to time based on the Earth's rotation.
///
/// This type is proleptic. For all dates prior to 1 Jan 1972, we presume
/// 9 leap seconds have elapsed, offsetting from TAI permanently by 9 seconds,
/// even though that's not what happened at the time (what happened at the time
/// was that UTC wasn't syncronized to TAI by integer leap seconds, but rather
/// to other time sources and contained fractional leap seconds which are hard
/// to reconstruct or even get a list of).  For all dates prior to 1 January
/// 1960, UTC didn't exist, but we pretend it did, offset 9 seconds from TAI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Utc;
impl Standard for Utc {
    fn abbrev() -> &'static str {
        "UTC"
    }

    fn to_tt(dur: Duration) -> Duration {
        Tai::to_tt(dur)
            + Duration::new(9, 0) // 9 leaps before 1972
            + Duration::new(leap_seconds_elapsed_for_utc(dur), 0) // leaps on or after 1972
    }

    fn from_tt(dur: Duration) -> Duration {
        Tai::from_tt(dur)
            - Duration::new(9, 0) // 9 leaps before 1972
            - Duration::new(leap_seconds_elapsed(Instant(dur)), 0) // leaps on or after 1972
    }
}
// https://data.iana.org/time-zones/data/leap-seconds.list
// Expires 28 June 2026
#[allow(clippy::unreadable_literal)]
fn iana_ntp_leap_seconds() -> Vec<i64> {
    vec![
        // Unixtime
        2272060800, //	10	# 1 Jan 1972      // 63072000
        2287785600, //	11	# 1 Jul 1972      // 78796800
        2303683200, //	12	# 1 Jan 1973      // 94694400
        2335219200, //	13	# 1 Jan 1974      // 126230400
        2366755200, //	14	# 1 Jan 1975      // 157766400
        2398291200, //	15	# 1 Jan 1976      // 189302400
        2429913600, //	16	# 1 Jan 1977      // 220924800
        2461449600, //	17	# 1 Jan 1978      // 252460800
        2492985600, //	18	# 1 Jan 1979      // 283996800
        2524521600, //	19	# 1 Jan 1980      // 315532800
        2571782400, //	20	# 1 Jul 1981      // 362793600
        2603318400, //	21	# 1 Jul 1982      // 394329600
        2634854400, //	22	# 1 Jul 1983      // 425865600
        2698012800, //	23	# 1 Jul 1985      // 489024000
        2776982400, //	24	# 1 Jan 1988      // 567993600
        2840140800, //	25	# 1 Jan 1990      // 631152000
        2871676800, //	26	# 1 Jan 1991      // 662688000
        2918937600, //	27	# 1 Jul 1992      // 709948800
        2950473600, //	28	# 1 Jul 1993      // 741484800
        2982009600, //	29	# 1 Jul 1994      // 773020800
        3029443200, //	30	# 1 Jan 1996      // 820454400
        3076704000, //	31	# 1 Jul 1997      // 867715200
        3124137600, //	32	# 1 Jan 1999      // 915148800
        3345062400, //	33	# 1 Jan 2006      // 1136073600
        3439756800, //	34	# 1 Jan 2009      // 1230768000
        3550089600, //	35	# 1 Jul 2012      // 1341100800
        3644697600, //	36	# 1 Jul 2015      // 1435708800
        3692217600, //	37	# 1 Jan 2017      // 1483228800
    ]
}

// This returns how many leap seconds have passed.
// (if the instant is inside of a leap second, that one does not get counted yet)
#[allow(clippy::cast_possible_wrap)] // fixed input data won't wrap
pub fn leap_seconds_elapsed(at: Instant) -> i64 {
    use crate::epoch::Epoch;

    // NOTE: if our instants ever differ from TimeStandard, we need to run this
    // instead:
    // let cmp = at + (Epoch::TimeStandard.as_instant() - Epoch::Ntp.as_instant());
    let cmp = at - Epoch::Ntp.as_instant();
    let secs = cmp.seconds_part();

    trace!("Comparing seconds {} to leap second list", secs);

    iana_ntp_leap_seconds()
        .iter()
        .enumerate()
        .find(|(_n, &leap)| secs < leap)
        .map_or_else(|| iana_ntp_leap_seconds().len(), |(n, _d)| n) as i64
}

// Similar to leap_seconds_elapsed(), but using an incorrect/unadjusted duration
// computed using UTC as if there were no leap seconds. This function is for
// converting from UTC to TAI.
#[allow(clippy::cast_possible_wrap)] // fixed input data won't wrap
fn leap_seconds_elapsed_for_utc(mut unadjusted_dur: Duration) -> i64 {
    use crate::epoch::Epoch;

    // Adjust the UTC based duration as close to TT as we can (all but leaps)
    unadjusted_dur = unadjusted_dur + Duration::new(9 + 32, 184_000_000_000_000_000);
    let cmp = unadjusted_dur - Epoch::Ntp.as_instant().0;
    let secs = cmp.seconds_part();

    trace!("Comparing seconds {} to leap second list (from UTC)", secs);

    iana_ntp_leap_seconds()
        .iter()
        .enumerate()
        .map(|(n, leap)| (n, leap - n as i64)) // each leap successively drug backwards
        .find(|(_n, leap)| secs < *leap)
        .map_or_else(|| iana_ntp_leap_seconds().len(), |(n, _d)| n) as i64
}

#[cfg(test)]
mod test {
    use super::leap_seconds_elapsed;
    use crate::calendar::Gregorian;
    use crate::date_time::DateTime;
    use crate::duration::Duration;
    use crate::instant::Instant;
    use crate::standard::{Standard, Tai, Tt, Utc};

    #[test]
    fn test_to_from_tt() {
        crate::setup_logging();

        let i = Duration {
            secs: 21_309_887,
            attos: 214_892_349_872_398_743,
        };

        let j = Tai::to_tt(Tai::from_tt(i));
        assert_eq!(i, j);

        let j = Utc::to_tt(Utc::from_tt(i));
        assert_eq!(i, j);

        // Test UTC in the vacinity of a leap second (1 January 1999)
        let leap_instant: Instant = From::from(
            DateTime::<Gregorian, Tt>::new(1999, 1, 1, 0, 0, 0, 0).unwrap()
                - Duration::new(32 + 32, 184_000_000_000_000_000),
        );
        for s in -100..100 {
            // leap happens at s=65 or 66
            // NOTE: we cannot possibly map in a lossy way to UTC and back again
            //       without an error somewhere. 3124137577 repeats.  Which TT
            //       second should it refer to?
            //       So we skip that one nasty value of s
            if s == 65 {
                continue;
            }

            // FIXME- the fact is that DateTime *SHOULD* have a :60 second
            // so that we can differentiate them. But our from_tt()/to_tt()
            // has lost such information. Perhaps we need to do conversions
            // between DateTime objects instead of between Instants.

            trace!("s={}", s);
            let a = leap_instant + Duration::new(s, 0);
            let b = Instant(Utc::to_tt(Utc::from_tt(a.0)));
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_leap_seconds_elapsed() {
        crate::setup_logging();

        // before any leaps
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(1970, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed(at), 0);

        // between leap 3 and 4
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(1973, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed(at), 3);

        // inside of leap second 4
        let at: Instant = From::from(
            DateTime::<Gregorian, Utc>::new(1973, 12, 31, 0, 0, 60, 500_000_000_000_000_000)
                .unwrap(),
        );
        assert_eq!(leap_seconds_elapsed(at), 3);

        // after leap second 4
        let at: Instant = From::from(
            DateTime::<Gregorian, Utc>::new(1974, 1, 1, 0, 0, 0, 500_000_000_000_000_000).unwrap(),
        );
        assert_eq!(leap_seconds_elapsed(at), 4);

        // after all leaps
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(2019, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed(at), 28);
    }
}
