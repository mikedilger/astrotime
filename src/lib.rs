//! astrotime
//!
//! Astrotime is a rust library for dealing with time for scientific and
//! astronomical purposes on the surface of the Earth, or for satellites
//! of the Earth. It is not sufficient for space travel or for solar system
//! objects.
//!
//! Here is an example of getting the current time:
//!
//! ```rust
//! # use astrotime::*;
//! let now = std::time::SystemTime::now();
//! let now: Instant = TryFrom::try_from(now).unwrap();
//! let date: DateTime<Gregorian, Utc> = now.into();
//! println!("{}", date);
//! ```
//!
//! How many seconds have elasped since the UNIX epoch, until New Years
//! 2026?  This will yield `1767225628` seconds.  Unixtime for that moment
//! was `1767225600` because it ignores the leap seconds that also passed.
//!
//! ```rust
//! # use astrotime::*;
//! let now: DateTime<Gregorian, Utc> = DateTime::new(2026, 1, 1, 0, 0, 0, 0).unwrap();
//! let n: Instant = now.into();
//! let d: Duration = n - Epoch::Unix.as_instant();
//! println!("{} seconds", d.seconds_part());
//! ```


#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod error;
pub use error::Error;

pub const ATTOS_PER_SEC_I64: i64 = 1_000_000_000_000_000_000;
pub const ATTOS_PER_SEC_U64: u64 = 1_000_000_000_000_000_000;
pub const ATTOS_PER_SEC_F64: f64 = 1_000_000_000_000_000_000.;

mod duration;
pub use duration::Duration;

mod instant;
pub use instant::Instant;

mod epoch;
pub use epoch::Epoch;

mod calendar;
pub use calendar::{Calendar, Gregorian, Julian};

mod standard;
pub use standard::{Standard, Tai, Tcg, Tt, Utc};

mod leaps;
pub use leaps::{leap_instants, leap_seconds_elapsed_at};

mod date_time;
pub use date_time::DateTime;

// Division and modulus
macro_rules! define_divmod {
    ($t:ty, $name:ident) => {
        #[inline]
        const fn $name(a: $t, b: $t) -> ($t, $t) {
            (a.div_euclid(b), a.rem_euclid(b))
            /*
            assert!(b>0); // i don't know what it means to modulo a negative.

            if a<0 {
                ( ((a+1)/b)-1 , ((a%b)+b)%b )
            } else {
                ( a/b         , a%b         )
            }
             */
        }
    };
}

define_divmod!(i64, divmod_i64);

#[test]
fn test_divmod() {
    let (div, modulo) = divmod_i64(47, 10);
    assert_eq!(div, 4);
    assert_eq!(modulo, 7);

    let (div, modulo) = divmod_i64(-47, 10);
    assert_eq!(div, -5);
    assert_eq!(modulo, 3);
}
