use std::cmp::{Ordering, PartialEq};
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::calendar::{Calendar, Gregorian, Julian};
use crate::duration::Duration;
use crate::epoch::Epoch;
use crate::error::Error;
use crate::instant::Instant;
use crate::standard::{Standard, Tai, Tcg, Tt, Utc};
use crate::{ATTOS_PER_SEC_F64, ATTOS_PER_SEC_I64, ATTOS_PER_SEC_U64};

/// A calendar date and time, with attosecond precision, representing the
/// time elapsed since the start of the Common Era in a traditional way
/// according to a particular time `Standard`.
///
/// `DateTime`s are type parameterized by a `Calendar` type which is either
/// `Gregorian` or `Julian`.
///
/// Normal ranges for values are as follows:
///
/// * year: any i32 value (`-2_147_483_648` .. `2_147_483_647`)
/// * month: `1` .. `12`
/// * day: `1` .. `31` (or less in some months)
/// * hour: `0` .. `23`
/// * minute: `0` .. `59`
/// * second: `0` .. `60` (60 is only used under leap-second time standards)
/// * attosecond: `0` .. `999_999_999_999_999_999`
///
/// Even when years are negative, the other values must remain in the positive ranges
/// as specified (i.e. negative values are not a reflection against zero, but just an
/// extension further backwards)
///
/// Zero and Negative years are handled in accordance with ISO 8601, such that
/// year 0 is 1 B.C., and year -1 is 2 B.C., etc. In general:
/// * _n_ B.C. is represented by year 1-_n_
/// * Year _-y_ represents year _y_+1 B.C. (for positive y).
///
/// This type is proleptic; that is, it allows values for dates outside the
/// normal usage period of the `Calendar`.
///
/// The oldest date representable is `-2147483648-01-01 00:00:00.000000000000000000`
///
/// The newest date representable is `2147483647-12-31 23:59:59.999999999999999999`
///
/// Internally this is stored in a packed format and is 128 bits in size.
///
/// This represents the same thing that an `Instant` does, but it makes `Calendar` data
/// easier to work with, and has such date precomputed and packed within.
#[derive(Clone, Copy)] // is also Send
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DateTime<C: Calendar, S: Standard> {
    packed: u64,
    attos: u64,
    _cal: PhantomData<C>,
    _std: PhantomData<S>,
}

// NOTE: Day and Month are packed with 0 basis (0 = 1st day or 1st month)
const YEAR_BITS: u64 = 0xFFFF_FFFF_0000_0000;
const SECOND_BITS: u64 = 0x0000_0000_FC00_0000;
const MINUTE_BITS: u64 = 0x0000_0000_03F0_0000;
const HOUR_BITS: u64 = 0x0000_0000_000F_8000;
const DAY0_BITS: u64 = 0x0000_0000_0000_7C00;
const _RESERVED_BITS: u64 = 0x0000_0000_0000_03F0;
const MONTH0_BITS: u64 = 0x0000_0000_0000_000F;
// We pack all values (except attos) into a u64 at the following offsets:
const YEAR_OFFSET: usize = 32;
const SECOND_OFFSET: usize = 26;
const MINUTE_OFFSET: usize = 20;
const HOUR_OFFSET: usize = 15;
const DAY0_OFFSET: usize = 10;
const MONTH0_OFFSET: usize = 0;

// Pack a value into the packed field
#[inline]
const fn pack(packed: &mut u64, bits: u64, offset: usize, value: u64) {
    *packed &= !bits; // zero
    *packed |= value << offset; // set
}

// Pack a value into the packed field, only if you know it's already zero
#[inline]
const fn pack_without_clearing(packed: &mut u64, offset: usize, value: u64) {
    *packed |= value << offset; // set
}

// Unpack a value from the packed field
#[inline]
const fn unpack(packed: u64, bits: u64, offset: usize) -> u64 {
    (packed & bits) >> offset
}

