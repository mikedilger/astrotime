//! astrotime
//!
//! Time related types for scientific and astronomical usage.

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
pub use standard::{Standard, Tai, Tt, Utc};

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
