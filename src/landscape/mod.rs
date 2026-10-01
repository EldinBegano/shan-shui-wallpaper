pub mod arch;
pub mod man;
pub mod mount;
pub mod tree;
pub mod world;

use crate::js;
use crate::noise::Noise;
use crate::prng::Prng;
use std::f64::consts::{E, PI};

pub type Pt = [f64; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

pub type Col = Option<Rgba>;

pub const fn rgba(r: u8, g: u8, b: u8, a: f64) -> Col {
    Some(Rgba { r, g, b, a })
}
pub const WHITE: Col = rgba(255, 255, 255, 1.0);
pub const NONE: Col = None;
const CLEAR: Col = rgba(0, 0, 0, 0.0);

pub fn ink(a: f64) -> Col {
    rgba(100, 100, 100, a)
}

pub struct Label {
    pub text: &'static str,
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub ang: f64,
}

pub struct Shape {
    pub pts: Vec<Pt>,
    pub fil: Col,
    pub str: Col,
    pub wid: f64,
    pub label: Option<Box<Label>>,
}

pub type Canv = Vec<Shape>;

pub type Fun<'a> = &'a dyn Fn(f64) -> f64;

pub struct Ctx {
    rng: Prng,
    noise: Noise,
}

impl Ctx {
    pub fn new(seed: &str) -> Ctx {
        Ctx { rng: Prng::new(seed), noise: Noise::new() }
    }

    #[inline]
    pub fn rand(&mut self) -> f64 {
        self.rng.next()
    }

    #[inline]
    pub fn noise(&mut self, x: f64, y: f64, z: f64) -> f64 {
        self.noise.noise(&mut self.rng, x, y, z)
    }

    pub fn choice<T: Copy>(&mut self, arr: &[T]) -> T {
        arr[(arr.len() as f64 * self.rand()).floor() as usize]
    }

    pub fn norm_rand(&mut self, m: f64, mm: f64) -> f64 {
        let r = self.rand();
        mapval(r, 0.0, 1.0, m, mm)
    }

    pub fn wtrand(&mut self, f: impl Fn(f64) -> f64) -> f64 {
        loop {
            let x = self.rand();
            let y = self.rand();
            if y < f(x) {
                return x;
            }
        }
    }

    pub fn gaussian(&mut self) -> f64 {
        self.wtrand(|x| E.powf(-24.0 * sq(x - 0.5))) * 2.0 - 1.0
    }
}

#[inline]
pub fn sq(x: f64) -> f64 {
    x * x
}

pub fn distance(p0: Pt, p1: Pt) -> f64 {
    (sq(p0[0] - p1[0]) + sq(p0[1] - p1[1])).sqrt()
}

pub fn mapval(value: f64, istart: f64, istop: f64, ostart: f64, ostop: f64) -> f64 {
    ostart + (ostop - ostart) * (((value - istart) * 1.0) / (istop - istart))
}

pub fn loop_noise(ns: &mut [f64]) {
    let n = ns.len();
    let dif = ns[n - 1] - ns[0];
    let mut bds = [100.0, -100.0];
    for i in 0..n {
        ns[i] += (dif * (n - 1 - i) as f64) / (n - 1) as f64;
        if ns[i] < bds[0] {
            bds[0] = ns[i];
        }
        if ns[i] > bds[1] {
            bds[1] = ns[i];
        }
    }
    for v in ns.iter_mut() {
        *v = mapval(*v, bds[0], bds[1], 0.0, 1.0);
    }
}

pub fn mid_pt(plist: &[Pt]) -> Pt {
    let n = plist.len() as f64;
    let mut acc = [0.0, 0.0];
    for v in plist {
        acc = [v[0] / n + acc[0], v[1] / n + acc[1]];
    }
    acc
}

pub fn bezmh(p: &[Pt], w: f64) -> Vec<Pt> {
    let p: Vec<Pt> = if p.len() == 2 { vec![p[0], mid_pt(&[p[0], p[1]]), p[1]] } else { p.to_vec() };
    let mut plist = Vec::new();
    for j in 0..p.len() - 2 {
        let p0 = if j == 0 { p[j] } else { mid_pt(&[p[j], p[j + 1]]) };
        let p1 = p[j + 1];
        let p2 = if j == p.len() - 3 { p[j + 2] } else { mid_pt(&[p[j + 1], p[j + 2]]) };
        let pl = 20;
        for i in 0..pl + (j == p.len() - 3) as usize {
            let t = i as f64 / pl as f64;
            let u = sq(1.0 - t) + 2.0 * t * (1.0 - t) * w + t * t;
            plist.push([
                (sq(1.0 - t) * p0[0] + 2.0 * t * (1.0 - t) * p1[0] * w + t * t * p2[0]) / u,
                (sq(1.0 - t) * p0[1] + 2.0 * t * (1.0 - t) * p1[1] * w + t * t * p2[1]) / u,
            ]);
        }
    }
    plist
}

pub fn div(plist: &[Pt], reso: f64) -> Vec<Pt> {
    let tl = (plist.len() as f64 - 1.0) * reso;
    let mut rlist = Vec::new();
    let mut i = 0.0;
    while i < tl {
        let lastp = plist[(i / reso).floor() as usize];
        let nextp = plist[(i / reso).ceil() as usize];
        let p = (i % reso) / reso;
        rlist.push([lastp[0] * (1.0 - p) + nextp[0] * p, lastp[1] * (1.0 - p) + nextp[1] * p]);
        i += 1.0;
    }
    if let Some(last) = plist.last() {
        rlist.push(*last);
    }
    rlist
}

pub fn offset(plist: &[Pt], xof: f64, yof: f64) -> Vec<Pt> {
    plist.iter().map(|p| [p[0] + xof, p[1] + yof]).collect()
}

#[derive(Default, Clone, Copy)]
pub struct PolyArgs {
    pub xof: f64,
    pub yof: f64,
    pub fil: Option<Col>,
    pub str: Option<Col>,
    pub wid: f64,
}

fn unnan(v: f64) -> f64 {
    if v.is_nan() { -1000.0 } else { v }
}

pub fn poly(plist: &[Pt], a: PolyArgs) -> Shape {
    let fil = a.fil.unwrap_or(CLEAR);
    Shape {
        pts: plist.iter().map(|p| [unnan(p[0] + a.xof), unnan(p[1] + a.yof)]).collect(),
        fil,
        str: a.str.unwrap_or(fil),
        wid: a.wid,
        label: None,
    }
}

pub fn label(text: &'static str, x: f64, y: f64, size: f64, ang: f64, fil: Col) -> Shape {
    Shape {
        pts: Vec::new(),
        fil,
        str: NONE,
        wid: 0.0,
        label: Some(Box::new(Label { text, x, y, size, ang })),
    }
}

#[derive(Default, Clone, Copy)]
pub struct StrokeArgs<'a> {
    pub xof: f64,
    pub yof: f64,
    pub wid: Option<f64>,
    pub col: Option<Col>,
    pub noi: Option<f64>,
    pub out: Option<f64>,
    pub fun: Option<Fun<'a>>,
}