impl<C: Calendar, S: Standard> DateTime<C, S> {
    /// Create a new `DateTime` with the given parts.
    ///
    /// # Safety
    ///
    /// Parameter values must be within normal ranges, or else the results
    /// are not defined.
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_lossless)]
    #[must_use]
    pub const unsafe fn new_unchecked(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        attosecond: u64,
    ) -> Self {
        let mut packed: u64 = 0;
        pack_without_clearing(&mut packed, YEAR_OFFSET, year as u64);
        pack_without_clearing(&mut packed, SECOND_OFFSET, second as u64);
        pack_without_clearing(&mut packed, MINUTE_OFFSET, minute as u64);
        pack_without_clearing(&mut packed, HOUR_OFFSET, hour as u64);
        pack_without_clearing(&mut packed, DAY0_OFFSET, (day - 1) as u64);
        pack_without_clearing(&mut packed, MONTH0_OFFSET, (month - 1) as u64);

        Self {
            packed,
            attos: attosecond,
            _cal: PhantomData,
            _std: PhantomData,
        }
    }

    /// Create a new `DateTime` from the given parts.
    ///
    /// Values must be within normal ranges. See `DateTime` for details.
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if any input is outside of the normal
    /// range (months from 1-12, days from 1-31, hours from 0-23, minutes from
    /// 0-59, seconds from 0-60, attoseconds from 0-999_999_999_999_999_999)
    #[allow(clippy::manual_range_contains)]
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        attosecond: u64,
    ) -> Result<Self, Error> {
        if month < 1 || month > 12 {
            return Err(Error::RangeError);
        }
        if day < 1 || day > C::month_days(month, year) {
            return Err(Error::RangeError);
        }
        if hour > 23 {
            return Err(Error::RangeError);
        }
        if minute > 59 {
            return Err(Error::RangeError);
        }
        if second > 60 {
            return Err(Error::RangeError);
        }
        if attosecond > 999_999_999_999_999_999 {
            return Err(Error::RangeError);
        }

        Ok(unsafe { Self::new_unchecked(year, month, day, hour, minute, second, attosecond) })
    }

    /// Create a new `DateTime` from the given parts, with BC years.
    ///
    /// Values must be within normal ranges. See `DateTime` for details.
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if any input is outside of the normal
    /// range (months from 1-12, days from 1-31, hours from 0-23, minutes from
    /// 0-59, seconds from 0-60, attoseconds from 0-999_999_999_999_999_999)
    #[allow(clippy::manual_range_contains)]
    pub fn new_bc(
        bc_year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        attosecond: u64,
    ) -> Result<Self, Error> {
        let year = 1 - bc_year;
        Self::new(year, month, day, hour, minute, second, attosecond)
    }

    /// Create a new `DateTime` from the given parts that may be out of range.
    ///
    /// Values that are out of normal ranges are allowed, including values that are negative.
    /// This function will adjust the input your provide into a normal form.
    ///
    /// The types we are working with are large i64 types, but they can still overflow.
    /// Overflow is not detected or reported (FIXME).
    ///
    /// # Warning
    ///
    /// This does not make any leap second adjustments under Utc. We presume that after
    /// rolling up the numbers, that is the right calendar date.
    ///
    /// # Panics
    ///
    /// Shouldn't panic but several math assertions may trigger if we have a bug when
    /// compiled in development mode.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new_abnormal(
        mut year: i32,
        month: i64,
        day: i64,
        mut hour: i64,
        mut minute: i64,
        mut second: i64,
        mut attosecond: i64,
    ) -> Self {
        use crate::divmod_i64;

        let mut month0 = month - 1;
        let mut day0 = day - 1;

        // NOTE: we don't check whether or not roll ups are necessary.
        // they don't hurt when they are not necessary as they only
        // add zero, and branch code tends to be more expensive. So we
        // do it unconditionally.

        // roll up attoseconds into seconds (handling negative values)
        let (div, modulus) = divmod_i64(attosecond, ATTOS_PER_SEC_I64);
        second += div;
        attosecond = modulus;
        assert!(attosecond >= 0);
        assert!(attosecond < ATTOS_PER_SEC_I64);

        // roll up seconds into minutes (handling negative values)
        let (div, modulus) = divmod_i64(second, 60);
        minute += div;
        second = modulus;
        assert!(second >= 0);
        assert!(second < 60);

        // roll up minutes into hours
        let (div, modulus) = divmod_i64(minute, 60);
        hour += div;
        minute = modulus;
        assert!(minute >= 0);
        assert!(minute < 60);

        // roll up hours into days
        let (div, modulus) = divmod_i64(hour, 24);
        day0 += div;
        hour = modulus;
        assert!(hour >= 0);
        assert!(hour < 24);

        // We handle the overflowing days further down

        // We cannot handle overflowing months or negative months in
        // the day_number() function, so we have to normalize months first
        let (div, modulus) = divmod_i64(month0, 12);
        year += div as i32;

        month0 = modulus;
        assert!(month0 >= 0);
        assert!(month0 < 12);

        // Compute the day number
        // NOTE: day may be overflowing or negative.
        //       C::day_number needs to handle this condition.
        let dn = C::day_number(year, (month0 + 1).try_into().unwrap(), day0 + 1).unwrap();

        // Now set the date from that day number
        let (y, m, d) = C::from_day_number(dn).unwrap();

        unsafe {
            Self::new_unchecked(
                y,
                m,
                d,
                hour as u8,
                minute as u8,
                second as u8,
                attosecond as u64,
            )
        }
    }

    /// Create a `DateTime` from a day number (integer).
    ///
    /// January 1st of 1 A.D. (Common Era) is the epoch and has a day number of 0.
    ///
    /// Hour, minute, second and attosecond will be zero.
    ///
    /// # Errors
    ///
    /// Will return a `Error::RangeError` if `day_number` is out of range.
    pub fn from_day_number(day_number: i64) -> Result<Self, Error> {
        let (year, month, day) = C::from_day_number(day_number)?;
        unsafe { Ok(Self::new_unchecked(year, month, day, 0, 0, 0, 0)) }
    }

    /// Create a `DateTime` from a day number (integer) and day fraction (float).
    ///
    /// January 1st of 1 A.D. (Common Era) is the epoch and has a day number of 0.
    ///
    /// # Errors
    ///
    /// Will return a `Error::RangeError` if `day_number` is out of range.
    ///
    /// Will return `Error::RangeError` if `day_fraction` is <0.0 or >=1.0
    ///
    /// # Panics
    ///
    /// Panics on assertions that should only fail if there is a bug.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    pub fn from_day_number_and_fraction(day_number: i64, day_fraction: f64) -> Result<Self, Error> {
        if day_fraction < 0.0 {
            return Err(Error::RangeError);
        }
        if day_fraction >= 1.0 {
            return Err(Error::RangeError);
        }

        let (year, month, day) = C::from_day_number(day_number)?;
        let (hour, min, sec, atto) = {
            const FACTOR: i64 = 100_000_000_000_000;

            // f64's mantissa is only 52 bits wide. We can only get 52 bits of precision
            // at maximum. So the output attoseconds will end with some zeros in any case,
            // and we use FACTOR (larger than an attosecond) so we don't overflow.
            let parts = (((FACTOR * 86400) as f64) * day_fraction) as i64;

            // We don't need euclidean modulus here because parts is guaranteed to
            // not be negative
            let mut s = parts / FACTOR;
            let atto = parts % FACTOR * 10000;

            let mut m = s / 60;
            s %= 60;
            assert!(s < 60);

            let h = m / 60;
            assert!(h < 24);
            m %= 60;
            assert!(m < 60);

            (h as u8, m as u8, s as u8, atto as u64)
        };

        Ok(unsafe { Self::new_unchecked(year, month, day, hour, min, sec, atto) })
    }

    /// Create a `DateTime` from a `Duration` from the calendar epoch
    /// (with the calendar epoch represented in time `Standard` `S`, such
    /// that no time Standard conversions are done here).
    ///
    /// When using TCG, there is precision loss since we use double
    /// precision floating point arithmetic, which is less precise than
    /// the integers used in `Duration`).
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // literal value won't
    #[allow(clippy::cast_precision_loss)]
    pub fn from_duration_from_epoch(duration: Duration) -> Self {
        // We will construct a calendar-applicable Duration,
        // only for computing the calendar date, NOT for actual instant
        // usage (as it would be wrong).
        let mut cal_duration: Duration = duration;

        let mut inside_a_leap: bool = false;

        // Leap second adjustment
        if S::abbrev() == "UTC" {
            let instant = C::epoch() + duration;

            let leaps = crate::leaps::leap_seconds_elapsed_at(instant);

            // Check if inside a leap second
            inside_a_leap =
                crate::leaps::leap_seconds_elapsed_at(instant + Duration::new(1, 0)) > leaps;

            // Remove the leap seconds:
            cal_duration -= Duration::new(leaps, 0);
        }

        // (Maybe) change time standards
        cal_duration -= S::tt_offset();

        if let Some(scale) = S::tt_scale() {
            let dur_since_sync = {
                let instant = C::epoch() + duration;
                let dss = instant - Epoch::TimeStandard.as_instant();
                dss.secs as f64 + dss.attos as f64 / ATTOS_PER_SEC_F64
            };
            let shift = dur_since_sync * scale;
            cal_duration += Duration::from_seconds(shift);
        }

        if inside_a_leap {
            cal_duration -= Duration::new(1, 0);
        }

        let mut output = Self::new_abnormal(1, 1, 1, 0, 0, cal_duration.secs, cal_duration.attos);

        if inside_a_leap {
            assert_eq!(output.second(), 59); // because we removed 1 second just above
            output.set_second(60).unwrap();
        }

        output
    }

    /// The year part
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn year(&self) -> i32 {
        unpack(self.packed, YEAR_BITS, YEAR_OFFSET) as i32
    }

    /// The year part in BC years
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn year_bc(&self) -> i32 {
        1 - self.year()
    }

    /// The month part. Ranges from 1 .. 12
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn month(&self) -> u8 {
        unpack(self.packed, MONTH0_BITS, MONTH0_OFFSET) as u8 + 1
    }

    /// The month part where January is mapped to 0. Ranges from 0 .. 11
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn month0(&self) -> u8 {
        unpack(self.packed, MONTH0_BITS, MONTH0_OFFSET) as u8
    }

    /// The day part. Ranges from 1 .. 31
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn day(&self) -> u8 {
        unpack(self.packed, DAY0_BITS, DAY0_OFFSET) as u8 + 1
    }

    /// The day part where the 1st day is mapped to 0. Ranges from 0 .. 30
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn day0(&self) -> u8 {
        unpack(self.packed, DAY0_BITS, DAY0_OFFSET) as u8
    }

    /// The hour part. Ranges from 0 .. 23
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn hour(&self) -> u8 {
        unpack(self.packed, HOUR_BITS, HOUR_OFFSET) as u8
    }

    /// The minute part. Ranges from 0 .. 59
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn minute(&self) -> u8 {
        unpack(self.packed, MINUTE_BITS, MINUTE_OFFSET) as u8
    }

    /// The second part. Ranges from 0 .. 59
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn second(&self) -> u8 {
        unpack(self.packed, SECOND_BITS, SECOND_OFFSET) as u8
    }

    /// The attosecond part. Ranges from `0` .. `999_999_999_999_999_999`
    #[must_use]
    #[inline]
    pub const fn attosecond(&self) -> u64 {
        self.attos
    }

    /// The day of the week from 1 (Monday) .. 7 (Sunday) (ISO 8601)
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_truncation)] // Bound by 0-7, it won't
    pub fn weekday(&self) -> u8 {
        let offset = if C::is_gregorian() { 0 } else { 5 };
        (self.day_number() + offset).rem_euclid(7) as u8 + 1
    }

    /// The date part
    ///
    /// Returns (year, month, day)
    #[must_use]
    #[inline]
    pub const fn date(&self) -> (i32, u8, u8) {
        (self.year(), self.month(), self.day())
    }

    /// The time part
    ///
    /// Returns (hour, minute, second, attosecond)
    #[must_use]
    #[inline]
    pub const fn time(&self) -> (u8, u8, u8, u64) {
        (self.hour(), self.minute(), self.second(), self.attosecond())
    }

    /// Set the year, leaving other fields unchanged
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    pub const fn set_year(&mut self, year: i32) {
        // "year as u64" treats the sign bit as a bit in the MSB, which is what we want,
        // because we must preserve negative years in our packing.
        pack(&mut self.packed, YEAR_BITS, YEAR_OFFSET, year as u64);
    }

    /// Set the year with a BC year, leaving other fields unchanged
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    pub const fn set_year_bc(&mut self, year_bc: i32) {
        let year = 1 - year_bc;
        // "year as u64" treats the sign bit as a bit in the MSB, which is what we want,
        // because we must preserve negative years in our packing.
        pack(&mut self.packed, YEAR_BITS, YEAR_OFFSET, year as u64);
    }

    /// Set the month, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `month` is <1 or >12.
    #[allow(clippy::manual_range_contains)]
    pub fn set_month(&mut self, month: u8) -> Result<(), Error> {
        if month < 1 || month > 12 {
            return Err(Error::RangeError);
        }
        if self.day() > C::month_days(month, self.year()) {
            return Err(Error::RangeError);
        }
        pack(
            &mut self.packed,
            MONTH0_BITS,
            MONTH0_OFFSET,
            u64::from(month - 1),
        );
        Ok(())
    }

    /// Set the day, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `day` is outside of the range of days
    /// for the month.
    pub fn set_day(&mut self, day: u8) -> Result<(), Error> {
        if day < 1 || day > C::month_days(self.month(), self.year()) {
            return Err(Error::RangeError);
        }
        pack(&mut self.packed, DAY0_BITS, DAY0_OFFSET, u64::from(day - 1));
        Ok(())
    }

    /// Set the hour, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `hour` is greater than 23.
    pub fn set_hour(&mut self, hour: u8) -> Result<(), Error> {
        if hour > 23 {
            return Err(Error::RangeError);
        }
        pack(&mut self.packed, HOUR_BITS, HOUR_OFFSET, u64::from(hour));
        Ok(())
    }

    /// Set the minute, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `minute` is greater than 59.
    pub fn set_minute(&mut self, minute: u8) -> Result<(), Error> {
        if minute > 59 {
            return Err(Error::RangeError);
        }
        pack(
            &mut self.packed,
            MINUTE_BITS,
            MINUTE_OFFSET,
            u64::from(minute),
        );
        Ok(())
    }

    /// Set the second, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `second` is greater than 60.
    /// The second of '60' should only be used for leapsecond situations, but
    /// no error is thrown if used otherwise.
    pub fn set_second(&mut self, second: u8) -> Result<(), Error> {
        if second > 60 {
            return Err(Error::RangeError);
        }
        pack(
            &mut self.packed,
            SECOND_BITS,
            SECOND_OFFSET,
            u64::from(second),
        );
        Ok(())
    }

    /// Set the attosecond, leaving other fields unchanged
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if `attosecond` are out of the proscribed range
    /// (more than 1 seconds worth of attoseconds)
    pub const fn set_attosecond(&mut self, attosecond: u64) -> Result<(), Error> {
        if attosecond > ATTOS_PER_SEC_U64 {
            return Err(Error::RangeError);
        }
        self.attos = attosecond;
        Ok(())
    }

    /// Set the date part (year, month, day)
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if any input values are out of the proscribed range
    pub fn set_date(&mut self, date: (i32, u8, u8)) -> Result<(), Error> {
        self.set_year(date.0);
        self.set_month(date.1)?;
        self.set_day(date.2)?;
        Ok(())
    }

    /// Set the time part (hour, minute, second, attosecond)
    ///
    /// # Errors
    ///
    /// Will return `Error::RangeError` if any input values are out of the proscribed range
    pub fn set_time(&mut self, date: (u8, u8, u8, u64)) -> Result<(), Error> {
        self.set_hour(date.0)?;
        self.set_minute(date.1)?;
        self.set_second(date.2)?;
        self.set_attosecond(date.3)?;
        Ok(())
    }

    /// Day number (integer).
    ///
    /// January 1st of 1 A.D. (Common Era) is the epoch and has a day number of 0.
    ///
    /// # Panics
    ///
    /// Will only panic on a bug that caused internal values to get out of range.
    #[must_use]
    pub fn day_number(&self) -> i64 {
        C::day_number(self.year(), self.month(), i64::from(self.day())).unwrap()
    }

    /// Day fraction, fractional part of the day since midnight
    ///
    /// This isn't attosecond accurate because a day contains more attoseconds than
    /// can fit in a f64 (which has 52 bits of precision).  However, it should be
    /// accurate to 10,000 attoseconds.
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn day_fraction(&self) -> f64 {
        // In order to preserve as much precision as we can, we count
        // in units of 10^-14 seconds (10,000 attoseconds).
        // A 24-hour duration of these won't overflow a u64. Anything
        // smaller would.
        const FACTOR: u64 = 100_000_000_000_000;

        (u64::from(self.hour()) * 3600 * FACTOR
            + u64::from(self.minute()) * 60 * FACTOR
            + u64::from(self.second()) * FACTOR
            + (self.attosecond() / 10000)) as f64
            / 8_640_000_000_000_000_000.
    }

    /// Duration from the calendar epoch.
    ///
    /// When using TCG, there is precision loss since we use double
    /// precision floating point arithmetic, which is less precise than
    /// the integers used in `Duration`).
    ///
    /// # Panics
    ///
    /// Shouldn't panic unless this library has a bug.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn duration_from_epoch(&self) -> Duration {
        let naive = {
            let dn = self.day_number();
            let seconds = dn * 86400
                + i64::from(self.hour()) * 3600
                + i64::from(self.minute()) * 60
                + i64::from(self.second());
            Duration::new(seconds, i64::try_from(self.attosecond()).unwrap())
        };

        // Shift time standards into Tt because calendar epochs are in Tt
        let mut d = naive + S::tt_offset();

        if let Some(scale) = S::tt_scale() {
            let dur_since_sync = {
                let instant = C::epoch() + d;
                let dss = instant - Epoch::TimeStandard.as_instant();
                dss.secs as f64 + dss.attos as f64 / ATTOS_PER_SEC_F64
            };
            let shift = dur_since_sync * scale;
            d -= Duration::from_seconds(shift);
        }

        // Leap second adjustment
        if S::abbrev() == "UTC" {
            let close: Instant = C::epoch() + d;
            let approx_leaps = crate::leaps::leap_seconds_elapsed_at(close);
            let actual_leaps =
                crate::leaps::leap_seconds_elapsed_at(close + Duration::new(approx_leaps + 1, 0));
            d += Duration::new(actual_leaps, 0);
        }

        d
    }
}

