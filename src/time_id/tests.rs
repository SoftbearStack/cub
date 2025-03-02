// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(test)]
mod time_tests {
    use crate::time_id::{UnixMillis, UnixTime};

    // (oauth2 already has a Chrono dependency.)
    #[cfg(feature = "oauth")]
    #[test]
    fn chrono_tests() {
        println!("Testing chrono");
        let t1 = UnixMillis::now();
        println!("t1 = {t1}");
        let (year, month, day, hour, minute, second) = t1.ymdhms();
        match UnixMillis::from_ymdhms(year, month, day, hour, minute, second) {
            Ok(t2) => println!("t2 = {t2}"),
            Err(e) => println!("{e:?}"),
        }
    }

    #[tokio::test]
    async fn time_casts_01() {
        println!("Testing time casts (for 0, 1).");
        let a1 = 1i64;
        let b1 = -1i64;
        let c1 = 1u64;
        let d1 = 0u64;

        let t1 = UnixMillis::from(a1);
        let a2: i64 = t1.into();
        println!("{a1} => {a2}");

        let t2 = UnixMillis::from(b1);
        let b2: i64 = t2.into();
        println!("{b1} => {b2}");

        let t3 = UnixMillis::try_from(c1).unwrap();
        let c2: u64 = t3.try_into().unwrap();
        println!("{c1} => {c2}");

        let t4 = UnixMillis::try_from(d1).unwrap();
        let d2: u64 = t4.try_into().unwrap();
        println!("{d1} => {d2}");
    }

    #[test]
    fn time_cast_now() {
        println!("Testing time casts (for time now).");
        let t1 = UnixMillis::now();
        println!("t1 = {t1:?}");

        let u: u64 = t1.try_into().unwrap();
        println!("u = {u:?}");

        let t2 = UnixMillis::try_from(u).unwrap();
        println!("t2 = {t2:?}");

        let i: i64 = t1.into();
        println!("i = {i:?}");

        assert_eq!(t1, t2);

        let t3 = UnixMillis::from(i);
        println!("t3 = {t3:?}");

        assert_eq!(t1, t3);

        println!("Time test completed");
    }
}
