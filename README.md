# astrotime

Time related types (and conversions) for scientific and astronomical usage.

This library is lightweight and high performance.

## Features

The following features are currently available:

* Handles times covering the entire duration of the universe.
* Handles times to attosecond precision
* Allows you to work with Calendar dates and times, either Gregorian or Julian
* Allows you to work with Julian Day values
* Converts between time standards (e.g. UTC, TAI, TT, TCG, TCB)
    * Einsteinean reference frames are accounted for with the `Tcg` (TCG is for
      geocentric satellites) and `Tcb` (TCB is for barycentric solar system
      objects) time standards.
    * Leap seconds are accounted for in conversions to and from UTC (but the list of leap seconds is
      currently compiled in and may go out of date).
* Supplies precise instants for well known Epochs such as 1900.0, J1900.0, the Unixtime epoch,
  Y2K, etc.
* Optional serde serialization (enable feature 'serde')

## Goals

The following are NOT available currently:

* Ability to update leapseconds from the IETF source online.
* Handling of civil time issues such as
    * Time zones
    * Daylight Savings Time
    * AM/PM versus 24-hour time
* Provides (possibly lossy) conversions for types in the 'chrono' and 'time' crates, including
  rust std SystemTime.
* Add GPS time and LORAN-C (easy)
* Add Sidereal time
* impl ApproxEq from the float_cmp trait for Duration, Instant, and DateTime

## One Duration type

`Duration` represents an interval of time.

`Duration`s can handle lengths of time about 40 times as long as the age of the
universe, and have attosecond (10^-18) precision.

Negative values are supported.

### DateTime and Instant

There are two types that represent an instant or moment in time.

The first type is a `DateTime<Calendar, Standard>`, which allows easy creation and manipulation
of dates using Calendars such as `Gregorian` or `Julian`, and which may flexibly represent
different time standards (such as `Utc`, `Tai`, `Tt`, `Tcg` and `Tcb`).  This type intnerally
stores the year, month, day, hour, minute, second and attosecond in a packed format. It is 128
bits in size. A `DateTime` can have a minimum and maximum year within an i32. Thus it does not
span the same duration of time that a `Duration` does.

The second type is an `Instant` with an opaque implementation, also 128 bits in size.
`Instant`s can be subtracted to get a `Duration`, and a `Duration` can be added to or
subtracted from an `Instant` to get a different `Instant`.  `Instants` can span the full
duration of time that `Duration` supports.  `Instant`s can convert to and from `DateTime`s
of varying `Calendar` and `Stanadard` parameters providing automatic conversion between said
`Calendar`s and `Standard`s.  `Instant`s  can also be converted to and from Julian Days.

### Epochs

Well known points in time are provided such as the start of the `JulianPeriod`, or the start
of the `JulianCalendar`, `J1900_0`, `Unix` (the start of UNIXTIME), `Y2k`, etc.

## FAQ

### Why attoseconds? Why not just nanoseconds?

I don't have a personal use for such precision, but the data types were 96 bits with
nanoseconds, and since computers these days tend to be 64-bit, it seemed that half a
word was being wasted. So I extended the fractional seconds part to handle attoseconds.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
