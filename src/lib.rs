//! astrotime
//!
//! Astrotime is a Rust library for representing and manipulating time:
//! * with attosecond precision;
//! * across a span of time about 40 times as long as the age of the universe;
//! * with scientific and astronomical time standards such as TAI, TT, and TCG;
//! * with Gregorian and Julian calendar representations.
//!
//! Astrotime does not aim to support:
//! * space travel or time modeling for Solar System objects;
//! * civil-time concerns such as time zones and daylight saving time.
//!
//! It does handle leap seconds accurately enough to support UTC.
//!
//! Here is an example of getting the current dated time:
//!
//! ```rust
//! # use astrotime::*;
//! let now = DatedInstant::now().unwrap();
//! let date: DateTime<Gregorian, Utc> = now.into();
//! println!("{}", date);
//! ```
//!
//! How many seconds have elapsed between the Unix epoch and New Year's Day
//! 2026? This will yield `1767225627` seconds. Unix time for that moment was
//! `1767225600` because it ignores the leap seconds that also passed.
//!
//! ```rust
//! # use astrotime::*;
//! let now: DateTime<Gregorian, Utc> = DateTime::new(2026, 1, 1, 0, 0, 0, 0).unwrap();
//! let n: DatedInstant = now.into();
//! let d: Duration = n - Epoch::Unix.as_dated_instant();
//! println!("{} seconds", d.seconds_part());
//! ```
//!
//! A `DatedInstant` can also represent the same moment across different time
//! standards. This example starts with a TAI date and converts it to UTC via
//! `DatedInstant`:
//!
//! ```rust
//! # use astrotime::*;
//! let tai = DateTime::<Gregorian, Tai>::new(2026, 1, 1, 0, 0, 0, 0).unwrap();
//! let dated: DatedInstant = tai.into();
//! let utc: DateTime<Gregorian, Utc> = dated.into();
//! println!("{}", utc);
//! ```
//!
//! `MonotonicInstant` is for measuring elapsed time and deadlines. It has no
//! epoch and cannot be converted to a calendar date:
//!
//! ```rust
//! # use astrotime::*;
//! let start = MonotonicInstant::now();
//! let deadline = start + Duration::new(1, 0);
//! let until_deadline = deadline - start;
//! assert_eq!(until_deadline, Duration::new(1, 0));
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod error;
pub use error::Error;

pub const ATTOS_PER_SEC_I64: i64 = 1_000_000_000_000_000_000;
pub const ATTOS_PER_SEC_U64: u64 = 1_000_000_000_000_000_000;
pub const ATTOS_PER_SEC_F64: f64 = 1_000_000_000_000_000_000.;
pub const ATTOS_PER_SEC_I128: i128 = 1_000_000_000_000_000_000;

mod duration;
pub use duration::Duration;

mod dated_instant;
pub use dated_instant::DatedInstant;

mod monotonic_instant;
pub use monotonic_instant::MonotonicInstant;

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