pub fn stroke(ctx: &mut Ctx, ptlist: &[Pt], a: StrokeArgs) -> Canv {
    let wid = a.wid.unwrap_or(2.0);
    let col = a.col.unwrap_or(rgba(200, 200, 200, 0.9));
    let noi = a.noi.unwrap_or(0.5);
    let out = a.out.unwrap_or(1.0);
    let default_fun = |x: f64| (x * PI).sin();
    let fun: Fun = a.fun.unwrap_or(&default_fun);

    if ptlist.is_empty() {
        return Vec::new();
    }
    let n = ptlist.len();
    let mut vtx0 = Vec::with_capacity(n);
    let mut vtx1 = Vec::with_capacity(n);
    let n0 = ctx.rand() * 10.0;
    for i in 1..n.saturating_sub(1) {
        let mut w = wid * fun(i as f64 / n as f64);
        w = w * (1.0 - noi) + w * noi * ctx.noise(i as f64 * 0.5, n0, 0.0);
        let p = ptlist[i];
        let a1 = (p[1] - ptlist[i - 1][1]).atan2(p[0] - ptlist[i - 1][0]);
        let a2 = (p[1] - ptlist[i + 1][1]).atan2(p[0] - ptlist[i + 1][0]);
        let mut a = (a1 + a2) / 2.0;
        if a < a2 {
            a += PI;
        }
        vtx0.push([p[0] + w * a.cos(), p[1] + w * a.sin()]);
        vtx1.push([p[0] - w * a.cos(), p[1] - w * a.sin()]);
    }
    let mut vtx = Vec::with_capacity(2 * n + 1);
    vtx.push(ptlist[0]);
    vtx.extend_from_slice(&vtx0);
    vtx.push(ptlist[n - 1]);
    vtx.extend(vtx1.iter().rev());
    vtx.push(ptlist[0]);
    vec![poly(&vtx, PolyArgs { xof: a.xof, yof: a.yof, fil: Some(col), str: Some(col), wid: out })]
}

