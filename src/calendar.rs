
use crate::error::Error;
use crate::epoch::Epoch;
use crate::instant::Instant;

/// This specifies traditional Calendar settings that use the traditional 12 months
/// and have leap years. This is implemented for `Gregorian` and `Julian`. It does
/// not handle more esoteric calendars.
///
/// Zero and Negative years are handled in accordance with ISO 8601, such that
/// year 0 is 1 B.C., and year -1 is 2 B.C., etc. In general:
/// * _n_ B.C. is represented by year 1-_n_
/// * Year _-y_ represents year _y_+1 B.C. (for positive y).
pub trait Calendar {

    /// If the calendar is Gregorian (since we only handle Julian and Gregorian, this
    /// is all that needs to be defined to differentiate them)
    fn is_gregorian() -> bool;

    /// The name of the calendar
    #[must_use]
    fn name() -> &'static str {
        if <Self as Calendar>::is_gregorian() {
            "Gregorian"
        } else {
            "Julian"
        }
    }

    /// Epoch this calendar starts from
    #[must_use]
    fn epoch() -> Instant {
        if <Self as Calendar>::is_gregorian() {
            Epoch::GregorianCalendar.as_instant()
        } else {
            Epoch::JulianCalendar.as_instant()
        }
    }

    /// Answers the question: is this year a leap year?
    #[must_use]
    fn is_year_leap(year: i32) -> bool {
        if <Self as Calendar>::is_gregorian() {
            (year%4==0) && ((year%100!=0) || (year%400==0))
        } else {
            year%4==0
        }
    }

    /// Converts a `year`, `month` and (month)`day` into a day number which counts the number
    /// of days from the start of the calendar epoch
    ///
    /// `year` may range from -2147483648 .. 2147483647 covering every possible i32.
    ///
    /// `month` must be in the range 1 .. 12
    ///
    /// `day` may be out of the normal bounds. It will be adjusted.
    ///
    /// # Errors
    ///
    /// Will return a `Error::RangeError` if `month` or `day` are out of range.
    #[allow(clippy::manual_range_contains)]
    fn day_number(year: i32, month: u8, day: i64) -> Result<i64, Error> {
        if month<1 || month>12 { return Err(Error::RangeError); }

        // Zero basis days and months
        let mut m0 = i64::from(month).checked_sub(1).ok_or(Error::RangeError)?;
        let d0 = day.checked_sub(1).ok_or(Error::RangeError)?;

        // Change our zero point to 1 B.C. (year 0) March 1st (Feb now being month 11 pushing the
        // leap year day to the very end of the year)
        m0 = (m0 + 10) % 12;

        // Use a larger type for years so we can handle the entire range without
        // numerical overflows.  Also adjust for starting on March 1st.
        let y: i64 = i64::from(year) - m0/10;

        // Main calculation
        let mut day = {
            365*y

            // leap year first approximation
                + y/4

            // For dates before 1 B.C. (year 0) March 1st, we need to subtract 1 more day since
            // 1 B.C. (year 0) is a leap year that our calculations above didn't catch.
            // (To be branchless, we just use the sign bit from the year; i64 type stays negative)

                //- (if y<0 { 1 } else { 0 })
                + (y>>63)

            // The number of days between march 1st and the start of the mth month
            // after march (brilliant!) (306 is the days in the 10 months from mar-dec)
                + (m0*306 + 5)/10

            // and dont forget the day of the month itself (zero basis)
                + d0
        };

        if <Self as Calendar>::is_gregorian() {
            day = day
            // leap year second approximation, Gregorian
                - y/100
            // leap year third approximation, Gregorian
                + y/400;
        }

        // revert back to january 1 basis (we were at march 1st, we need to move ahead)
        Ok(day - 306)
    }

    /// Converts a day number which counts the number of days from the start of
    /// the calendar epoch into a year, month and day
    ///
    /// For the Gregorian calendar, `day_number` must fall in the range
    /// `-784_352_296_671` .. `784_352_295_938`
    /// which represent calendar dates `-2147483648-01-01` ..  `2147483647-12-31`
    /// respectively.
    ///
    /// For the Julian calendar, `day_number` must fall in the range
    /// `-784_368_402_798` .. `784_368_402_065`
    /// which represent calendar dates `-2147483648-01-01` ..  `2147483647-12-31`
    /// respectively.
    ///
    /// Returns a (year, month, day)
    ///
    /// # Errors
    ///
    /// Will return a `Error::RangeError` if `day_number` is out of range.
    ///
    /// # Panics
    ///
    /// Panics on assertions that should only fail if there is a bug.
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn from_day_number(mut day_number: i64) -> Result<(i32, u8, u8), Error> {

        // These extreme values have been checked, so we don't have to use
        // checked math operations in the main function (which are slower)
        let (min,max) = if <Self as Calendar>::is_gregorian() {
            (-784_352_296_671, 784_352_295_938)
        } else {
            (-784_368_402_798,784_368_402_065)
        };
        if day_number < min || day_number > max {
            return Err(Error::RangeError);
        }

        // Change to a March 1st basis, year 0 (back about 9 months from the epoch)
        // The leap day will be at the very end rather than somewhere annoyingly in the
        // middle.
        day_number += 306;

        let days_in_year_times_10000 = if <Self as Calendar>::is_gregorian() {
            365_2425
        } else {
            365_2500
        };

        // Calculate the year (march 1st basis)
        let mut offset_year: i64 = (10_000 * day_number + 14780) / days_in_year_times_10000;

        // Caculate the remaining days
        let calc_remaining_days = |day_number: i64, offset_year: i64| -> i64 {
            let zeroeth_year = offset_year>>63;
            let mut remaining_days = day_number - 365*offset_year - offset_year/4 - zeroeth_year;
            if <Self as Calendar>::is_gregorian() {
                remaining_days = remaining_days + offset_year/100 - offset_year/400;
            }
            remaining_days
        };
        let mut remaining_days = calc_remaining_days(day_number, offset_year);
        if remaining_days < 0 {
            offset_year -= 1;
            remaining_days = calc_remaining_days(day_number, offset_year);
        }

        let offset_month = (100*remaining_days + 52)/3060;

        // come back from our march-1st basis

        let year = offset_year + (offset_month + 2)/12;

        let month = (offset_month + 2)%12;
        assert!(month >= 0);
        assert!(month < 12);

        let day = remaining_days - (offset_month*306 + 5)/10;
        assert!(day < 31);
        assert!(day >= 0);

        Ok((year as i32, (month+1) as u8, (day+1) as u8))
    }

    /// Returns the number of days in a given month (year is required for leap year calculations)
    #[must_use]
    fn month_days(month: u8, year: i32) -> u8 {
        assert!(month>=1);
        assert!(month<=12);
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            2 => if <Self as Calendar>::is_year_leap(year + i32::from((month-1)/12)) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Julian;

impl Calendar for Julian {
    fn is_gregorian() -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Gregorian;

impl Calendar for Gregorian {
    fn is_gregorian() -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::{Calendar, Gregorian, Julian};

    #[test]
    fn test_gregorian_julian_date_matches() {
        crate::setup_logging();

        // JULIAN: October 5, 1582     (JD 2299161)
        // GREGORIAN: October 15, 1582 (JD 2299161)
        let dnj = Julian::day_number(1582,10,5).unwrap();
        let dng = Gregorian::day_number(1582,10,15).unwrap();
        // Julian day numbers are offset from Gregorian day numbers by 2
        assert_eq!(dnj - 2, dng);

        // The Julian Epoch (Julian Day 0) should match
        // [Wikipedia](https://en.wikipedia.org/wiki/Julian_day)
        // January 1, 4713 BCE, proleptic Julian calendar
        // November 24, 4714 BCE, in the proleptic Gregorian calendar)

        let dnj = Julian::day_number(-4713,1,1).unwrap();
        let dng = Gregorian::day_number(-4714,11,24).unwrap();
        // Julian day numbers are offset from Gregorian day numbers by 2
        assert_eq!(dnj - 2, dng);

        // Note: julian day numbers
        //   1 Jan 4713 BCE (Julian Calendar) -- 0
        //   1 Jan 1 CE (Julian Calendar) -- 1721424
        //   1 Jan 2000 CE (Julian Calendar) -- 2451558
        //   1 Jan 2000 CE (Gregorian Calendar) -- 2451545
    }

    #[test]
    fn test_calendar_gregorian_day_numbers() {
        crate::setup_logging();

        // Epoch (year 1)
        let dn = Gregorian::day_number(1,1,1).unwrap();
        assert_eq!(dn, 0);
        let (y,m,d) = Gregorian::from_day_number(0).unwrap();
        assert_eq!( (y,m,d), (1,1,1) );

        // One day earlier
        let dn = Gregorian::day_number(0,12,31).unwrap();
        assert_eq!(dn, -1);
        let (y,m,d) = Gregorian::from_day_number(-1).unwrap();
        assert_eq!( (y,m,d), (0,12,31) );

        // Days around the leap day in 1 BCE
        let mar1 = Gregorian::day_number(0,3,1).unwrap();
        assert_eq!(mar1, -306);
        let (y,m,d) = Gregorian::from_day_number(mar1).unwrap();
        assert_eq!( (y,m,d), (0,3,1) );

        let feb29 = Gregorian::day_number(0,2,29).unwrap();
        assert_eq!(feb29, -307);
        let (y,m,d) = Gregorian::from_day_number(feb29).unwrap();
        assert_eq!( (y,m,d), (0,2,29) );

        let feb28 = Gregorian::day_number(0,2,28).unwrap();
        assert_eq!(feb28, -308);
        let (y,m,d) = Gregorian::from_day_number(feb28).unwrap();
        assert_eq!( (y,m,d), (0,2,28) );

        // Epoch (year 5)
        let dn = Gregorian::day_number(4,1,1).unwrap();
        assert_eq!(dn, 365*3);
        let (y,m,d) = Gregorian::from_day_number(dn).unwrap();
        assert_eq!( (y,m,d), (4,1,1) );

        // year 1582
        let dn = Gregorian::day_number(1582,1,1).unwrap();
        assert_eq!(dn, 365*(1582-1) + (1582-1)/4 - (1582-1)/100 + (1582-1)/400);
        let (y,m,d) = Gregorian::from_day_number(dn).unwrap();
        assert_eq!( (y,m,d), (1582,1,1) );

        // year 1582,  1st of march
        let dn = Gregorian::day_number(1582,3,1).unwrap();
        assert_eq!(dn, 365*(1582-1) + (1582-1)/4 - (1582-1)/100 + (1582-1)/400 + 31 + 28);
        let (y,m,d) = Gregorian::from_day_number(dn).unwrap();
        assert_eq!( (y,m,d), (1582,3,1) );

        // year 1582, 15th of october
        let dn = Gregorian::day_number(1582,10,15).unwrap();
        assert_eq!(dn, 365*(1582-1) + (1582-1)/4 - (1582-1)/100 + (1582-1)/400
                   + 31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 14);
        let (y,m,d) = Gregorian::from_day_number(dn).unwrap();
        assert_eq!( (y,m,d), (1582,10,15) );

        // Year 2000
        let dn = Gregorian::day_number(2000,1,1).unwrap();
        assert_eq!(dn, 730119);
        let (y,m,d) = Gregorian::from_day_number(730119).unwrap();
        assert_eq!( (y,m,d) , (2000,1,1) );

        // Minimum
        let dn = Gregorian::day_number(-2147483648,1,1).unwrap();
        assert_eq!(dn, -784_352_296_671);

        let (y,m,d) = Gregorian::from_day_number(-784_352_296_671).unwrap();
        assert_eq!(y,-2147483648);
        assert_eq!(m,1);
        assert_eq!(d,1);

        // Maximum
        let dn = Gregorian::day_number(2147483647,12,31).unwrap();
        assert_eq!(dn, 784_352_295_938);

        let (y,m,d) = Gregorian::from_day_number(784_352_295_938).unwrap();
        assert_eq!(y,2147483647);
        assert_eq!(m,12);
        assert_eq!(d,31);
    }

    #[test]
    fn test_calendar_julian_day_numbers() {
        crate::setup_logging();

        // Epoch (year 1)
        let dn = Julian::day_number(1,1,1).unwrap();
        assert_eq!(dn, 0);
        let (y,m,d) = Julian::from_day_number(0).unwrap();
        assert_eq!(y,1);
        assert_eq!(m,1);
        assert_eq!(d,1);

        // Year 2000
        let dn = Julian::day_number(2000,1,1).unwrap();
        assert_eq!(dn, 730134);
        let (y,m,d) = Julian::from_day_number(dn).unwrap();
        assert_eq!(y,2000);
        assert_eq!(m,1);
        assert_eq!(d,1);

        // Minimum
        let dn = Julian::day_number(-2147483648,1,1).unwrap();
        assert_eq!(dn, -784_368_402_798);
        let (y,m,d) = Julian::from_day_number(dn).unwrap();
        assert_eq!(y,-2147483648);
        assert_eq!(m,1);
        assert_eq!(d,1);

        // Maximum
        let dn = Julian::day_number(2147483647,12,31).unwrap();
        assert_eq!(dn, 784_368_402_065);
        let (y,m,d) = Julian::from_day_number(dn).unwrap();
        assert_eq!(y,2147483647);
        assert_eq!(m,12);
        assert_eq!(d,31);
    }
}