impl<C: Calendar, S: Standard> fmt::Debug for DateTime<C, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:018} {} {}",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.attosecond(),
            C::name(),
            S::abbrev()
        )
    }
}

impl<C: Calendar, S: Standard> fmt::Display for DateTime<C, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:018} {} {}",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.attosecond(),
            C::name(),
            S::abbrev()
        )
    }
}

impl<C: Calendar, S: Standard> Add<Duration> for DateTime<C, S> {
    type Output = Self;

    #[allow(clippy::cast_possible_wrap)]
    fn add(self, rhs: Duration) -> Self {
        Self::new_abnormal(
            self.year(),
            i64::from(self.month()),
            i64::from(self.day()),
            i64::from(self.hour()),
            i64::from(self.minute()),
            i64::from(self.second()) + rhs.seconds_part(),
            self.attosecond() as i64 + rhs.attos_part(),
        )
    }
}

impl<C: Calendar, S: Standard> Sub<Duration> for DateTime<C, S> {
    type Output = Self;

    #[allow(clippy::cast_possible_wrap)]
    fn sub(self, rhs: Duration) -> Self {
        Self::new_abnormal(
            self.year(),
            i64::from(self.month()),
            i64::from(self.day()),
            i64::from(self.hour()),
            i64::from(self.minute()),
            i64::from(self.second()) - rhs.seconds_part(),
            self.attosecond() as i64 - rhs.attos_part(),
        )
    }
}