#[derive(Default, Clone, Copy)]
pub struct BlobArgs<'a> {
    pub len: Option<f64>,
    pub wid: Option<f64>,
    pub ang: Option<f64>,
    pub col: Option<Col>,
    pub noi: Option<f64>,
    pub fun: Option<Fun<'a>>,
}

pub fn blob_pts(ctx: &mut Ctx, x: f64, y: f64, a: BlobArgs) -> Vec<Pt> {
    let len = a.len.unwrap_or(20.0);
    let wid = a.wid.unwrap_or(5.0);
    let ang = a.ang.unwrap_or(0.0);
    let noi = a.noi.unwrap_or(0.5);
    let default_fun = |x: f64| {
        if x <= 1.0 { (x * PI).sin().powf(0.5) } else { -((x + 1.0) * PI).sin().powf(0.5) }
    };
    let fun: Fun = a.fun.unwrap_or(&default_fun);

    let reso = 20.0;
    let mut lalist = [[0.0; 2]; 21];
    for (i, la) in lalist.iter_mut().enumerate() {
        let p = (i as f64 / reso) * 2.0;
        let xo = len / 2.0 - (p - 1.0).abs() * len;
        let yo = (fun(p) * wid) / 2.0;
        *la = [(xo * xo + yo * yo).sqrt(), yo.atan2(xo)];
    }
    let mut nslist = [0.0; 21];
    let n0 = ctx.rand() * 10.0;
    for (i, ns) in nslist.iter_mut().enumerate() {
        *ns = ctx.noise(i as f64 * 0.05, n0, 0.0);
    }
    loop_noise(&mut nslist);
    let mut plist = Vec::with_capacity(21);
    for i in 0..lalist.len() {
        let ns = nslist[i] * noi + (1.0 - noi);
        let nx = x + (lalist[i][1] + ang).cos() * lalist[i][0] * ns;
        let ny = y + (lalist[i][1] + ang).sin() * lalist[i][0] * ns;
        plist.push([nx, ny]);
    }
    plist
}

pub fn blob(ctx: &mut Ctx, x: f64, y: f64, a: BlobArgs) -> Canv {
    let col = a.col.unwrap_or(rgba(200, 200, 200, 0.9));
    let plist = blob_pts(ctx, x, y, a);
    vec![poly(&plist, PolyArgs { fil: Some(col), str: Some(col), wid: 0.0, ..Default::default() })]
}

pub type ColFn<'a> = &'a dyn Fn(&mut Ctx, f64) -> Col;
pub type DisFn<'a> = &'a dyn Fn(&mut Ctx) -> f64;

#[derive(Default, Clone, Copy)]
pub struct TexArgs<'a> {
    pub xof: f64,
    pub yof: f64,
    pub tex: Option<f64>,
    pub wid: Option<f64>,
    pub len: Option<f64>,
    pub sha: Option<f64>,
    pub noi: Option<Fun<'a>>,
    pub col: Option<ColFn<'a>>,
    pub dis: Option<DisFn<'a>>,
}

