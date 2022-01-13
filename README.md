# astrotime

Time related types for scientific and astronomical usage.

This library is lightweight and high performance. The library has few dependencies.

## Goals

* Spanning the entire duration of the universe (from about 20x further back than the big bang to about 20x the current age of the universe from now).
* Providing attosecond (10^-18 seconds) precision.
* Accounts for Einsteinean reference frames (Earth's Geoid, Geocentric, and Barycentric)
* Converts between different time standards (UTC, TAI, TT, TGC, TCB)
* Presents and parses data in different time formats (Gregorian, Unixtime, Julian variations)
* Handles well known Epoch instants (1900.0, J2000, Unix Epoch)
* Handles leap seconds where needed for conversion, and can fetch the current list of leapseconds when required and enabled
* Handles civil time issues such as
    * Time zones
    * Daylight Savings Time
    * AM/PM versus 24-hour time
* Provides (possibly lossy) conversions for types in the 'chrono' and 'time' crates.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
