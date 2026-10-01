use super::arch;
use super::mount::{self, DistArgs, FlatArgs};
use super::*;
use crate::js;
use std::collections::HashMap;

pub struct Chunk {
    pub tag: &'static str,
    pub x: f64,
    pub y: f64,
    pub canv: Canv,
    pub slice: f64,
}

pub fn water(ctx: &mut Ctx, xoff: f64, yoff: f64) -> Canv {
    let hei = 2.0;
    let len = 800.0;
    let clu = 10;
    let mut canv = Canv::new();
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    let mut yk = 0.0;
    for _ in 0..clu {
        let mut row = Vec::new();
        let xk = (ctx.rand() - 0.5) * (len / 8.0);
        yk += ctx.rand() * 5.0;
        let lk = len / 4.0 + ctx.rand() * (len / 4.0);
        let reso = 5.0;
        let mut j = -lk;
        while j < lk {
            row.push([j + xk, (j * 0.2).sin() * hei * ctx.noise(j * 0.1, 0.0, 0.0) - 20.0 + yk]);
            j += reso;
        }
        ptlist.push(row);
    }
    for row in &ptlist[1..] {
        let pts = offset(row, xoff, yoff);
        let col = ink(js::fixed(0.3 + ctx.rand() * 0.3, 3));
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(col), wid: Some(1.0), ..Default::default() }));
    }
    canv
}

#[derive(Clone, Copy, PartialEq)]
enum Tag {
    Mount,
    DistMount,
    FlatMount,
    Boat,
}

struct Plan {
    tag: Tag,
    x: f64,
    y: f64,
}

pub struct World {
    pub ctx: Ctx,
    pub xmin: f64,
    pub xmax: f64,
    pub cwid: f64,
    planmtx: HashMap<i64, f64>,
}

impl World {
    pub fn new(seed: &str) -> World {
        World { ctx: Ctx::new(seed), xmin: 0.0, xmax: 0.0, cwid: 512.0, planmtx: HashMap::new() }
    }

    fn pm(&self, k: i64) -> f64 {
        self.planmtx.get(&k).copied().unwrap_or(f64::NAN)
    }

    fn mountplanner(&mut self, xmin: f64, xmax: f64) -> Vec<Plan> {
        let samp = 0.03;
        let ns = |ctx: &mut Ctx, x: f64| js::max(ctx.noise(x * samp, 0.0, 0.0) - 0.55, 0.0) * 2.0;
        let yr = |ctx: &mut Ctx, x: f64| ctx.noise(x * 0.01, std::f64::consts::PI, 0.0);
        let locmax = |ctx: &mut Ctx, x: f64, r: f64| {
            let z0 = ns(ctx, x);
            if z0 <= 0.3 {
                return false;
            }
            let mut i = x - r;
            while i < x + r {
                if ns(ctx, i) > z0 {
                    return false;
                }
                i += 1.0;
            }
            true
        };
        let mut reg: Vec<Plan> = Vec::new();
        let chadd = |reg: &mut Vec<Plan>, r: Plan, mind: f64| {
            if reg.iter().any(|q| (q.x - r.x).abs() < mind) {
                return false;
            }
            reg.push(r);
            true
        };

        let xstep = 5.0;
        let mwid = 200.0;
        let mut i = xmin;
        while i < xmax {
            let i1 = (i / xstep).floor() as i64;
            let v = self.pm(i1);
            self.planmtx.insert(i1, if v.is_nan() || v == 0.0 { 0.0 } else { v });
            i += xstep;
        }

        let mut i = xmin;
        while i < xmax {
            let mut j = 0.0;
            while j < yr(&mut self.ctx, i) * 480.0 {
                if locmax(&mut self.ctx, i, 2.0) {
                    let xof = i + 2.0 * (self.ctx.rand() - 0.5) * 500.0;
                    let yof = j + 300.0;
                    if chadd(&mut reg, Plan { tag: Tag::Mount, x: xof, y: yof }, 10.0) {
                        let mut k = ((xof - mwid) / xstep).floor();
                        while k < (xof + mwid) / xstep {
                            let v = self.pm(k as i64);
                            self.planmtx.insert(k as i64, v + 1.0);
                            k += 1.0;
                        }
                    }
                }
                j += 30.0;
            }
            if i.abs() % 1000.0 < js::max(1.0, xstep - 1.0) {
                let y = 280.0 - self.ctx.rand() * 50.0;
                chadd(&mut reg, Plan { tag: Tag::DistMount, x: i, y }, 10.0);
            }
            i += xstep;
        }

        let mut i = xmin;
        while i < xmax {
            if self.pm((i / xstep).floor() as i64) == 0.0 && self.ctx.rand() < 0.01 {
                let mut j = 0.0;
                while j < 4.0 * self.ctx.rand() {
                    let x = i + 2.0 * (self.ctx.rand() - 0.5) * 700.0;
                    chadd(&mut reg, Plan { tag: Tag::FlatMount, x, y: 700.0 - j * 50.0 }, 10.0);
                    j += 1.0;
                }
            }
            i += xstep;
        }

        let mut i = xmin;
        while i < xmax {
            if self.ctx.rand() < 0.2 {
                let y = 300.0 + self.ctx.rand() * 390.0;
                chadd(&mut reg, Plan { tag: Tag::Boat, x: i, y }, 400.0);
            }
            i += xstep;
        }
        reg
    }

