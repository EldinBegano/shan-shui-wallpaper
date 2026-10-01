use crate::js::to_int32;
use crate::prng::Prng;

const YWRAPB: i32 = 4;
const YWRAP: f64 = (1 << YWRAPB) as f64;
const ZWRAPB: i32 = 8;
const ZWRAP: f64 = (1 << ZWRAPB) as f64;
const SIZE: i32 = 4095;
const OCTAVES: usize = 4;
const FALLOFF: f64 = 0.5;

const SC_BITS: u32 = 12;

pub struct Noise {
    perlin: Option<Box<[f64; SIZE as usize + 1]>>,
    sc_key: Box<[f64; 1 << SC_BITS]>,
    sc_val: Box<[f64; 1 << SC_BITS]>,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            perlin: None,
            sc_key: Box::new([f64::NAN; 1 << SC_BITS]),
            sc_val: Box::new([0.0; 1 << SC_BITS]),
        }
    }

    #[inline]
    fn scaled_cosine(&mut self, i: f64) -> f64 {
        let bits = i.to_bits();
        let h = ((bits ^ (bits >> 32)) as u32).wrapping_mul(0x9e3779b1) >> (32 - SC_BITS);
        let h = h as usize;
        if self.sc_key[h] == i {
            return self.sc_val[h];
        }
        let v = 0.5 * (1.0 - (i * std::f64::consts::PI).cos());
        self.sc_key[h] = i;
        self.sc_val[h] = v;
        v
    }

    #[inline]
    fn at(p: &[f64; SIZE as usize + 1], of: f64) -> f64 {
        p[(to_int32(of) & SIZE) as usize]
    }

    pub fn noise(&mut self, rng: &mut Prng, x: f64, y: f64, z: f64) -> f64 {
        let y = if y.is_nan() { 0.0 } else { y };
        let z = if z.is_nan() { 0.0 } else { z };
        if self.perlin.is_none() {
            let mut p = Box::new([0.0; SIZE as usize + 1]);
            for v in p.iter_mut() {
                *v = rng.next();
            }
            self.perlin = Some(p);
        }
        let x = if x < 0.0 { -x } else { x };
        let y = if y < 0.0 { -y } else { y };
        let z = if z < 0.0 { -z } else { z };
        let mut xi = x.floor();
        let mut yi = y.floor();
        let mut zi = z.floor();
        let mut xf = x - xi;
        let mut yf = y - yi;
        let mut zf = z - zi;
        let mut r = 0.0;
        let mut ampl = 0.5;
        for _ in 0..OCTAVES {
            let mut of = xi
                + to_int32(yi).wrapping_shl(YWRAPB as u32) as f64
                + to_int32(zi).wrapping_shl(ZWRAPB as u32) as f64;
            let rxf = self.scaled_cosine(xf);
            let ryf = if yf != 0.0 { self.scaled_cosine(yf) } else { 0.0 };
            let rzf = if zf != 0.0 { self.scaled_cosine(zf) } else { 0.0 };
            let p = self.perlin.as_ref().unwrap();
            let mut n1 = Noise::at(p, of);
            n1 += rxf * (Noise::at(p, of + 1.0) - n1);
            if yf != 0.0 {
                let mut n2 = Noise::at(p, of + YWRAP);
                n2 += rxf * (Noise::at(p, of + YWRAP + 1.0) - n2);
                n1 += ryf * (n2 - n1);
            }
            if zf != 0.0 {
                of += ZWRAP;
                let mut n2 = Noise::at(p, of);
                n2 += rxf * (Noise::at(p, of + 1.0) - n2);
                if yf != 0.0 {
                    let mut n3 = Noise::at(p, of + YWRAP);
                    n3 += rxf * (Noise::at(p, of + YWRAP + 1.0) - n3);
                    n2 += ryf * (n3 - n2);
                }
                n1 += rzf * (n2 - n1);
            }
            r += n1 * ampl;
            ampl *= FALLOFF;
            xi = to_int32(xi).wrapping_shl(1) as f64;
            xf *= 2.0;
            yi = to_int32(yi).wrapping_shl(1) as f64;
            yf *= 2.0;
            zi = to_int32(zi).wrapping_shl(1) as f64;
            zf *= 2.0;
            if xf >= 1.0 {
                xi += 1.0;
                xf -= 1.0;
            }
            if yf >= 1.0 {
                yi += 1.0;
                yf -= 1.0;
            }
            if zf >= 1.0 {
                zi += 1.0;
                zf -= 1.0;
            }
        }
        r
    }
}
