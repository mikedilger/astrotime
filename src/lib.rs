//! astrotime
//!
//! Time related types for scientific and astronomical usage.

#![warn(
     clippy::all,
     clippy::pedantic,
     clippy::nursery,
     clippy::cargo,
)]

#[allow(unused_imports)]
#[macro_use] extern crate log;

mod calendar;
pub use calendar::{Calendar, Julian, Gregorian};

mod duration;
pub use duration::Duration;

mod error;
pub use error::Error;

// When running tests, we setup the logger
#[cfg(test)]
static INIT: std::sync::Once = std::sync::Once::new();
#[cfg(test)]
fn setup_logging() {
    INIT.call_once(|| {
        pretty_env_logger::init();
    });
}