impl<C: Calendar, S: Standard> Sub for DateTime<C, S> {
    type Output = Duration;

    #[allow(clippy::cast_possible_wrap)]
    fn sub(self, other: Self) -> Duration {
        let secs = (self.day_number() - other.day_number()) * 86400
            + (i64::from(self.hour()) - i64::from(other.hour())) * 3600
            + (i64::from(self.minute()) - i64::from(other.minute())) * 60
            + (i64::from(self.second()) - i64::from(other.second()));
        let attos = self.attosecond() as i64 - other.attosecond() as i64;
        Duration::new(secs, attos) // it will normalize
    }
}

impl<C: Calendar, S: Standard> PartialEq<Self> for DateTime<C, S> {
    fn eq(&self, other: &Self) -> bool {
        self.packed == other.packed && self.attos == other.attos
    }
}

impl<C: Calendar, S: Standard> Eq for DateTime<C, S> {}

impl<C: Calendar, S: Standard> Ord for DateTime<C, S> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.year() != other.year() {
            return self.year().cmp(&other.year());
        }
        if self.month() != other.month() {
            return self.month().cmp(&other.month());
        }
        if self.day() != other.day() {
            return self.day().cmp(&other.day());
        }
        if self.hour() != other.hour() {
            return self.hour().cmp(&other.hour());
        }
        if self.minute() != other.minute() {
            return self.minute().cmp(&other.minute());
        }
        if self.second() != other.second() {
            return self.second().cmp(&other.second());
        }
        self.attosecond().cmp(&other.attosecond())
    }
}

