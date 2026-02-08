use crate::duration::Duration;
use crate::instant::Instant;

/// A reference for a well known `Instant` in time, used for offsetting events from.
pub enum Epoch {
    /// The start of the Julian Period,
    /// which is 4713 BCE on Jan 1st Julian, 12:00:00.0 TT
    /// which is 4714 BCE on Nov 24 Gregorian, 12:00:00.0 TT
    /// JD 0 (by definition)
    JulianPeriod,

    /// The start of the Julian Calendar period,
    /// which is January 1st, CE 1, Julian, 00:00:00.0 TT
    /// Specified in TT
    /// Note that this is exactly two days prior to the Gregorian Epoch.
    ///
    /// Not to be confused with the `Epoch::JulianPeriod` which is much earlier.
    /// JD 1721423.5 (unverified, based on 2 day offset from Gregorian)
    JulianCalendar,

    /// The start of the Gregorian Calendar period,
    /// which is January 1st, CE 1, Gregorian, 00:00:00.0 TT
    /// JD 1721425.5
    GregorianCalendar,

    /// The Spreadsheet Epoch
    /// Which is December 30, 1899 CE gregorian, 00:00:00.0 UTC
    /// This is the point from which dates in a spreadsheet are counted
    /// as the (floating point) number of days since.
    /// JD 2415018.500476666666  // 32.184 [TAI-TT] + 9 [UTC-TAI]
    Spreadsheet,

    /// The J1900.0 astronomical epoch,
    /// which is December 31, 1899 CE gregorian, 12:00:00.0 TT
    /// JD 2415020.0 (verified at <https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html>)
    J1900_0,

    /// The NTP time base used in the IANA leap seconds file
    /// which is January 1st, 1900 CE gregorian, 00:00:00.0 UTC
    /// (seconds elapsed since 1900-01-01 00:00:00 UTC, except the leap seconds themselves)
    /// JD 2415020.500476666666  // 32.184 [TAI-TT] + 9 [UTC-TAI]
    Ntp,

    /// The UNIX Epoch,
    /// which is January 1st, 1970 CE gregorian, 00:00:00.0 UTC
    /// JD 2440587.500476666666  // 32.184 [TAI-TT] + 9 [UTC-TAI]
    Unix,

    /// The epoch where TT, TCB, and TCG all read the same.
    /// which is January 1st, 1977 CE gregorian, 00:00:00 TAI
    /// JD 2443144.5003725 (verified at <https://en.wikipedia.org/wiki/International_Atomic_Time>)
    /// This is 16 seconds different from this date in UTC, and another 32.184 [TAI-TT] from TT.
    TimeStandard,

    /// A year I use for testing
    /// 1977-1-1 00:00:00 UTC
    Y1977,

    /// The J1991.25 astronomical epoch,
    /// which is April 2, 1991 CE gregorian, 13:30:00.0 TT
    /// JD 2448349.0625 (verified at <https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html>)
    J1991_25,

    /// The Year 2000
    /// which is January 1st, 2000 CE gregorian, 00:00:00.0 UTC
    /// JD 2451544.50074287037 // 32.184 [TAI-TT] + 32 [UTC-TAI]
    Y2k,

    /// The J2000.0 astronomical epoch,
    /// which is January 1, 2000 CE gregorian, 12:00:00.0 TT
    /// JD 2451545.0 (verified at <https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html>)
    /// (verified at <https://en.wikipedia.org/wiki/Epoch_(astronomy)>)
    J2000_0,

    /// The J2100.0 astronomical epoch,
    /// which is January 1, 2100 CE gregorian, 12:00:00.0 TT
    /// JD 2488070.0 (verified at <https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html>)
    J2100_0,

    /// The J2200.0 astronomical epoch,
    /// which is January 2, 2200 CE gregorian, 12:00:00.0 TT
    /// JD 2524595.0 (verified at <https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html>)
    J2200_0,
}

