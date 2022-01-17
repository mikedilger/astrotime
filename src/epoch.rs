
use crate::instant::Instant;
use crate::duration::Duration;

/// A reference for a well known `Instant` in time, used for offsetting events from.
pub enum Epoch {
    /// The start of the Julian Period,
    /// which is 4713 BCE on Jan 1st Julian, 00:00:00.0
    /// Specified in TT (in accordance with the International Astronomical Union since 1997)
    // JD 0 (by definition)
    JulianPeriod,

    /// The start of the Julian Calendar period,
    /// which is January 1st, CE 1, 00:00:00.0 in the Julian calendar.
    /// Specified in TT
    /// Note that this is exactly two days prior to the Gregorian Epoch.
    ///
    /// Not to be confused with the `JulianPeriodEpoch` which is much earlier.
    // JD 1721423.5 (unverified, based on 2 day offset from Gregorian)
    JulianCalendar,

    /// The start of the Gregorian Calendar period,
    /// which is January 1st, CE 1, 00:00:00.0 in the Gregorian calendar.
    /// Specified in TT
    // JD 1721425.5 (verified from https://en.wikipedia.org/wiki/Julian_day)
    GregorianCalendar,

    /// The J1900.0 astronomical epoch,
    /// which is December 31, 1899 CE gregorian, 12:00:00.0
    /// Specified in TT
    // JD 2415020.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
    J1900_0,

    /// The 1900.0 astronomical epoch
    /// which is January 1st, 1900 CE gregorian, 00:00:00.0
    /// Specified in TT
    // JD 2415020.5
    E1900_0,

    /// The UNIX Epoch,
    /// which is January 1st, 1970 CE gregorian, 00:00:00.0
    /// Specified in UTC
    // JD 2440587.5 (approx, modify for UTC)
    Unix,

    /// The Time Standard Epoch where TT, TCB, and TCG all read the same.
    /// which is January 1st, 1977 CE gregorian, 00:00:32.184
    /// Specified in TT, TCB and TCG, where they align
    // JD 2443144.5003725 (https://en.wikipedia.org/wiki/International_Atomic_Time)
    TimeStandard,

    /// The J1991.25 astronomical epoch,
    /// which is April 2, 1991 CE gregorian, 13:30:00.0
    /// Specified in TT
    // JD 2448349.0625 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
    J1991_25,

    /// The Year 2000
    /// which is January 1st, 2000 CE gregorian, 00:00:00.0
    /// Specified in UTC
    // JD 2451544.5 (approx, modify for UTC)
    Y2k,

    /// The J2000.0 astronomical epoch,
    /// which is January 1, 2000 CE gregorian, 12:00:00.0
    /// Specified in TT
    // JD 2451545.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
    J2000_0,

    /// The J2100.0 astronomical epoch,
    /// which is January 1, 2100 CE gregorian, 12:00:00.0
    /// Specified in TT
    // JD 2488070.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
    J2100_0,

    /// The J2200.0 astronomical epoch,
    /// which is January 2, 2200 CE gregorian, 12:00:00.0
    /// Specified in TT
    // JD 2524595.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
    J2200_0
}

