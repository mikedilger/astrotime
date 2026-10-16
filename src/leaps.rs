use crate::{ATTOS_PER_SEC_I128, DatedInstant, Duration, Epoch};

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
// Expires 28 June 2027
const IANA_NTP_EXPIRES: i64 = 4_023_129_600;

#[allow(clippy::unreadable_literal)]
const IANA_NTP_LEAP_SECONDS: &[i64] = &[
    2272060800, //	10	# 1 Jan 1972 (not a leap second, TAI-UTC starts at 10)
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

// TAI-UTC from 1961 until 1972, from
// https://hpiers.obspm.fr/eop-pc/earthor/utc/TAI-UTC_tab.html
// TAI-UTC = offset + (MJD - reference MJD) * drift, in UTC.
// (start MJD, offset in 1e-7 s, reference MJD, drift in 1e-9 s per s)
const EARLY_UTC: &[(i64, i64, i64, i64)] = &[
    (i64::MIN / 86400 / 2, 14_228_180, 37300, 0), // before 1961, as in 1961
    (37300, 14_228_180, 37300, 15),               // 1961 Jan 1
    (37512, 13_728_180, 37300, 15),               // 1961 Aug 1
    (37665, 18_458_580, 37665, 13),               // 1962 Jan 1
    (38334, 19_458_580, 37665, 13),               // 1963 Nov 1
    (38395, 32_401_300, 38761, 15),               // 1964 Jan 1
    (38486, 33_401_300, 38761, 15),               // 1964 Apr 1
    (38639, 34_401_300, 38761, 15),               // 1964 Sep 1
    (38761, 35_401_300, 38761, 15),               // 1965 Jan 1
    (38820, 36_401_300, 38761, 15),               // 1965 Mar 1
    (38942, 37_401_300, 38761, 15),               // 1965 Jul 1
    (39004, 38_401_300, 38761, 15),               // 1965 Sep 1
    (39126, 43_131_700, 39126, 30),               // 1966 Jan 1
    (39887, 42_131_700, 39126, 30),               // 1968 Feb 1
];

const MJD_1972: i64 = 41317;

// Gregorian day number of MJD 0 (1858-11-17)
const MJD_DAY_NUMBER: i64 = 678_575;

const TT_MINUS_TAI: i128 = 32_184_000_000_000_000_000;

fn mjd_attos(mjd: i64) -> i128 {
    i128::from(mjd + MJD_DAY_NUMBER) * 86400 * ATTOS_PER_SEC_I128
}

fn to_attos(d: Duration) -> i128 {
    i128::from(d.secs) * ATTOS_PER_SEC_I128 + i128::from(d.attos)
}

#[allow(clippy::cast_possible_truncation)]
const fn from_attos(a: i128) -> Duration {
    Duration::new(
        a.div_euclid(ATTOS_PER_SEC_I128) as i64,
        a.rem_euclid(ATTOS_PER_SEC_I128) as i64,
    )
}

// Segment `i` of the early table: its label start, and TT minus label at
// label `l` within it, in attoseconds.
fn early_segment(i: usize) -> (i128, impl Fn(i128) -> i128) {
    let (start, offset, reference, drift) = EARLY_UTC[i];
    let reference = mjd_attos(reference);
    let offset = i128::from(offset) * 100_000_000_000 + TT_MINUS_TAI;
    let tt_minus_label =
        move |l: i128| offset + ((l - reference) * i128::from(drift)).div_euclid(1_000_000_000);
    (mjd_attos(start), tt_minus_label)
}

fn early_index(label: i128) -> usize {
    EARLY_UTC
        .iter()
        .rposition(|e| mjd_attos(e.0) <= label)
        .unwrap_or(0)
}

// 1972-01-01 00:00:00 UTC
fn utc_1972() -> DatedInstant {
    Epoch::GregorianCalendar.as_dated_instant()
        + from_attos(mjd_attos(MJD_1972) + 10 * ATTOS_PER_SEC_I128 + TT_MINUS_TAI)
}

/// How much time was added (or, if negative, skipped) at the end of the UTC
/// day ending at `day_end` (a label, seconds since 0001-01-01 Gregorian), if
/// that is before 1972.
pub fn early_added_at(day_end: Duration) -> Option<Duration> {
    let day_end = to_attos(day_end);
    if day_end > mjd_attos(MJD_1972) {
        return None;
    }
    let (_, old) = early_segment(early_index(day_end - 1));
    let new = if day_end == mjd_attos(MJD_1972) {
        10 * ATTOS_PER_SEC_I128 + TT_MINUS_TAI
    } else {
        early_segment(early_index(day_end)).1(day_end)
    };
    Some(from_attos(new - old(day_end)))
}

/// TT minus UTC at a UTC label (seconds since 0001-01-01 Gregorian) before
/// 1972. `second_60` is for labels in the time added at the end of a day.
pub fn early_utc_offset(label: Duration, second_60: bool) -> Option<Duration> {
    let label = to_attos(label);
    if label >= mjd_attos(MJD_1972) + i128::from(second_60) * ATTOS_PER_SEC_I128 {
        return None;
    }
    if second_60 {
        // The added time runs at SI rate from the end of the old day
        let day_end = label - label.rem_euclid(86400 * ATTOS_PER_SEC_I128);
        let (_, tt_minus_label) = early_segment(early_index(day_end - 1));
        return Some(from_attos(tt_minus_label(day_end)));
    }
    let (_, tt_minus_label) = early_segment(early_index(label));
    Some(from_attos(tt_minus_label(label)))
}

/// The UTC label (seconds since 0001-01-01 Gregorian) of a TT time (since
/// the same date in TT) before 1972, and whether it falls in time added at
/// the end of a day (shown as second 60).
pub fn early_utc_label(tt: Duration) -> Option<(Duration, bool)> {
    let tt = to_attos(tt);
    let end = mjd_attos(MJD_1972);
    if tt >= end + 10 * ATTOS_PER_SEC_I128 + TT_MINUS_TAI {
        return None;
    }
    let mut i = EARLY_UTC.len() - 1;
    loop {
        let (start, tt_minus_label) = early_segment(i);
        if i == 0 || start + tt_minus_label(start) <= tt {
            let next = EARLY_UTC.get(i + 1).map_or(end, |e| mjd_attos(e.0));
            // tt = l + tt_minus_label(l), increasing in l: estimate, then settle
            let mut l = tt - tt_minus_label(tt - tt_minus_label(tt));
            while l + 1 + tt_minus_label(l + 1) <= tt {
                l += 1;
            }
            while l + tt_minus_label(l) > tt {
                l -= 1;
            }
            if l >= next {
                let added = tt - next - tt_minus_label(next);
                return Some((from_attos(next - ATTOS_PER_SEC_I128 + added), true));
            }
            return Some((from_attos(l), false));
        }
        i -= 1;
    }
}

/// This returns the `DatedInstant`s directly after leap seconds.
///
/// It is mainly for internal use as this crate handles leap seconds automatically,
/// but it is exposed nonetheless.
pub fn leap_instants() -> impl Iterator<Item = DatedInstant> {
    LeapInstantIter { i: 1 }
}

pub struct LeapInstantIter {
    i: usize,
}

impl Iterator for LeapInstantIter {
    type Item = DatedInstant;

    #[allow(clippy::cast_possible_wrap)] // usizes here are going to be small
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= IANA_NTP_LEAP_SECONDS.len() {
            None
        } else {
            // Every entry in the IANA table is in NTP time, which
            // does not count leaps. To get true durations/instants
            // we need to add the leaps back in.
            // The first leap (the second entry) is 1 second further in the
            // future, the next 2 seconds, etc.
            let leaps_elapsed = self.i;

            let offset = Duration::new(
                IANA_NTP_LEAP_SECONDS[self.i] - IANA_NTP_LEAP_SECONDS[0] + leaps_elapsed as i64,
                0,
            );

            // Bump the internal counter for iteration
            self.i += 1;

            Some(utc_1972() + offset)
        }
    }
}