impl Epoch {
    /// Generate the `Instant` that this `Epoch` refers to
    #[must_use]
    pub const fn as_instant(&self) -> Instant {
        match *self {
            // NOTE: all instants are internally represented in TT standard.
            Self::JulianPeriod => Instant(Duration {
                secs: -211_087_684_832,
                attos: -184_000_000_000_000_000,
            }),
            Self::JulianCalendar => Instant(Duration {
                secs: -62_356_694_432,
                attos: -184_000_000_000_000_000,
            }),
            Self::GregorianCalendar => Instant(Duration {
                secs: -62_356_521_632,
                attos: -184_000_000_000_000_000,
            }),
            Self::Spreadsheet => Instant(Duration {
                secs: -2_430_086_391,
                attos: -0,
            }),
            Self::J1900_0 => Instant(Duration {
                secs: -2_429_956_832,
                attos: -184_000_000_000_000_000,
            }),
            Self::Ntp => Instant(Duration {
                secs: -2_429_913_591,
                attos: -0,
            }),
            Self::Unix => Instant(Duration {
                secs: -220_924_791,
                attos: -0,
            }),
            Self::TimeStandard => Instant(Duration { secs: 0, attos: 0 }),
            Self::Y1977 => Instant(Duration { secs: 16, attos: 0 }),
            Self::J1991_25 => Instant(Duration {
                secs: 449_674_167,
                attos: 816_000_000_000_000_000,
            }),
            Self::Y2k => Instant(Duration {
                secs: 725_760_032,
                attos: 0,
            }),
            Self::J2000_0 => Instant(Duration {
                secs: 725_803_167,
                attos: 816_000_000_000_000_000,
            }),
            Self::J2100_0 => Instant(Duration {
                secs: 3_881_563_167,
                attos: 816_000_000_000_000_000,
            }),
            Self::J2200_0 => Instant(Duration {
                secs: 7_037_323_167,
                attos: 816_000_000_000_000_000,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Epoch;
    use crate::calendar::{Gregorian, Julian};
    use crate::date_time::DateTime;
    use crate::instant::Instant;
    use crate::standard::{Tai, Tt, Utc};

    macro_rules! epoch_check {
        ($epoch:expr, $cal:ty, $std:ty, $def:expr) => {
            let instant = $epoch.as_instant();
            let dt: DateTime<$cal, $std> = From::from(instant);
            assert_eq!(dt, $def);
            let check: Instant = From::from(dt);
            assert_eq!(instant, check);
        };
    }

    #[test]
    fn check_epochs_and_conversion() {
        epoch_check!(
            Epoch::JulianPeriod,
            Julian,
            Tt,
            DateTime::<Julian, Tt>::new_bc(4713, 1, 1, 12, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::JulianCalendar,
            Julian,
            Tt,
            DateTime::<Julian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::GregorianCalendar,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::Spreadsheet,
            Gregorian,
            Utc,
            DateTime::<Gregorian, Utc>::new(1899, 12, 30, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::J1900_0,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(1899, 12, 31, 12, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::Ntp,
            Gregorian,
            Utc,
            DateTime::<Gregorian, Utc>::new(1900, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::Unix,
            Gregorian,
            Utc,
            DateTime::<Gregorian, Utc>::new(1970, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::TimeStandard,
            Gregorian,
            Tai,
            DateTime::<Gregorian, Tai>::new(1977, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::Y1977,
            Gregorian,
            Utc,
            DateTime::<Gregorian, Utc>::new(1977, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::J1991_25,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(1991, 4, 2, 13, 30, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::Y2k,
            Gregorian,
            Utc,
            DateTime::<Gregorian, Utc>::new(2000, 1, 1, 0, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::J2000_0,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(2000, 1, 1, 12, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::J2100_0,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(2100, 1, 1, 12, 0, 0, 0).unwrap()
        );
        epoch_check!(
            Epoch::J2200_0,
            Gregorian,
            Tt,
            DateTime::<Gregorian, Tt>::new(2200, 1, 2, 12, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_instant_julian_day_formatted() {
        assert_eq!(
            Epoch::JulianPeriod.as_instant().as_julian_day_formatted(),
            "JD 0"
        );
        assert_eq!(
            Epoch::JulianCalendar.as_instant().as_julian_day_formatted(),
            "JD 1721423.5"
        );
        assert_eq!(
            Epoch::GregorianCalendar
                .as_instant()
                .as_julian_day_formatted(),
            "JD 1721425.5"
        );
        assert_eq!(
            Epoch::Spreadsheet.as_instant().as_julian_day_formatted(),
            "JD 2415018.5004766666666667"
        );
        assert_eq!(
            Epoch::J1900_0.as_instant().as_julian_day_formatted(),
            "JD 2415020"
        );
        assert_eq!(
            Epoch::Ntp.as_instant().as_julian_day_formatted(),
            "JD 2415020.5004766666666667"
        );
        assert_eq!(
            Epoch::Unix.as_instant().as_julian_day_formatted(),
            "JD 2440587.5004766666666667"
        );
        assert_eq!(
            Epoch::TimeStandard.as_instant().as_julian_day_formatted(),
            "JD 2443144.5003725"
        );
        assert_eq!(
            Epoch::J1991_25.as_instant().as_julian_day_formatted(),
            "JD 2448349.0625"
        );
        assert_eq!(
            Epoch::Y2k.as_instant().as_julian_day_formatted(),
            "JD 2451544.5007428703703704"
        );
        assert_eq!(
            Epoch::J2000_0.as_instant().as_julian_day_formatted(),
            "JD 2451545"
        );
        assert_eq!(
            Epoch::J2100_0.as_instant().as_julian_day_formatted(),
            "JD 2488070"
        );
        assert_eq!(
            Epoch::J2200_0.as_instant().as_julian_day_formatted(),
            "JD 2524595"
        );
    }
}