pub fn texture(ctx: &mut Ctx, ptlist: &[Vec<Pt>], a: TexArgs) -> Canv {
    let tex = a.tex.unwrap_or(400.0);
    let wid = a.wid.unwrap_or(1.5);
    let len = a.len.unwrap_or(0.2);
    let sha = a.sha.unwrap_or(0.0);
    let default_noi = |x: f64| 30.0 / x;
    let noi: Fun = a.noi.unwrap_or(&default_noi);
    let default_col = |ctx: &mut Ctx, _: f64| ink(js::fixed(ctx.rand() * 0.3, 3));
    let col: ColFn = a.col.unwrap_or(&default_col);
    let default_dis = |ctx: &mut Ctx| {
        if ctx.rand() > 0.5 { (1.0 / 3.0) * ctx.rand() } else { (1.0 * 2.0) / 3.0 + (1.0 / 3.0) * ctx.rand() }
    };
    let dis: DisFn = a.dis.unwrap_or(&default_dis);

    let reso = [ptlist.len() as f64, ptlist[0].len() as f64];
    let mut texlist: Vec<Vec<Pt>> = Vec::new();
    let mut i = 0.0;
    while i < tex {
        let mid = js::to_int32(dis(ctx) * reso[1]) as f64;
        let hlen = (ctx.rand() * (reso[1] * len)).floor();
        let start = js::min(js::max(mid - hlen, 0.0), reso[1]);
        let end = js::min(js::max(mid + hlen, 0.0), reso[1]);
        let layer = (i / tex) * (reso[0] - 1.0);
        let mut row = Vec::new();
        let mut j = start;
        while j < end {
            let p = layer - layer.floor();
            let lo = &ptlist[layer.floor() as usize][j as usize];
            let hi = &ptlist[layer.ceil() as usize][j as usize];
            let x = lo[0] * p + hi[0] * (1.0 - p);
            let y = lo[1] * p + hi[1] * (1.0 - p);
            let ns = [
                noi(layer + 1.0) * (ctx.noise(x, j * 0.5, 0.0) - 0.5),
                noi(layer + 1.0) * (ctx.noise(y, j * 0.5, 0.0) - 0.5),
            ];
            row.push([x + ns[0], y + ns[1]]);
            j += 1.0;
        }
        texlist.push(row);
        i += 1.0;
    }
    let mut canv = Canv::new();
    let n = texlist.len() as f64;
    if sha != 0.0 {
        let mut j = 0.0;
        while j < n {
            let pts = offset(&texlist[j as usize], a.xof, a.yof);
            let s = StrokeArgs { col: Some(ink(0.1)), wid: Some(sha), ..Default::default() };
            canv.extend(stroke(ctx, &pts, s));
            j += 2.0;
        }
    }
    let mut j = sha;
    while j < n {
        let pts = offset(&texlist[j as usize], a.xof, a.yof);
        let c = col(ctx, j / n);
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(c), wid: Some(wid), ..Default::default() }));
        j += 1.0 + sha;
    }
    canv
}

fn sides_of(plist: &[Pt]) -> Vec<f64> {
    let n = plist.len();
    (0..n)
        .map(|i| {
            let pt = plist[i];
            let np = plist[if i != n - 1 { i + 1 } else { 0 }];
            (sq(np[0] - pt[0]) + sq(np[1] - pt[1])).sqrt()
        })
        .collect()
}

fn area_of(plist: &[Pt]) -> f64 {
    let sl = sides_of(plist);
    let g = |i: usize| sl.get(i).copied().unwrap_or(f64::NAN);
    let (a, b, c) = (g(0), g(1), g(2));
    let s = (a + b + c) / 2.0;
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

fn shatter(plist: &[Pt], a: f64, out: &mut Vec<Vec<Pt>>) {
    if plist.is_empty() {
        return;
    }
    if area_of(plist) < a {
        out.push(plist.to_vec());
        return;
    }
    let sl = sides_of(plist);
    let mut ind = 0;
    for (i, &x) in sl.iter().enumerate() {
        if x > sl[ind] {
            ind = i;
        }
    }
    let n = plist.len();
    let nind = (ind + 1) % n;
    let lind = (ind + 2) % n;
    let mid = mid_pt(&[plist[ind], plist[nind]]);
    shatter(&[plist[ind], mid, plist[lind]], a, out);
    shatter(&[plist[lind], plist[nind], mid], a, out);
}

pub fn triangulate(plist: &[Pt], area: f64, optimize: bool) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    let mut plist = plist.to_vec();
    loop {
        if plist.len() <= 3 {
            shatter(&plist, area, &mut out);
            return out;
        }
        let n = plist.len();
        let ear = |i: usize| -> [Pt; 3] { [plist[if i != 0 { i - 1 } else { n - 1 }], plist[i], plist[if i != n - 1 { i + 1 } else { 0 }]] };
        let mut best = if optimize { None } else { Some(0) };
        if optimize {
            let mut best_ratio = 0.0;
            for i in 0..n {
                let tri = ear(i);
                let r = area_of(&tri) / sides_of(&tri).iter().fold(0.0, |m, s| m + s);
                if r >= best_ratio {
                    best = Some(i);
                    best_ratio = r;
                }
            }
        }
        match best {
            Some(b) => {
                shatter(&ear(b), area, &mut out);
                plist.remove(b);
            }
            None => {
                shatter(&plist, area, &mut out);
                return out;
            }
        }
    }
}
