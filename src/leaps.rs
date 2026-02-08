use crate::{Duration, Epoch, Instant};

/*
 *  LEAP SECONDS occur like this
 *
 *  59.0      59.5      60.0      60.5      0.0        0.5        1.0       <--UTC :seconds
 *  |         |         |         |         |          |          |
 *  599.0     599.5     599.0     599.5     600.0      600.5      601.0     <--NTC time
 *
 *                      |<----------------->|
 *                         The Leap Second
 *                                         /|\
 *                                          |
 *                                  IANA Leap Instant
 *
 *  NTP times [599.0, 600.0) are replayed.  But UTC times are distinct with the :60 indicator.
 *
 *  The NTP times listed in the IANA list refer to the instant AFTER the leap second, which
 *  itself is not ill-defined.
 */

// https://data.iana.org/time-zones/data/leap-seconds.list
// Expires 28 June 2026
#[allow(clippy::unreadable_literal)]
const IANA_NTP_LEAP_SECONDS: &[i64] = &[
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

/// This returns the `Instant`s directly after leap seconds.
///
/// It is mainly for internal use as this crate handles leap seconds automatically,
/// but it is exposed nonetheless.
pub fn leap_instants() -> impl Iterator<Item = Instant> {
    LeapInstantIter { i: 0 }
}

pub struct LeapInstantIter {
    i: usize,
}

impl Iterator for LeapInstantIter {
    type Item = Instant;

    #[allow(clippy::cast_possible_wrap)] // usizes here are going to be small
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= IANA_NTP_LEAP_SECONDS.len() {
            None
        } else {
            // Every entry in the IANA table is in NTP time, which
            // does not count leaps. To get true durations/instants
            // we need to add the leaps back in.
            // The first entry is actually 1 second further in the future,
            // The second entry 2 seconds, etc.
            let leaps_elapsed = self.i + 1;

            let offset = Duration::new(IANA_NTP_LEAP_SECONDS[self.i] + leaps_elapsed as i64, 0);

            // Bump the internal counter for iteration
            self.i += 1;

            Some(Epoch::Ntp.as_instant() + offset)
        }
    }
}

/// This counts how many leap seconds have elapsed at a given instant.
///
/// This only includes IANA listed leap seconds. That is to say, we
/// are not counting the 9 seconds separating UTC and TAI before 1972.
///
/// If the instant is during a leap second, that one doesn't count.
/// If it is immediately after, which is at the IANA instant itself,
/// then it does.
#[must_use]
pub fn leap_seconds_elapsed_at(i: Instant) -> i64 {
    let mut count: i64 = 0;
    for leap_instant in leap_instants() {
        if i >= leap_instant {
            count += 1;
        } else {
            break;
        }
    }
    count
}

#[cfg(test)]
mod test {
    use super::{leap_instants, leap_seconds_elapsed_at};
    use crate::{DateTime, Epoch, Gregorian, Instant, Utc};

    #[test]
    fn test_leap_instants() {
        let fourth = leap_instants().skip(3).next().unwrap();
        let dur = fourth - Epoch::Ntp.as_instant();
        assert_eq!(dur.secs, 2335219200 + 4);
    }

    #[test]
    fn test_leap_seconds_elapsed_at() {
        // before any leaps
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(1970, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 0);

        // between leap 3 and 4
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(1973, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 3);

        // inside of leap second 4
        let at: Instant = From::from(
            DateTime::<Gregorian, Utc>::new(1973, 12, 31, 0, 0, 60, 500_000_000_000_000_000)
                .unwrap(),
        );
        assert_eq!(leap_seconds_elapsed_at(at), 3);

        // after leap second 4
        let at: Instant = From::from(
            DateTime::<Gregorian, Utc>::new(1974, 1, 1, 0, 0, 0, 500_000_000_000_000_000).unwrap(),
        );
        assert_eq!(leap_seconds_elapsed_at(at), 4);

        // after all leaps
        let at: Instant =
            From::from(DateTime::<Gregorian, Utc>::new(2019, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 28);
    }
}
