
use std::fmt::Debug;

/// A standard of time
pub trait Standard: Debug + Sized + Clone {
    /// Short capital-letter abbreviation for the time standard (usually 2 or 3 letters)
    fn abbrev() -> &'static str;
}

/// Whether a Standard is Continuous or not
pub trait Continuous { }

/// Terrestrial Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// See [Wikipedia](https://en.wikipedia.org/wiki/Terrestrial_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tt;
impl Standard for Tt {
    fn abbrev() -> &'static str {
        "TT"
    }
}
impl Continuous for Tt { }

/// Geocentric Coordinate Time
///
/// This is a continuous time standard for satellites that orbit the Earth
/// See [Wikipedia](https://en.wikipedia.org/wiki/Geocentric_Coordinate_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tcg;
impl Standard for Tcg {
    fn abbrev() -> &'static str {
        "TCG"
    }
}
impl Continuous for Tcg { }

/// Barycentric Coordinate Time
///
/// This is a continuous time standard for satellites that orbit the Sun
/// See [Wikipedia](https://en.wikipedia.org/wiki/Barycentric_Coordinate_Time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tcb;
impl Standard for Tcb {
    fn abbrev() -> &'static str {
        "TCB"
    }
}
impl Continuous for Tcb { }

/// International Atomic Time
///
/// This is a continuous time standard for the surface of the Earth (Earth's geoid)
/// realized via atomic clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tai;
impl Standard for Tai {
    fn abbrev() -> &'static str {
        "TAI"
    }
}
impl Continuous for Tai { }

/// Universal Coordinated Time
///
/// This is civil time as usually reported.  It is discontinuous, having leap
/// seconds inserted from time to time based on the Earth's rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utc;
impl Standard for Utc {
    fn abbrev() -> &'static str {
        "UTC"
    }
}
