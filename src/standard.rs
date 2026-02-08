use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::duration::Duration;

/// A standard of time
pub trait Standard: Debug + Sized + Clone {
    /// Short capital-letter abbreviation for the time standard (usually 2 or 3 letters)
    fn abbrev() -> &'static str;

    /// Offset from Tt  (This + offset = TT)
    ///
    /// This function is not meant to be called from outside the library.
    /// But if you wish to create a new Standard that implements this trait, you'll need
    /// access to this. This should not adjust for leap seconds.
    fn tt_offset() -> Duration;
}

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

    fn tt_offset() -> Duration {
        Duration::new(0, 0)
    }
}

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

    fn tt_offset() -> Duration {
        Duration::new(32, 184_000_000_000_000_000)
    }
}

/// Universal Coordinated Time
///
/// This is civil time as usually reported.  It is discontinuous, having leap
/// seconds inserted from time to time based on the Earth's rotation.
///
/// This type is proleptic. For all dates prior to 1 Jan 1972, we presume
/// 9 seconds have elapsed (like leaps) offsetting from TAI permanently by 9
/// seconds, even though that's not what happened at the time (what happened at
/// the time was that UTC wasn't syncronized to TAI by integer leap seconds,
/// but rather to other time sources and contained fractional leap seconds which
/// are hard to reconstruct or even get a list of).  For all dates prior to
/// 1 January 1960, UTC didn't exist, but we pretend it did, offset 9 seconds
/// from TAI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Utc;
impl Standard for Utc {
    fn abbrev() -> &'static str {
        "UTC"
    }

    fn tt_offset() -> Duration {
        Duration::new(41, 184_000_000_000_000_000)
    }
}
