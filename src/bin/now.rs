use astrotime::{DateTime, Gregorian, Instant, Tai, Tt, Utc};

fn main() {
    let now = std::time::SystemTime::now();
    let now: Instant = TryFrom::try_from(now).unwrap();
    let now_gregorian_utc: DateTime<Gregorian, Utc> = From::from(now);
    println!("{}", now_gregorian_utc);
    let now_gregorian_tai: DateTime<Gregorian, Tai> = From::from(now);
    println!("{}", now_gregorian_tai);
    let now_gregorian_tt: DateTime<Gregorian, Tt> = From::from(now);
    println!("{}", now_gregorian_tt);
}
