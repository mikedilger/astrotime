
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::duration::Duration;
use crate::instant::Instant;

const TCB_FACTOR: f64 = 1.550_505e-8;
const TCG_FACTOR: f64 = 6.969_290_134e-10;

/// A standard of time
pub trait Standard: Debug + Sized + Clone {
    /// Short capital-letter abbreviation for the time standard (usually 2 or 3 letters)
    fn abbrev() -> &'static str;

    /// Convert the given abnormal `Instant` `at` into an `Instant` using TT.
    fn to_tt(at: Instant) -> Instant;

    /// Convert the given `Instant` `at` into an abnormal `Instant` using `S`
    fn from_tt(at: Instant) -> Instant;
}

/// Whether a Standard is Continuous or not
pub trait Continuous { }

/// Terrestrial Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// See [Wikipedia](https://en.wikipedia.org/wiki/Terrestrial_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct Tt;
impl Standard for Tt {
    fn abbrev() -> &'static str {
        "TT"
    }

    fn to_tt(at: Instant) -> Instant {
        at
    }

    fn from_tt(at: Instant) -> Instant {
        at
    }
}
impl Continuous for Tt { }

/// Geocentric Coordinate Time
///
/// This is a continuous time standard for satellites that orbit the Earth
/// See [Wikipedia](https://en.wikipedia.org/wiki/Geocentric_Coordinate_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct Tcg;
impl Standard for Tcg {

    fn abbrev() -> &'static str {
        "TCG"
    }

    fn to_tt(at: Instant) -> Instant {
        Instant(at.0 * (1.0 - TCG_FACTOR))
    }

    fn from_tt(at: Instant) -> Instant {
        Instant(at.0 * (1.0 / (1.0 - TCG_FACTOR)))
    }
}
impl Continuous for Tcg { }

/// Barycentric Coordinate Time
///
/// This is a continuous time standard for satellites that orbit the Sun
/// See [Wikipedia](https://en.wikipedia.org/wiki/Barycentric_Coordinate_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct Tcb;
impl Standard for Tcb {

    fn abbrev() -> &'static str {
        "TCB"
    }

    fn to_tt(at: Instant) -> Instant {
        Instant(at.0 * (1.0 - TCB_FACTOR))
    }

    fn from_tt(at: Instant) -> Instant {
        Instant(at.0 * (1.0 / (1.0 - TCB_FACTOR)))
    }
}
impl Continuous for Tcb { }

/// International Atomic Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// realized via atomic clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct Tai;
impl Standard for Tai {
    fn abbrev() -> &'static str {
        "TAI"
    }

    fn to_tt(at: Instant) -> Instant {
        at + Duration::new(32, 184_000_000_000_000_000)
    }

    fn from_tt(at: Instant) -> Instant {
        at - Duration::new(32, 184_000_000_000_000_000)
    }
}
impl Continuous for Tai { }

/// Universal Coordinated Time
///
/// This is civil time as usually reported.  It is discontinuous, having leap
/// seconds inserted from time to time based on the Earth's rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct Utc;
impl Standard for Utc {
    fn abbrev() -> &'static str {
        "UTC"
    }

    fn to_tt(at: Instant) -> Instant {
        Tai::to_tt(at)
            + Duration::new(9, 0) // 9 leaps before 1972
            + Duration::new(leaps_at(at), 0) // leaps from 1972 - at
    }

    fn from_tt(at: Instant) -> Instant {
        Tai::from_tt(at)
            - Duration::new(9, 0) // 9 leaps before 1972
            - Duration::new(leaps_at(at), 0) // leaps from 1972 - at
    }
}