impl<C: Calendar, S: Standard> PartialOrd<Self> for DateTime<C, S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Calendar, S: Standard> Hash for DateTime<C, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.packed.hash(state);
        self.attos.hash(state);
    }
}

unsafe impl<C: Calendar, S: Standard> Send for DateTime<C, S> {}

impl<S: Standard> TryFrom<DateTime<Gregorian, S>> for DateTime<Julian, S> {
    type Error = Error;
    fn try_from(input: DateTime<Gregorian, S>) -> Result<Self, Self::Error> {
        let dn = input.day_number() + 2;
        let mut r = Self::from_day_number(dn)?;
        r.set_time(input.time())?;
        Ok(r)
    }
}

impl<S: Standard> TryFrom<DateTime<Julian, S>> for DateTime<Gregorian, S> {
    type Error = Error;
    fn try_from(input: DateTime<Julian, S>) -> Result<Self, Self::Error> {
        let dn = input.day_number() - 2;
        let mut r = Self::from_day_number(dn)?;
        r.set_time(input.time())?;
        Ok(r)
    }
}

// For converting calendars between time standards, we cannot
// use generics because we cannot specify that the two standards
// are not equal. So we specify all pairwise combinations:
impl<C: Calendar> From<DateTime<C, Tt>> for DateTime<C, Tai> {
    fn from(s1: DateTime<C, Tt>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tt>> for DateTime<C, Utc> {
    fn from(s1: DateTime<C, Tt>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tt>> for DateTime<C, Tcg> {
    fn from(s1: DateTime<C, Tt>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}

impl<C: Calendar> From<DateTime<C, Tai>> for DateTime<C, Tt> {
    fn from(s1: DateTime<C, Tai>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tai>> for DateTime<C, Utc> {
    fn from(s1: DateTime<C, Tai>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tai>> for DateTime<C, Tcg> {
    fn from(s1: DateTime<C, Tai>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}

impl<C: Calendar> From<DateTime<C, Utc>> for DateTime<C, Tt> {
    fn from(s1: DateTime<C, Utc>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Utc>> for DateTime<C, Tai> {
    fn from(s1: DateTime<C, Utc>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Utc>> for DateTime<C, Tcg> {
    fn from(s1: DateTime<C, Utc>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}

impl<C: Calendar> From<DateTime<C, Tcg>> for DateTime<C, Tt> {
    fn from(s1: DateTime<C, Tcg>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tcg>> for DateTime<C, Tai> {
    fn from(s1: DateTime<C, Tcg>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}
impl<C: Calendar> From<DateTime<C, Tcg>> for DateTime<C, Utc> {
    fn from(s1: DateTime<C, Tcg>) -> Self {
        // Convert through Instant
        let i: Instant = s1.into();
        i.into()
    }
}

#[cfg(test)]
mod test {
    use super::DateTime;
    use crate::calendar::{Gregorian, Julian};
    use crate::standard::{Tai, Tcg, Tt, Utc};
    use crate::{ATTOS_PER_SEC_I64, ATTOS_PER_SEC_U64};
    use crate::{Duration, Epoch, Instant};

    #[test]
    fn test_range_errors() {
        assert!(DateTime::<Gregorian, Tt>::new(2000, 0, 31, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2000, 13, 31, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2000, 6, 0, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2000, 6, 31, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2000, 7, 32, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2003, 2, 29, 0, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2004, 2, 29, 24, 0, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2004, 2, 29, 0, 60, 0, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2004, 2, 29, 0, 0, 61, 0).is_err());
        assert!(DateTime::<Gregorian, Tt>::new(2004, 2, 29, 0, 0, 0, ATTOS_PER_SEC_U64).is_err());

        let _ = DateTime::<Gregorian, Tt>::new_abnormal(0, 1, 31, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2000, 0, 31, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2000, 13, 31, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2000, 6, 0, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2000, 6, 31, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2000, 7, 32, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2003, 2, 29, 0, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2004, 2, 29, 24, 0, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2004, 2, 29, 0, 60, 0, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2004, 2, 29, 0, 0, 61, 0);
        let _ = DateTime::<Gregorian, Tt>::new_abnormal(2004, 2, 29, 0, 0, 0, ATTOS_PER_SEC_I64);
    }

    #[test]
    fn test_normalize() {
        // This is right out of leap second file for 1 Jan 1972
        let dt = DateTime::<Gregorian, Tt>::new_abnormal(1900, 1, 1, 0, 0, 2272060800, 0);
        assert_eq!(dt.year(), 1972);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.attosecond(), 0);

        // 3rd leap second
        // NOTE FIXME ELSEWHERE: t1900 must not include leap seconds, or else
        // this would be off by 2 as it does not account for the 2 leap seconds
        // added prior to it.
        let dt = DateTime::<Gregorian, Tt>::new_abnormal(1900, 1, 1, 0, 0, 2303683200, 0);
        assert_eq!(dt.year(), 1973);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.attosecond(), 0);

        // Test hour roll over that crosses a month during the end of
        // February during a leap year
        let dt = DateTime::<Gregorian, Tt>::new_abnormal(1972, 2, 29, 25, 0, 0, 0);
        assert_eq!(dt.month(), 3); // mar
        assert_eq!(dt.day(), 1); // 1st
        assert_eq!(dt.hour(), 1);

        // Test some negative values
        let dt = DateTime::<Gregorian, Tt>::new_abnormal(
            2000,
            1 - 11,
            1 + (365 - 31),
            -12,
            60 * 12,
            0,
            0,
        );
        // We subtract 11 months, but add back the (365-11) days
        // We subtract 12 hours, but add back the (60*12) minutes
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.attosecond(), 0);

        // Test further negative values
        let dt =
            DateTime::<Gregorian, Tt>::new_abnormal(2000, 1 - 60, 1 + (365 * 4 + 366), 0, 0, 0, 0);
        // We subtract 60 months, but add back the (365 + 365 + 365 + 366 + 365) days
        // We subtract 12 hours, but add back the (60*12) minutes
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);

        // Test year rollover
        let dt = DateTime::<Gregorian, Tt>::new_abnormal(1970, 12, 31, 25, 0, 0, 0);
        assert_eq!(dt.year(), 1971);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 1);
    }

    #[test]
    fn test_day_number() {
        let dt = DateTime::<Gregorian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap(); // year 1
        assert_eq!(dt.day_number(), 0);

        let dt2 = DateTime::<Gregorian, Tt>::from_day_number(dt.day_number()).unwrap();
        assert_eq!(dt, dt2);

        let dt = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(dt.day_number(), 730119);

        let dt2 = DateTime::<Gregorian, Tt>::from_day_number(dt.day_number()).unwrap();
        assert_eq!(dt, dt2);

        assert_eq!(dt2.day_number(), dt.day_number())
    }

    #[test]
    fn test_day_fraction() {
        use float_cmp::ApproxEq;
        let g1 = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 12, 0, 0, 0).unwrap();
        assert!(g1.day_fraction().approx_eq(0.5, (0.0, 1)));
        let g2 = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 18, 0, 0, 0).unwrap();
        assert!(g2.day_fraction().approx_eq(0.75, (0.0, 1)));
        let g3 = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 0, 0, 1, 0).unwrap();
        assert!(g3.day_fraction().approx_eq(1. / 86400., (0.0, 1)));

        let g4 =
            DateTime::<Gregorian, Tt>::from_day_number_and_fraction(g1.day_number(), 0.75).unwrap();
        assert_eq!(g4, g2);

        let g4 =
            DateTime::<Gregorian, Tt>::from_day_number_and_fraction(g1.day_number(), 19. / 97.)
                .unwrap();
        assert!(g4.day_fraction().approx_eq(19. / 97., (0.0, 1)));
    }

    #[test]
    fn test_extractors() {
        let g = DateTime::<Gregorian, Tt>::new(1965, 3, 7, 14, 29, 42, 500_000_000_000_000_000)
            .unwrap();
        assert_eq!(g.year(), 1965);
        assert_eq!(g.month(), 3);
        assert_eq!(g.month0(), 2);
        assert_eq!(g.day(), 7);
        assert_eq!(g.day0(), 6);
        assert_eq!(g.hour(), 14);
        assert_eq!(g.minute(), 29);
        assert_eq!(g.second(), 42);
        assert_eq!(g.attosecond(), 500_000_000_000_000_000);
    }

    #[test]
    fn test_setters() {
        let mut g = DateTime::<Gregorian, Tt>::new(1965, 3, 7, 14, 29, 42, 500_000_000_000_000_000)
            .unwrap();

        g.set_year(1921);
        assert_eq!(g.year(), 1921);

        g.set_month(1).unwrap();
        assert_eq!(g.month(), 1);

        g.set_day(17).unwrap();
        assert_eq!(g.day(), 17);

        g.set_hour(3).unwrap();
        assert_eq!(g.hour(), 3);

        g.set_minute(55).unwrap();
        assert_eq!(g.minute(), 55);

        g.set_second(51).unwrap();
        assert_eq!(g.second(), 51);

        g.set_attosecond(123_456_789_012_345_678).unwrap();
        assert_eq!(g.attosecond(), 123_456_789_012_345_678);

        let h = DateTime::<Gregorian, Tt>::new(1921, 1, 17, 3, 55, 51, 123_456_789_012_345_678)
            .unwrap();

        assert_eq!(g, h);

        let mut g = DateTime::<Gregorian, Tt>::new(1997, 3, 30, 17, 24, 06, 2340897).unwrap();
        assert!(g.set_month(2).is_err());
        assert_eq!(g.month(), 3);
        assert!(g.set_day(28).is_ok());
        assert!(g.set_month(2).is_ok());
        assert_eq!(g.month(), 2);
        assert_eq!(g.day(), 28);
    }

    #[test]
    fn test_comparison() {
        let g = DateTime::<Gregorian, Tt>::new(1965, 3, 7, 14, 29, 42, 500_000_000_000_000_000)
            .unwrap();
        let h = DateTime::<Gregorian, Tt>::new(1966, 1, 17, 3, 55, 51, 123_456_789_012_345_678)
            .unwrap();
        let i = DateTime::<Gregorian, Tt>::new(1966, 3, 7, 14, 29, 42, 500_000_000_000_000_000)
            .unwrap();
        let j = DateTime::<Gregorian, Tt>::new(1966, 3, 7, 14, 29, 42, 500_000_000_000_000_000)
            .unwrap();
        assert!(g < h);
        assert!(h < i);
        assert!(i == j);
    }

    #[test]
    fn test_math() {
        let g = DateTime::<Gregorian, Tt>::new(1996, 3, 2, 0, 0, 0, 50).unwrap();
        let week_less_150ns = Duration::new(86400 * 7, 150);
        let earlier = g - week_less_150ns;
        assert_eq!(earlier.year(), 1996);
        assert_eq!(earlier.month(), 2);
        assert_eq!(earlier.day(), 23);
        assert_eq!(earlier.hour(), 23);
        assert_eq!(earlier.minute(), 59);
        assert_eq!(earlier.second(), 59);
        assert_eq!(earlier.attosecond(), ATTOS_PER_SEC_U64 - 100);

        let g1 = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 0, 0, 0, 0).unwrap();
        let g2 = DateTime::<Gregorian, Tt>::new(2001, 2, 2, 1, 3, 5, 11).unwrap();
        let diff = g2 - g1;
        assert_eq!(
            diff.seconds_part(),
            366 * 86400 + 31 * 86400 + 1 * 86400 + 1 * 3600 + 3 * 60 + 5
        );
        assert_eq!(diff.attos_part(), 11);
    }

    #[test]
    fn test_print_extremes() {
        let min = DateTime::<Gregorian, Tt>::new(std::i32::MIN, 1, 1, 0, 0, 0, 0).unwrap();
        println!("Min gregorian: {}", min);
        let max = DateTime::<Gregorian, Tt>::new(
            std::i32::MAX,
            12,
            31,
            23,
            59,
            59,
            999_999_999_999_999_999,
        )
        .unwrap();
        println!("Max gregorian: {}", max);
    }

    #[test]
    fn test_bc_day_numbers() {
        let mar1 = DateTime::<Gregorian, Tt>::new(0, 3, 1, 0, 0, 0, 0).unwrap();
        let feb29 = DateTime::<Gregorian, Tt>::new(0, 2, 29, 0, 0, 0, 0).unwrap();
        let feb28 = DateTime::<Gregorian, Tt>::new(0, 2, 28, 0, 0, 0, 0).unwrap();
        assert_eq!(mar1.day_number(), -306);
        assert_eq!(feb29.day_number(), -307);
        assert_eq!(feb28.day_number(), -308);

        let mar1x = DateTime::<Gregorian, Tt>::from_day_number(-306).unwrap();
        let feb29x = DateTime::<Gregorian, Tt>::from_day_number(-307).unwrap();
        let feb28x = DateTime::<Gregorian, Tt>::from_day_number(-308).unwrap();
        assert_eq!(mar1, mar1x);
        assert_eq!(feb29, feb29x);
        assert_eq!(feb28, feb28x);
    }

    #[test]
    fn test_convert_calendar() {
        let j = DateTime::<Julian, Tt>::new(1582, 10, 5, 0, 0, 0, 0).unwrap();
        let g = DateTime::<Gregorian, Tt>::new(1582, 10, 15, 0, 0, 0, 0).unwrap();
        let j2: DateTime<Julian, Tt> = TryFrom::try_from(g).unwrap();
        assert_eq!(j, j2);
        let g2: DateTime<Gregorian, Tt> = TryFrom::try_from(j).unwrap();
        assert_eq!(g, g2);

        let j = DateTime::<Julian, Tt>::new(1582, 10, 4, 0, 0, 0, 0).unwrap();
        let g = DateTime::<Gregorian, Tt>::new(1582, 10, 14, 0, 0, 0, 0).unwrap();
        let j2: DateTime<Julian, Tt> = TryFrom::try_from(g).unwrap();
        assert_eq!(j, j2);
        let g2: DateTime<Gregorian, Tt> = TryFrom::try_from(j).unwrap();
        assert_eq!(g, g2);

        let j = DateTime::<Julian, Tt>::new(-4713, 1, 1, 0, 0, 0, 0).unwrap();
        let g = DateTime::<Gregorian, Tt>::new(-4714, 11, 24, 0, 0, 0, 0).unwrap();
        let j2: DateTime<Julian, Tt> = TryFrom::try_from(g).unwrap();
        assert_eq!(j, j2);
        let g2: DateTime<Gregorian, Tt> = TryFrom::try_from(j).unwrap();
        assert_eq!(g, g2);

        let j = DateTime::<Julian, Tt>::new(1, 1, 3, 0, 0, 0, 0).unwrap();
        let g = DateTime::<Gregorian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap();
        let j2: DateTime<Julian, Tt> = TryFrom::try_from(g).unwrap();
        assert_eq!(j, j2);
        let g2: DateTime<Gregorian, Tt> = TryFrom::try_from(j).unwrap();
        assert_eq!(g, g2);

        let j = DateTime::<Julian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap();
        let g = DateTime::<Gregorian, Tt>::new(0, 12, 30, 0, 0, 0, 0).unwrap();
        let j2: DateTime<Julian, Tt> = TryFrom::try_from(g).unwrap();
        assert_eq!(j, j2);
        let g2: DateTime<Gregorian, Tt> = TryFrom::try_from(j).unwrap();
        assert_eq!(g, g2);
    }

    #[test]
    fn test_epoch_duration() {
        let g = DateTime::<Gregorian, Tt>::new(1582, 10, 14, 0, 0, 0, 0).unwrap();
        let h = DateTime::<Gregorian, Tt>::from_duration_from_epoch(g.duration_from_epoch());
        assert_eq!(g, h);

        let g = DateTime::<Julian, Tt>::new(1582, 10, 14, 11, 0, 5, 130).unwrap();
        let h = DateTime::<Julian, Tt>::from_duration_from_epoch(g.duration_from_epoch());
        assert_eq!(g, h);
    }

    #[test]
    fn test_weekday() {
        let g = DateTime::<Gregorian, Tt>::new(2026, 2, 1, 12, 0, 0, 0).unwrap();
        assert_eq!(g.weekday(), 7);
        let j = DateTime::<Julian, Tt>::new(2026, 1, 19, 12, 0, 0, 0).unwrap();
        assert_eq!(j.weekday(), 7);
    }

    #[test]
    fn test_datetime_instant_conversions_in_tt() {
        let p1: Instant = Epoch::J1991_25.as_instant();
        let d: DateTime<Gregorian, Tt> = p1.into();
        let p2: Instant = d.into();
        assert_eq!(p1, p2);

        let d1 = DateTime::<Gregorian, Tt>::new(1993, 6, 30, 0, 0, 27, 0).unwrap();
        let p: Instant = d1.into();
        let d2: DateTime<Gregorian, Tt> = p.into();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_datetime_instant_conversions_without_leapseconds() {
        let p1: Instant = Epoch::J1991_25.as_instant();
        let d: DateTime<Gregorian, Tai> = p1.into();
        let p2: Instant = d.into();
        assert_eq!(p1, p2);

        let d1 = DateTime::<Gregorian, Tai>::new(1993, 6, 30, 0, 0, 27, 0).unwrap();
        let p: Instant = d1.into();
        let d2: DateTime<Gregorian, Tai> = p.into();
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_datetime_instant_conversions_with_leapseconds() {
        let leap_instant = Instant::from_ntp_date(2429913600, 0);
        assert_eq!(leap_instant, Epoch::Y1977.as_instant()); // FAILLING, but why?
        let date: DateTime<Gregorian, Utc> = leap_instant.into();
        let expected_date = DateTime::<Gregorian, Utc>::new(1977, 1, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(date, expected_date);

        let plus_three = leap_instant + Duration::new(3, 0);
        let date: DateTime<Gregorian, Utc> = plus_three.into();
        let expected_date = DateTime::<Gregorian, Utc>::new(1977, 1, 1, 0, 0, 3, 0).unwrap();
        assert_eq!(date, expected_date);

        let minus_three = leap_instant - Duration::new(3, 0);
        let date: DateTime<Gregorian, Utc> = minus_three.into();
        let expected_date = DateTime::<Gregorian, Utc>::new(1976, 12, 31, 23, 59, 58, 0).unwrap();
        assert_eq!(date, expected_date);

        let minus_half = leap_instant - Duration::new(0, ATTOS_PER_SEC_I64 / 2);
        let date: DateTime<Gregorian, Utc> = minus_half.into();
        let expected_date =
            DateTime::<Gregorian, Utc>::new(1976, 12, 31, 23, 59, 60, ATTOS_PER_SEC_U64 / 2)
                .unwrap();
        assert_eq!(date, expected_date);
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

    #[test]
    fn test_instant_datetime_utc_range() {
        // Test UTC in the vacinity of a leap second (1 January 1999)
        let leap_instant: Instant = From::from(
            DateTime::<Gregorian, Tt>::new(1999, 1, 1, 0, 0, 0, 0).unwrap()
                - Duration::new(32 + 32, 184_000_000_000_000_000),
        );
        for s in -100..100 {
            println!("s={}", s);
            let a = leap_instant + Duration::new(s, 0);
            let dt: DateTime<Gregorian, Utc> = a.into();
            let b: Instant = dt.into();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_tcg() {
        // Because we have to use floating point, we
        let d = DateTime::<Gregorian, Tt>::new(2020, 1, 1, 0, 0, 0, 0).unwrap();
        let d2: DateTime<Gregorian, Tcg> = d.into();
        let d3: DateTime<Gregorian, Tt> = d2.into();
        let diff = d3 - d;
        assert_eq!(diff.secs, 0);
        assert!(diff.attos.abs() < 1_000_000_000_000);

        let d = DateTime::<Gregorian, Tai>::new(2020, 1, 1, 0, 0, 0, 0).unwrap();
        let d2: DateTime<Gregorian, Tcg> = d.into();
        let d3: DateTime<Gregorian, Tai> = d2.into();
        let diff = d3 - d;
        assert_eq!(diff.secs, 0);
        assert!(diff.attos.abs() < 1_000_000_000_000);

        let d = DateTime::<Gregorian, Utc>::new(2020, 1, 1, 0, 0, 0, 0).unwrap();
        let d2: DateTime<Gregorian, Tcg> = d.into();
        let d3: DateTime<Gregorian, Utc> = d2.into();
        let diff = d3 - d;
        assert_eq!(diff.secs, 0);
        assert!(diff.attos.abs() < 1_000_000_000_000);
    }
}
