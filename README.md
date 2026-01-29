# astrotime

Astrotime is a rust library for dealing with time for scientific and
astronomical purposes on the surface of the Earth. It is not sufficient
for satellites or space travellers (but PRs are welcome).

This library is lightweight and high performance.


## Features

The following features are currently available:

* Handles times covering the entire duration of the universe.
* Handles times to attosecond precision.
* Calendar representations of time (day, month, year, hour, minute, second)
  including the well known month length and leap year oddities.
* Using and converting between calendar standards: Gregorian, Julian
* Julian Day support
* Converts between time standards (UTC, TT, TAI), including leap second
  handling (WARNING: the leap second list is compiled in and will require
  code updates to refresh it if more leap seconds are added).
* Supplies precise instants for well known Epochs such as J1900.0, J2000.0,
  TcbTcgEphemeris, the Unix epoch, Y2K, etc.
* Optional serde serialization (enable feature 'serde')


### Currently Excluded

We do not deal with these civil time issues:

* Timezones
* Daylight savings time
* AM/PM

UTC is the closest we get to civil time, and we do so primarily because
Internet time standards are based on it, not because it is good.


### Possible Improvements

The following are NOT available currently, but we are not against PRs
that implement any of these:

* The ability to update leapseconds from the IETF source online.
* Handling of civil time issues such as
    * Time zones
    * Daylight Savings Time
    * AM/PM versus 24-hour time
* Provides (possibly lossy) conversions for types in the 'chrono' and
  'time' crates, including rust std SystemTime.
* Add GPS time and LORAN-C
* Add Sidereal time
* impl ApproxEq from the float_cmp trait for Duration, Instant, and DateTime


## Types

### Duration

`Duration` represents an interval of time.

`Duration`s can handle lengths of time about 40 times as long as the age of the
universe, and have attosecond (10^-18) precision.

Negative values are supported.

#### Why attoseconds? Why not just nanoseconds?

I don't have a personal use for such precision, but the data types were 96 bits with
nanoseconds, and since computers these days tend to be 64-bit, it seemed that half a
word was being wasted. So I extended the fractional seconds part to handle attoseconds.
The biggest downside is typing all 18 zeroes.

### DateTime and Instant

There are two types that represent an instant or moment in time.

The first type is a `DateTime<Calendar, Standard>`, which allows easy
creation and manipulation of dates using Calendars such as `Gregorian` or
`Julian`, and which may flexibly represent different time standards (`Utc`,
`Tai`, or `Tt`).  This type internally stores the year, month, day, hour,
minute, second and attosecond in a packed format. It is 128 bits in size.

A `DateTime` can have a minimum and maximum year within an `i32` type. Thus
it does not span the same duration of time that a `Duration` does.

The second type is an `Instant` with an opaque implementation, which is also
128 bits in size. `Instant`s can be subtracted to get a `Duration`, and a
`Duration` can be added to or subtracted from an `Instant` to get a
different `Instant`.  `Instants` can span the full duration of time that
`Duration` supports.  `Instant`s can convert to and from `DateTime`s
of varying `Calendar` and `Stanadard` parameters providing automatic
conversion between said `Calendar`s and `Standard`s.  `Instant`s can
also be converted to and from Julian Days.

### Epochs

Well known points in time are provided such as the start of the `JulianPeriod`, or the start
of the `JulianCalendar`, `J1900_0`, `Unix` (the start of UNIXTIME), `Y2k`, etc.

## Curiousities about Time

UnixTime is discontinuous and has overlapping seconds. Some values of unixtime refer to
two different points in time exactly one second apart. UTC differentiate these two seconds
with a :59 and a :60 but unixtime has no such differentiator.

Spreadsheets (starting with MicroSoft Excel?) store dates as a floating point count of days
since December 30, 1899, at midnight. This epoch is 18 hours before J1900.0.

Astronomers use the TT (terrestrial time) standard, because leap second shifting, while
it accounts for the rotation of the earth, confuses all other astronomical timings. For
example, the position of the Earth in it's orbit around the Sun has nothing to do with
whether or not Earth's rotation on it's axis has changed. TT differs from UTC by a little
over a minute.

Astronomers use Julian Days which start and end at noon GMT so that the entire night lies
within a single Julian day (astronomers work at night).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