// This returns how many leap seconds have passed.
// (if the instant is inside of a leap second, that one does not get counted yet)
fn leaps_at(at: Instant) -> i64 {
    use crate::epoch::Epoch;

    let cmp = at + (Epoch::TimeStandard.as_instant() - Epoch::E1900_0.as_instant());
    let secs = cmp.0.seconds_part();

    trace!("Comparing seconds {} to leap second list", secs);

    // https://www.ietf.org/timezones/data/leap-seconds.list
    // FIXME: fetch the list dynamically if the user allows
    #[allow(clippy::unreadable_literal)]
    let leaps: Vec<i64> = vec![
        2272060800, //	10	# 1 Jan 1972
        2287785600, //	11	# 1 Jul 1972
        2303683200, //	12	# 1 Jan 1973
        2335219200, //	13	# 1 Jan 1974
        2366755200, //	14	# 1 Jan 1975
        2398291200, //	15	# 1 Jan 1976
        2429913600, //	16	# 1 Jan 1977
        2461449600, //	17	# 1 Jan 1978
        2492985600, //	18	# 1 Jan 1979
        2524521600, //	19	# 1 Jan 1980
        2571782400, //	20	# 1 Jul 1981
        2603318400, //	21	# 1 Jul 1982
        2634854400, //	22	# 1 Jul 1983
        2698012800, //	23	# 1 Jul 1985
        2776982400, //	24	# 1 Jan 1988
        2840140800, //	25	# 1 Jan 1990
        2871676800, //	26	# 1 Jan 1991
        2918937600, //	27	# 1 Jul 1992
        2950473600, //	28	# 1 Jul 1993
        2982009600, //	29	# 1 Jul 1994
        3029443200, //	30	# 1 Jan 1996
        3076704000, //	31	# 1 Jul 1997
        3124137600, //	32	# 1 Jan 1999
        3345062400, //	33	# 1 Jan 2006
        3439756800, //	34	# 1 Jan 2009
        3550089600, //	35	# 1 Jul 2012
        3644697600, //	36	# 1 Jul 2015
        3692217600, //	37	# 1 Jan 2017
    ];

    leaps.iter().enumerate()
        .find(|(_n,&leap)| secs <= leap)
        .map_or(leaps.len(), |(n,_d)| n) as i64
}

#[cfg(test)]
mod test {
    use super::leaps_at;
    use crate::date_time::DateTime;
    use crate::duration::Duration;
    use crate::calendar::Gregorian;
    use crate::instant::Instant;
    use crate::standard::{Standard, Tai, Tcg, Utc};

    #[test]
    fn test_to_from_tt() {
        crate::setup_logging();

        let i = Instant(Duration { secs: 21_309_887, attos: 214_892_349_872_398_743 });

        let j = Tai::to_tt(Tai::from_tt(i));
        assert_eq!(i, j);

        let j = Utc::to_tt(Utc::from_tt(i));
        assert_eq!(i, j);

        let j = Tcg::to_tt(Tcg::from_tt(i));
        assert_eq!(i.0.secs, j.0.secs);
        assert!(i.0.attos - j.0.attos < 10400000);
        assert!(i.0.attos - j.0.attos > -10400000);
    }

    #[test]
    fn test_leaps_at() {
        crate::setup_logging();

        // before any leaps
        let at: Instant = From::from(DateTime::<Gregorian, Utc>::new(1970, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leaps_at(at), 0);

        // between leap 3 and 4
        let at: Instant = From::from(DateTime::<Gregorian, Utc>::new(1973, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leaps_at(at), 3);

        // inside of leap second 4
        let at: Instant = From::from(DateTime::<Gregorian, Utc>::new(1973,12, 31, 0, 0, 60, 500_000_000_000_000_000).unwrap());
        assert_eq!(leaps_at(at), 3);

        // after leap second 4
        let at: Instant = From::from(DateTime::<Gregorian, Utc>::new(1974, 1, 1, 0, 0, 0, 500_000_000_000_000_000).unwrap());
        assert_eq!(leaps_at(at), 4);

        // after all leaps
        let at: Instant = From::from(DateTime::<Gregorian, Utc>::new(2019, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leaps_at(at), 28);
    }
}
