//! astrotime
//!
//! Time related types for scientific and astronomical usage.

#![warn(
     clippy::all,
     clippy::pedantic,
     clippy::nursery,
     clippy::cargo,
)]

#[macro_use] extern crate log;

mod calendar;
pub use calendar::{Calendar, Julian, Gregorian};

mod date_time;
pub use date_time::DateTime;

mod duration;
pub use duration::Duration;

mod epoch;
pub use epoch::Epoch;

mod error;
pub use error::Error;

mod instant;
pub use instant::Instant;

mod standard;
pub use standard::{Standard, Tt, Tai, Utc};


// When running tests, we setup the logger
#[cfg(test)]
static INIT: std::sync::Once = std::sync::Once::new();
#[cfg(test)]
fn setup_logging() {
    INIT.call_once(|| {
        pretty_env_logger::init();
    });
}

// Division and modulus
macro_rules! define_divmod {
    ($t:ty, $name:ident) => {
        #[inline]
        const fn $name(a: $t, b: $t) -> ($t, $t)
        {
            (
                a.div_euclid(b),
                a.rem_euclid(b)
            )
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
    crate::setup_logging();

    let (div,modulo) = divmod_i64(47, 10);
    assert_eq!(div, 4);
    assert_eq!(modulo, 7);

    let (div,modulo) = divmod_i64(-47, 10);
    assert_eq!(div, -5);
    assert_eq!(modulo, 3);
}
