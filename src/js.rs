#[inline]
pub fn to_int32(x: f64) -> i32 {
    if x.abs() < 2147483648.0 {
        return x as i32;
    }
    if !x.is_finite() {
        return 0;
    }
    let m = x.trunc().rem_euclid(4294967296.0);
    m as u64 as u32 as i32
}

pub fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else {
        b
    }
}

pub fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}

pub fn fixed(x: f64, digits: i32) -> f64 {
    let s = 10f64.powi(digits);
    (x * s).round() / s
}

pub fn to_fixed_str(x: f64, digits: usize) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    let neg = x < 0.0;
    let exact = format!("{:.80}", x.abs());
    let (int_part, frac) = exact.split_once('.').unwrap();
    let mut digs: Vec<u8> = int_part.bytes().chain(frac.bytes().take(digits)).collect();
    let next = frac.as_bytes()[digits];
    if next >= b'5' {
        let mut i = digs.len();
        loop {
            if i == 0 {
                digs.insert(0, b'1');
                break;
            }
            i -= 1;
            if digs[i] == b'9' {
                digs[i] = b'0';
            } else {
                digs[i] += 1;
                break;
            }
        }
    }
    let n_int = digs.len() - digits;
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    s.push_str(std::str::from_utf8(&digs[..n_int]).unwrap());
    if digits > 0 {
        s.push('.');
        s.push_str(std::str::from_utf8(&digs[n_int..]).unwrap());
    }
    s
}

pub fn num_str(x: f64) -> String {
    if x.is_nan() {
        "NaN".into()
    } else if x == 0.0 {
        "0".into()
    } else {
        format!("{x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int32() {
        assert_eq!(to_int32(3.7), 3);
        assert_eq!(to_int32(-3.7), -3);
        assert_eq!(to_int32(4294967296.0 + 5.0), 5);
        assert_eq!(to_int32(2147483648.0), -2147483648);
        assert_eq!(to_int32(f64::NAN), 0);
    }

    #[test]
    fn to_fixed() {
        assert_eq!(to_fixed_str(1.25, 1), "1.3");
        assert_eq!(to_fixed_str(-1.25, 1), "-1.3");
        assert_eq!(to_fixed_str(0.05, 1), "0.1");
        assert_eq!(to_fixed_str(9.96, 1), "10.0");
        assert_eq!(to_fixed_str(-0.04, 1), "-0.0");
        assert_eq!(to_fixed_str(123.456, 3), "123.456");
        assert_eq!(to_fixed_str(0.0, 1), "0.0");
    }
}
