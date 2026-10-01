const P: f64 = 999979.0;
const Q: f64 = 999983.0;
const M: f64 = P * Q;

pub struct Prng {
    s: f64,
}

impl Prng {
    pub fn new(seed: &str) -> Prng {
        let mut rng = Prng { s: 1234.0 };
        rng.seed(seed);
        rng
    }

    fn hash(x: &str) -> f64 {
        let y = base64(&json_quote(x));
        let mut z = 0.0;
        for (i, c) in y.bytes().enumerate() {
            z += c as f64 * 128f64.powi(i as i32);
        }
        z
    }

    fn seed(&mut self, x: &str) {
        let h = Prng::hash(x);
        let mut y = 0.0;
        let mut z = 0.0;
        while y % P == 0.0 || y % Q == 0.0 || y == 0.0 || y == 1.0 {
            y = (h + z) % M;
            z += 1.0;
        }
        self.s = y;
        for _ in 0..10 {
            self.next();
        }
    }

    #[inline]
    pub fn next(&mut self) -> f64 {
        self.s = (self.s * self.s) % M;
        self.s / M
    }
}

fn json_quote(x: &str) -> Vec<u8> {
    let mut out = vec![b'"'];
    for ch in x.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            c if (c as u32) < 0x20 => out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes()),
            c if (c as u32) <= 0xff => out.push(c as u32 as u8),
            c => out.extend_from_slice(c.to_string().as_bytes()),
        }
    }
    out.push(b'"');
    out
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::new();
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64() {
        assert_eq!(base64(b"\"1234\""), "IjEyMzQi");
        assert_eq!(base64(b"ab"), "YWI=");
    }
}