    pub fn chunkloader(&mut self, xmin: f64, xmax: f64, emit: &mut dyn FnMut(Chunk)) {
        while xmax > self.xmax - self.cwid || xmin < self.xmin + self.cwid {
            let slice = if xmax > self.xmax - self.cwid { self.xmax } else { self.xmin - self.cwid };
            let plan = if xmax > self.xmax - self.cwid {
                let p = self.mountplanner(self.xmax, self.xmax + self.cwid);
                self.xmax += self.cwid;
                p
            } else {
                let p = self.mountplanner(self.xmin - self.cwid, self.xmin);
                self.xmin -= self.cwid;
                p
            };
            self.prune_planmtx();
            let ctx = &mut self.ctx;
            for (i, p) in plan.iter().enumerate() {
                let i = i as f64;
                match p.tag {
                    Tag::Mount => {
                        let seed = i * 2.0 * ctx.rand();
                        emit(Chunk { tag: "mount", x: p.x, y: p.y, canv: mount::mountain(ctx, p.x, p.y, seed), slice });
                        emit(Chunk { tag: "mount", x: p.x, y: p.y - 10000.0, canv: water(ctx, p.x, p.y), slice });
                    }
                    Tag::FlatMount => {
                        let seed = 2.0 * ctx.rand() * std::f64::consts::PI;
                        let wid = 600.0 + ctx.rand() * 400.0;
                        let cho = 0.5 + ctx.rand() * 0.2;
                        let canv = mount::flat_mount(ctx, p.x, p.y, seed, FlatArgs { wid, hei: 100.0, cho });
                        emit(Chunk { tag: "flatmount", x: p.x, y: p.y, canv, slice });
                    }
                    Tag::DistMount => {
                        let seed = ctx.rand() * 100.0;
                        let len = ctx.choice(&[500.0, 1000.0, 1500.0]);
                        let canv = mount::dist_mount(ctx, p.x, p.y, seed, DistArgs { hei: 150.0, len });
                        emit(Chunk { tag: "distmount", x: p.x, y: p.y, canv, slice });
                    }
                    Tag::Boat => {
                        let seed = ctx.rand();
                        let _ = seed;
                        let fli = ctx.choice(&[true, false]);
                        let canv = arch::boat01(ctx, p.x, p.y, p.y / 800.0, fli);
                        emit(Chunk { tag: "boat", x: p.x, y: p.y, canv, slice });
                    }
                }
            }
        }
    }

    fn prune_planmtx(&mut self) {
        let keep = (self.cwid + 1500.0) / 5.0;
        let (lo, hi) = (self.xmin / 5.0 - keep, self.xmax / 5.0 + keep);
        if self.planmtx.len() > 4096 {
            self.planmtx.retain(|&k, _| (k as f64) > lo && (k as f64) < hi);
        }
    }
}

pub fn insert_chunk<T>(chunks: &mut Vec<T>, nch: T, y: impl Fn(&T) -> f64) {
    let ny = y(&nch);
    if chunks.is_empty() {
        chunks.push(nch);
    } else if ny <= y(&chunks[0]) {
        chunks.insert(0, nch);
    } else if ny >= y(&chunks[chunks.len() - 1]) {
        chunks.push(nch);
    } else {
        for j in 0..chunks.len() - 1 {
            if y(&chunks[j]) <= ny && ny <= y(&chunks[j + 1]) {
                chunks.insert(j + 1, nch);
                return;
            }
        }
    }
}