impl Epoch {
    /// Generate the `Instant` that this `Epoch` refers to
    #[must_use]
    pub const fn as_instant(&self) -> Instant {
        match *self {
            // NOTE: all instants are internally represented in TT standard.
            Epoch::JulianPeriod =>
                Instant(Duration { secs: -211_087_684_832, attos: -184_000_000_000_000_000 }),
            Epoch::JulianCalendar =>
                Instant(Duration { secs: -62_356_694_432, attos: -184_000_000_000_000_000 }),
            Epoch::GregorianCalendar =>
                Instant(Duration { secs: -62_356_521_632, attos: -184_000_000_000_000_000 }),
            Epoch::J1900_0 =>
                Instant(Duration { secs: -2_429_956_832, attos: -184_000_000_000_000_000 }),
            Epoch::E1900_0 =>
                Instant(Duration { secs: -2_429_913_632, attos: -184_000_000_000_000_000 }),
            Epoch::Unix =>
                Instant(Duration { secs: -220_924_791, attos: 0 }),
            Epoch::TimeStandard =>
                Instant(Duration { secs: 0, attos: 0 }),
            Epoch::J1991_25 =>
                Instant(Duration { secs: 449_674_167, attos: 816_000_000_000_000_000 }),
            Epoch::Y2k =>
                Instant(Duration { secs: 725_760_032, attos: 0 }),
            Epoch::J2000_0 =>
                Instant(Duration { secs: 725_803_167, attos: 816_000_000_000_000_000 }),
            Epoch::J2100_0 =>
                Instant(Duration { secs: 3_881_563_167, attos: 816_000_000_000_000_000 }),
            Epoch::J2200_0 =>
                Instant(Duration { secs: 7_037_323_167, attos: 816_000_000_000_000_000 }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Epoch;
    use crate::calendar::{Julian, Gregorian};
    use crate::date_time::DateTime;
    use crate::instant::Instant;

    #[test]
    fn check_epochs_and_conversion() {
        crate::setup_logging();

        let instant = Epoch::JulianCalendar.as_instant();
        let dt: DateTime<Julian> = From::from(instant);
        assert_eq!(dt, DateTime::<Julian>::new(1, 1, 1, 0, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::GregorianCalendar.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(1, 1, 1, 0, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::JulianPeriod.as_instant();
        let dt: DateTime<Julian> = From::from(instant);
        assert_eq!(dt, DateTime::<Julian>::new_bc(4713, 1, 1, 12, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::J1900_0.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(1899, 12, 31, 12, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::E1900_0.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(1900, 1, 1, 0, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::J1991_25.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(1991, 4, 2, 13, 30, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::J2000_0.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(2000, 1, 1, 12, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::J2100_0.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(2100, 1, 1, 12, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::J2200_0.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(2200, 1, 2, 12, 0, 0, 0).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::Unix.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        // the reason this is off is due to the conversion between Tt and Utc. FIXME (use UTC and zero this)
        assert_eq!(dt, DateTime::<Gregorian>::new(1970, 1, 1, 0, 0, 41, 184_000_000_000_000_000).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::Y2k.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        // the reason this is off is due to the conversion between Tt and Utc. FIXME (use UTC and zero this)
        assert_eq!(dt, DateTime::<Gregorian>::new(2000, 1, 1, 0, 1, 4, 184_000_000_000_000_000).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);

        let instant = Epoch::TimeStandard.as_instant();
        let dt: DateTime<Gregorian> = From::from(instant);
        assert_eq!(dt, DateTime::<Gregorian>::new(1977, 1, 1, 0, 0, 32, 184_000_000_000_000_000).unwrap());
        let check: Instant = From::from(dt);
        assert_eq!(instant, check);
    }

    #[test]
    fn test_instant_julian_day_formatted() {
        crate::setup_logging();

        assert_eq!(Epoch::GregorianCalendar.as_instant().as_julian_day_formatted(), "JD 1721425.5");
        assert_eq!(Epoch::JulianCalendar.as_instant().as_julian_day_formatted(), "JD 1721423.5");
        assert_eq!(Epoch::JulianPeriod.as_instant().as_julian_day_formatted(), "JD 0");
        assert_eq!(Epoch::J1900_0.as_instant().as_julian_day_formatted(), "JD 2415020");
        assert_eq!(Epoch::J1991_25.as_instant().as_julian_day_formatted(), "JD 2448349.0625");
        assert_eq!(Epoch::J2000_0.as_instant().as_julian_day_formatted(), "JD 2451545");
        assert_eq!(Epoch::J2100_0.as_instant().as_julian_day_formatted(), "JD 2488070");
        assert_eq!(Epoch::J2200_0.as_instant().as_julian_day_formatted(), "JD 2524595");

        // FIXME - this isn't right because UTC is not TT
        assert_eq!(Epoch::Unix.as_instant().as_julian_day_formatted(), "JD 2440587.5004766666666667");

        // FIXME - this isn't right because UTC is not TT
        assert_eq!(Epoch::Y2k.as_instant().as_julian_day_formatted(), "JD 2451544.5007428703703704");

        assert_eq!(Epoch::TimeStandard.as_instant().as_julian_day_formatted(), "JD 2443144.5003725");
    }

}
