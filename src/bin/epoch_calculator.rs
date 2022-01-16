
use astrotime::*;

fn main() {
    // let zero_point = DateTime::<Julian, Tt>::new_bc(4713, 1, 1, 12, 0, 0, 0).unwrap();
    let zero_point_greg = DateTime::<Gregorian, Tt>::new(1977, 1, 1, 0, 0, 32, 184_000_000_000_000_000).unwrap();
    let zero_point: DateTime::<Julian, Tt> = TryFrom::try_from(zero_point_greg).unwrap();

    // ----------------------------------------------------
    let julian_epoch = {
        // JD 0
        let e = DateTime::<Julian, Tt>::new_bc(4713, 1, 1, 12, 0, 0, 0).unwrap();
        e - zero_point
    };
    println!("Julian Epoch: {:?}", julian_epoch);

    // ----------------------------------------------------
    let julian_cal = {
        // JD 1721423.5 (unverified, based on 2 day offset from Gregorian)
        let e = DateTime::<Julian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap();
        e - zero_point
    };
    println!("Julian Cal: {:?}", julian_cal);

    // ----------------------------------------------------
    let gregorian_cal = {
        // JD 1721425.5 (verified from https://en.wikipedia.org/wiki/Julian_day)
        let e = DateTime::<Gregorian, Tt>::new(1, 1, 1, 0, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("Gregorian Cal: {:?}", gregorian_cal);

    // ----------------------------------------------------
    let j1900 = {
        // JD 2415020.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
        let e = DateTime::<Gregorian, Tt>::new(1899, 12, 31, 12, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("J1900.0: {:?}", j1900);

    // ----------------------------------------------------
    // FIXME UTC
    let unix = {
        // JD 2440587.5 (APPROX - modified because of UTC)
        let e = DateTime::<Gregorian, Tt>::new(1970, 1, 1, 0, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("unix: {:?}", unix);

    // ----------------------------------------------------
    let ts = {
        let e = DateTime::<Gregorian, Tt>::new(1977, 1, 1, 0, 0, 32, 184_000_000_000_000_000).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("ts: {:?}", ts);

    // ----------------------------------------------------
    let j1991_25 = {
        // JD 2448349.0625 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
        let e = DateTime::<Gregorian, Tt>::new(1991, 4, 2, 13, 30, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("J1991.25: {:?}", j1991_25);

    // ----------------------------------------------------
    // FIXME UTC
    let y2k = {
        let e = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 0, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("Y2K: {:?}", y2k);

    // ----------------------------------------------------
    let j2000 = {
        // JD 2451545.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
        let e = DateTime::<Gregorian, Tt>::new(2000, 1, 1, 12, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("J2000.0: {:?}", j2000);

    // ----------------------------------------------------
    let j2100 = {
        // JD 2488070.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
        let e = DateTime::<Gregorian, Tt>::new(2100, 1, 1, 12, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("J2100.0: {:?}", j2100);

    // ----------------------------------------------------
    let j2200 = {
        // JD 2524595.0 (verified at https://www.astronomyclub.xyz/celestial-sphere-2/epochs-for-coordinate-systems.html
        let e = DateTime::<Gregorian, Tt>::new(2200, 1, 2, 12, 0, 0, 0).unwrap();
        let e: DateTime<Julian, Tt> = TryFrom::try_from(e).unwrap();
        e - zero_point
    };
    println!("J2200.0: {:?}", j2200);

    // ----------------------------------------------------

}

/*
Julian Epoch: Duration { secs: -211087684832, attos: -184000000000000000 }
Julian Cal: Duration { secs: -62356694432, attos: -184000000000000000 }
Gregorian Cal: Duration { secs: -62356521632, attos: -184000000000000000 }
J1900.0: Duration { secs: -2429956832, attos: -184000000000000000 }
unix: Duration { secs: -220924832, attos: -184000000000000000 }
ts: Duration { secs: 0, attos: 0 }
J1991.25: Duration { secs: 449674167, attos: 816000000000000000 }
Y2K: Duration { secs: 725759967, attos: 816000000000000000 }
J2000.0: Duration { secs: 725803167, attos: 816000000000000000 }
J2100.0: Duration { secs: 3881563167, attos: 816000000000000000 }
J2200.0: Duration { secs: 7037323167, attos: 816000000000000000 }
*/