/// When the leap second list built into this crate expires. Leap seconds
/// are announced about six months ahead; after this we assume there are no
/// more, which may be wrong.
#[must_use]
pub fn leap_seconds_expire() -> DatedInstant {
    DatedInstant::from_ntp_date(IANA_NTP_EXPIRES, 0)
}

/// This counts how many leap seconds have elapsed at a given instant.
///
/// This only includes IANA listed leap seconds. That is to say, we
/// are not counting the 10 seconds separating UTC and TAI in 1972.
/// After [`leap_seconds_expire`] no further leap seconds are assumed.
///
/// If the instant is during a leap second, that one doesn't count.
/// If it is immediately after, which is at the IANA instant itself,
/// then it does.
#[must_use]
pub fn leap_seconds_elapsed_at(i: DatedInstant) -> i64 {
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
    use crate::{Calendar, DateTime, DatedInstant, Duration, Gregorian, Tai, Utc};

    #[test]
    fn test_leap_instants() {
        let fourth = leap_instants().nth(3).unwrap();
        let after: DatedInstant =
            From::from(DateTime::<Gregorian, Utc>::new(1975, 1, 1, 0, 0, 0, 0).unwrap());
        assert_eq!(fourth, after);
    }

    #[test]
    fn test_utc_before_1972() {
        let utc = |y, mo, d, h, mi, s, a| -> DatedInstant {
            DateTime::<Gregorian, Utc>::new(y, mo, d, h, mi, s, a)
                .unwrap()
                .into()
        };
        let tai = |y, mo, d| -> DatedInstant {
            DateTime::<Gregorian, Tai>::new(y, mo, d, 0, 0, 0, 0)
                .unwrap()
                .into()
        };
        // TAI-UTC was 1.4228180 s on 1961 Jan 1, and 8.000082 s on 1970 Jan 1
        assert_eq!(
            utc(1961, 1, 1, 0, 0, 0, 0) - tai(1961, 1, 1),
            Duration::new(1, 422_818_000_000_000_000)
        );
        assert_eq!(
            utc(1970, 1, 1, 0, 0, 0, 0) - tai(1970, 1, 1),
            Duration::new(8, 82_000_000_000_000)
        );
        // The last day of 1971 drifted 0.002592 s and ended 0.107758 s late
        let day = utc(1972, 1, 1, 0, 0, 0, 0) - utc(1971, 12, 31, 0, 0, 0, 0);
        assert_eq!(day, Duration::new(86400, 110_350_000_000_000_000));

        // That added time is second 60, round tripping exactly
        let mut dt =
            DateTime::<Gregorian, Utc>::new(1971, 12, 31, 23, 59, 59, 100_000_000_000_000_000)
                .unwrap();
        assert!(dt.set_second(60).is_ok());
        let i: DatedInstant = dt.into();
        assert_eq!(DateTime::<Gregorian, Utc>::from(i), dt);
        let mut dt =
            DateTime::<Gregorian, Utc>::new(1971, 12, 31, 23, 59, 59, 200_000_000_000_000_000)
                .unwrap();
        assert!(dt.set_second(60).is_err());

        // No time was added on these days (1968 Feb 1 skipped 0.1 s)
        for (y, m, d) in [(1971, 6, 30), (1968, 1, 31), (1950, 12, 31)] {
            let mut dt = DateTime::<Gregorian, Utc>::new(y, m, d, 23, 59, 59, 0).unwrap();
            assert!(dt.set_second(60).is_err(), "{y}-{m}-{d}");
        }

        // Every label round trips, through the steps
        let mut i = utc(1960, 12, 31, 23, 0, 0, 0);
        while i < utc(1972, 1, 2, 0, 0, 0, 0) {
            let dt = DateTime::<Gregorian, Utc>::from(i);
            assert_eq!(DatedInstant::from(dt), i, "{dt}");
            i += Duration::new(1_234_567, 890_123_456_789);
        }
    }

    #[test]
    fn test_early_utc_table_dates() {
        for (mjd, (y, m)) in super::EARLY_UTC.iter().skip(1).map(|e| e.0).zip([
            (1961, 1),
            (1961, 8),
            (1962, 1),
            (1963, 11),
            (1964, 1),
            (1964, 4),
            (1964, 9),
            (1965, 1),
            (1965, 3),
            (1965, 7),
            (1965, 9),
            (1966, 1),
            (1968, 2),
        ]) {
            assert_eq!(
                Gregorian::day_number(y, m, 1).unwrap(),
                mjd + super::MJD_DAY_NUMBER
            );
        }
        assert_eq!(
            Gregorian::day_number(1972, 1, 1).unwrap(),
            super::MJD_1972 + super::MJD_DAY_NUMBER
        );
    }

    #[test]
    fn test_leap_seconds_expire() {
        let expires: DatedInstant =
            From::from(DateTime::<Gregorian, Utc>::new(2027, 6, 28, 0, 0, 0, 0).unwrap());
        assert_eq!(super::leap_seconds_expire(), expires);
        assert_eq!(leap_instants().count(), 27);
    }

    #[test]
    fn test_leap_seconds_elapsed_at() {
        // before any leaps
        let at: DatedInstant =
            From::from(DateTime::<Gregorian, Utc>::new(1970, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 0);

        // between leap 2 and 3
        let at: DatedInstant =
            From::from(DateTime::<Gregorian, Utc>::new(1973, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 2);

        // inside of leap second 3
        let at: DatedInstant = From::from(
            DateTime::<Gregorian, Utc>::new(1973, 12, 31, 23, 59, 60, 500_000_000_000_000_000)
                .unwrap(),
        );
        assert_eq!(leap_seconds_elapsed_at(at), 2);

        // after leap second 3
        let at: DatedInstant = From::from(
            DateTime::<Gregorian, Utc>::new(1974, 1, 1, 0, 0, 0, 500_000_000_000_000_000).unwrap(),
        );
        assert_eq!(leap_seconds_elapsed_at(at), 3);

        // after all leaps
        let at: DatedInstant =
            From::from(DateTime::<Gregorian, Utc>::new(2019, 9, 17, 13, 45, 18, 0).unwrap());
        assert_eq!(leap_seconds_elapsed_at(at), 27);
    }
}
