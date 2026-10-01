use super::*;
use crate::js;
use std::f64::consts::PI;

#[derive(Default, Clone, Copy)]
pub struct TreeArgs<'a> {
    pub hei: Option<f64>,
    pub wid: Option<f64>,
    pub clu: Option<f64>,
    pub col: Option<Rgba>,
    pub ben: Option<Fun<'a>>,
}

fn trunk_noise(ctx: &mut Ctx, reso: usize) -> Vec<Pt> {
    (0..reso).map(|i| [ctx.noise(i as f64 * 0.5, 0.0, 0.0), ctx.noise(i as f64 * 0.5, 0.5, 0.0)]).collect()
}

pub fn tree01(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(50.0);
    let wid = a.wid.unwrap_or(3.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let reso = 10.0;
    let nslist = trunk_noise(ctx, 10);
    let leaf = col;
    let mut canv = Canv::new();
    let mut line1 = Vec::new();
    let mut line2 = Vec::new();
    for i in 0..10 {
        let fi = i as f64;
        let nx = x;
        let ny = y - (fi * hei) / reso;
        if fi >= reso / 4.0 {
            let mut j = 0.0;
            while j < (reso - fi) / 5.0 {
                let bx = nx + (ctx.rand() - 0.5) * wid * 1.2 * (reso - fi);
                let by = ny + (ctx.rand() - 0.5) * wid;
                let len = ctx.rand() * 20.0 * (reso - fi) * 0.2 + 10.0;
                let bw = ctx.rand() * 6.0 + 3.0;
                let ang = ((ctx.rand() - 0.5) * PI) / 6.0;
                let alpha = js::fixed(ctx.rand() * 0.2 + leaf.a, 1);
                let c = rgba(leaf.r, leaf.g, leaf.b, alpha);
                canv.extend(blob(ctx, bx, by, BlobArgs { len: Some(len), wid: Some(bw), ang: Some(ang), col: Some(c), ..Default::default() }));
                j += 1.0;
            }
        }
        line1.push([nx + (nslist[i][0] - 0.5) * wid - wid / 2.0, ny]);
        line2.push([nx + (nslist[i][1] - 0.5) * wid + wid / 2.0, ny]);
    }
    canv.push(poly(&line1, PolyArgs { fil: Some(NONE), str: Some(Some(col)), wid: 1.5, ..Default::default() }));
    canv.push(poly(&line2, PolyArgs { fil: Some(NONE), str: Some(Some(col)), wid: 1.5, ..Default::default() }));
    canv
}

pub fn tree02(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(16.0);
    let wid = a.wid.unwrap_or(8.0);
    let clu = a.clu.unwrap_or(5.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let fun = |x: f64| {
        if x <= 1.0 {
            ((x * PI).sin() * x).powf(0.5)
        } else {
            -((x - 2.0) * PI * (x - 2.0)).sin().powf(0.5)
        }
    };
    let mut canv = Canv::new();
    let mut i = 0.0;
    while i < clu {
        let bx = x + ctx.gaussian() * clu * 4.0;
        let by = y + ctx.gaussian() * clu * 4.0;
        let bw = ctx.rand() * wid * 0.75 + wid * 0.5;
        let len = ctx.rand() * hei * 0.75 + hei * 0.5;
        let args = BlobArgs { ang: Some(PI / 2.0), fun: Some(&fun), wid: Some(bw), len: Some(len), col: Some(Some(col)), ..Default::default() };
        canv.extend(blob(ctx, bx, by, args));
        i += 1.0;
    }
    canv
}

pub fn tree03(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(50.0);
    let wid = a.wid.unwrap_or(5.0);
    let zero = |_: f64| 0.0;
    let ben: Fun = a.ben.unwrap_or(&zero);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let reso = 10.0;
    let nslist = trunk_noise(ctx, 10);
    let leaf = col;
    let mut canv = Canv::new();
    let mut blobs = Canv::new();
    let mut line1 = Vec::new();
    let mut line2 = Vec::new();
    for i in 0..10 {
        let fi = i as f64;
        let nx = x + ben(fi / reso) * 100.0;
        let ny = y - (fi * hei) / reso;
        if fi >= reso / 5.0 {
            let mut j = 0.0;
            while j < (reso - fi) * 2.0 {
                let shape = |x: f64| (50.0 * x + 1.0).ln() / 3.95;
                let ox = ctx.rand() * wid * 2.0 * shape((reso - fi) / reso);
                let bx = nx + ox * ctx.choice(&[-1.0, 1.0]);
                let by = ny + (ctx.rand() - 0.5) * wid * 2.0;
                let bw = ctx.rand() * 6.0 + 3.0;
                let ang = ((ctx.rand() - 0.5) * PI) / 6.0;
                let alpha = js::fixed(ctx.rand() * 0.2 + leaf.a, 3);
                let c = rgba(leaf.r, leaf.g, leaf.b, alpha);
                blobs.extend(blob(ctx, bx, by, BlobArgs { len: Some(ox * 2.0), wid: Some(bw), ang: Some(ang), col: Some(c), ..Default::default() }));
                j += 1.0;
            }
        }
        line1.push([nx + (((nslist[i][0] - 0.5) * wid - wid / 2.0) * (reso - fi)) / reso, ny]);
        line2.push([nx + (((nslist[i][1] - 0.5) * wid + wid / 2.0) * (reso - fi)) / reso, ny]);
    }
    let lc: Vec<Pt> = line1.iter().chain(line2.iter().rev()).copied().collect();
    canv.push(poly(&lc, PolyArgs { fil: Some(WHITE), str: Some(Some(col)), wid: 1.5, ..Default::default() }));
    canv.extend(blobs);
    canv
}

#[derive(Clone, Copy)]
struct BranchArgs {
    hei: f64,
    wid: f64,
    ang: f64,
    det: f64,
    ben: f64,
}

impl Default for BranchArgs {
    fn default() -> Self {
        BranchArgs { hei: 300.0, wid: 6.0, ang: 0.0, det: 10.0, ben: PI * 0.2 }
    }
}

fn branch(ctx: &mut Ctx, a: BranchArgs) -> [Vec<Pt>; 2] {
    let BranchArgs { hei, wid, ang, det, ben } = a;
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut tlist = vec![[nx, ny]];
    let mut a0 = 0.0;
    let g = 3.0;
    for _ in 0..3 {
        let r = ben / 2.0 + (ctx.rand() * ben) / 2.0;
        a0 += r * ctx.choice(&[-1.0, 1.0]);
        nx += (a0.cos() * hei) / g;
        ny -= (a0.sin() * hei) / g;
        tlist.push([nx, ny]);
    }
    let last = tlist[tlist.len() - 1];
    let ta = last[1].atan2(last[0]);
    for t in tlist.iter_mut() {
        let a = t[1].atan2(t[0]);
        let d = (t[0] * t[0] + t[1] * t[1]).sqrt();
        t[0] = d * (a - ta + ang).cos();
        t[1] = d * (a - ta + ang).sin();
    }

    let mut trlist1 = Vec::new();
    let mut trlist2 = Vec::new();
    let span = det;
    let tl = (tlist.len() as f64 - 1.0) * span;
    let mut lx = 0.0;
    let mut ly = 0.0;
    let mut i = 0.0;
    while i < tl {
        let lastp = tlist[(i / span).floor() as usize];
        let nextp = tlist[(i / span).ceil() as usize];
        let p = (i % span) / span;
        let nx = lastp[0] * (1.0 - p) + nextp[0] * p;
        let ny = lastp[1] * (1.0 - p) + nextp[1] * p;
        let ang = (ny - ly).atan2(nx - lx);
        let woff = ((ctx.noise(i * 0.3, 0.0, 0.0) - 0.5) * wid * hei) / 80.0;
        let mut b = 0.0;
        if p == 0.0 {
            b = ctx.rand() * wid;
        }
        let nw = wid * (((tl - i) / tl) * 0.5 + 0.5);
        trlist1.push([
            nx + (ang + PI / 2.0).cos() * (nw + woff + b),
            ny + (ang + PI / 2.0).sin() * (nw + woff + b),
        ]);
        trlist2.push([
            nx + (ang - PI / 2.0).cos() * (nw - woff + b),
            ny + (ang - PI / 2.0).sin() * (nw - woff + b),
        ]);
        lx = nx;
        ly = ny;
        i += 1.0;
    }
    [trlist1, trlist2]
}

#[derive(Clone, Copy)]
struct TwigArgs {
    dir: f64,
    sca: f64,
    wid: f64,
    ang: f64,
    lea: (bool, f64),
}

impl Default for TwigArgs {
    fn default() -> Self {
        TwigArgs { dir: 1.0, sca: 1.0, wid: 1.0, ang: 0.0, lea: (true, 12.0) }
    }
}

fn leaf_fun(x: f64) -> f64 {
    if x <= 1.0 {
        ((x * PI).sin() * x).powf(0.5)
    } else {
        -((x - 2.0) * PI * (x - 2.0)).sin().powf(0.5)
    }
}

fn twig(ctx: &mut Ctx, tx: f64, ty: f64, dep: f64, a: TwigArgs) -> Canv {
    let TwigArgs { dir, sca, wid, ang, lea } = a;
    let mut canv = Canv::new();
    let mut twlist = Vec::new();
    let tl = 10.0;
    let hs = ctx.rand() * 0.5 + 0.5;
    let _ = ctx.choice(&[0]);
    let a0 = ((ctx.rand() * PI) / 6.0) * dir + ang;
    for i in 0..10 {
        let fi = i as f64;
        let tfun = -1.0 / (fi / tl + 1.0).powf(5.0) + 1.0;
        let mx = dir * tfun * 50.0 * sca * hs;
        let my = -fi * 5.0 * sca;
        let a = my.atan2(mx);
        let d = (mx * mx + my * my).powf(0.5);
        let nx = (a + a0).cos() * d;
        let ny = (a + a0).sin() * d;
        twlist.push([nx + tx, ny + ty]);
        if (fi == js::to_int32(tl / 3.0) as f64 || fi == js::to_int32((tl * 2.0) / 3.0) as f64) && dep > 0.0 {
            let ndir = dir * ctx.choice(&[-1.0, 1.0]);
            canv.extend(twig(ctx, nx + tx, ny + ty, dep - 1.0, TwigArgs { ang, sca: sca * 0.8, wid, dir: ndir, lea }));
        }
        if fi == tl - 1.0 && lea.0 {
            for j in 0..5 {
                let dj = (j as f64 - 2.5) * 5.0;
                let bx = nx + tx + ang.cos() * dj * wid;
                let by = ny + ty + (ang.sin() * dj - lea.1 / (dep + 1.0)) * wid;
                let bw = (6.0 + 3.0 * ctx.rand()) * wid;
                let len = (15.0 + 12.0 * ctx.rand()) * wid;
                let bang = ang / 2.0 + PI / 2.0 + PI * 0.2 * (ctx.rand() - 0.5);
                let col = ink(js::fixed(0.5 + dep * 0.2, 3));
                let args = BlobArgs { wid: Some(bw), len: Some(len), ang: Some(bang), col: Some(col), fun: Some(&leaf_fun), ..Default::default() };
                canv.extend(blob(ctx, bx, by, args));
            }
        }
    }
    let fun = |x: f64| ((x * PI) / 2.0).cos();
    canv.extend(stroke(ctx, &twlist, StrokeArgs { wid: Some(1.0), fun: Some(&fun), col: Some(ink(0.5)), ..Default::default() }));
    canv
}

fn bark(ctx: &mut Ctx, x: f64, y: f64, wid: f64, ang: f64) -> Canv {
    let len = 10.0 + 10.0 * ctx.rand();
    let noi = 0.5;
    let fun = |x: f64| {
        if x <= 1.0 { (x * PI).sin().powf(0.5) } else { -((x + 1.0) * PI).sin().powf(0.5) }
    };
    let reso = 20.0;
    let mut lalist = Vec::with_capacity(21);
    for i in 0..21 {
        let p = (i as f64 / reso) * 2.0;
        let xo = len / 2.0 - (p - 1.0).abs() * len;
        let yo = (fun(p) * wid) / 2.0;
        lalist.push([(xo * xo + yo * yo).sqrt(), yo.atan2(xo)]);
    }
    let n0 = ctx.rand() * 10.0;
    let mut nslist: Vec<f64> = (0..21).map(|i| ctx.noise(i as f64 * 0.05, n0, 0.0)).collect();
    loop_noise(&mut nslist);
    let mut brklist = Vec::with_capacity(21);
    for i in 0..lalist.len() {
        let ns = nslist[i] * noi + (1.0 - noi);
        brklist.push([
            x + (lalist[i][1] + ang).cos() * lalist[i][0] * ns,
            y + (lalist[i][1] + ang).sin() * lalist[i][0] * ns,
        ]);
    }
    let fr = ctx.rand();
    let sfun = |x: f64| ((x + fr) * PI * 3.0).sin();
    stroke(ctx, &brklist, StrokeArgs { wid: Some(0.8), noi: Some(0.0), col: Some(ink(0.4)), out: Some(0.0), fun: Some(&sfun), ..Default::default() })
}

fn barkify(ctx: &mut Ctx, x: f64, y: f64, trlist: &mut [Vec<Pt>; 2]) -> Canv {
    let mut canv = Canv::new();
    let n = trlist[0].len();
    let mut i = 2;
    while i + 1 < n {
        let (t0, t1) = (&trlist[0], &trlist[1]);
        let a0 = (t0[i][1] - t0[i - 1][1]).atan2(t0[i][0] - t0[i - 1][0]);
        let a1 = (t1[i][1] - t1[i - 1][1]).atan2(t1[i][0] - t1[i - 1][0]);
        let p = ctx.rand();
        let nx = t0[i][0] * (1.0 - p) + t1[i][0] * p;
        let ny = t0[i][1] * (1.0 - p) + t1[i][1] * p;
        let (p0, p1) = (t0[i], t1[i]);
        if ctx.rand() < 0.2 {
            let args = BlobArgs { noi: Some(1.0), len: Some(15.0), wid: Some(6.0 - (p - 0.5).abs() * 10.0), ang: Some((a0 + a1) / 2.0), col: Some(ink(0.6)), ..Default::default() };
            canv.extend(blob(ctx, nx + x, ny + y, args));
        } else {
            canv.extend(bark(ctx, nx + x, ny + y, 5.0 - (p - 0.5).abs() * 10.0, (a0 + a1) / 2.0));
        }
        if ctx.rand() < 0.05 {
            let jl = ctx.rand() * 2.0 + 2.0;
            let xya = if ctx.choice(&[0, 1]) == 0 { [p0[0], p0[1], a0] } else { [p1[0], p1[1], a1] };
            let mut j = 0.0;
            while j < jl {
                let bx = xya[0] + x + xya[2].cos() * (j - jl / 2.0) * 4.0;
                let by = xya[1] + y + xya[2].sin() * (j - jl / 2.0) * 4.0;
                let len = 4.0 + 6.0 * ctx.rand();
                let args = BlobArgs { wid: Some(4.0), len: Some(len), ang: Some(a0 + PI / 2.0), col: Some(ink(0.6)), ..Default::default() };
                canv.extend(blob(ctx, bx, by, args));
                j += 1.0;
            }
        }
        i += 1;
    }

    let refs: Vec<(usize, usize)> = (0..trlist[0].len()).map(|k| (0, k)).chain((0..trlist[1].len()).rev().map(|k| (1, k))).collect();
    let mut rglist: Vec<Vec<(usize, usize)>> = vec![Vec::new()];
    for r in refs {
        if ctx.rand() < 0.5 {
            rglist.push(Vec::new());
        } else {
            rglist.last_mut().unwrap().push(r);
        }
    }
    for (gi, group) in rglist.iter().enumerate() {
        let pts: Vec<Pt> = group.iter().map(|&(s, k)| trlist[s][k]).collect();
        let mut line = div(&pts, 4.0);
        for (j, p) in line.iter_mut().enumerate() {
            let g = gi as f64;
            p[0] += (ctx.noise(g, j as f64 * 0.1, 1.0) - 0.5) * (15.0 + 5.0 * ctx.gaussian());
            p[1] += (ctx.noise(g, j as f64 * 0.1, 2.0) - 0.5) * (15.0 + 5.0 * ctx.gaussian());
        }
        if let (Some(&(s, k)), Some(&last)) = (group.last(), line.last()) {
            trlist[s][k] = last;
        }
        let pts = offset(&line, x, y);
        canv.extend(stroke(ctx, &pts, StrokeArgs { wid: Some(1.5), col: Some(ink(0.7)), out: Some(0.0), ..Default::default() }));
    }
    canv
}

fn concat_rev(a: &[Pt], b: &[Pt]) -> Vec<Pt> {
    a.iter().chain(b.iter().rev()).copied().collect()
}

fn trunk_stroke(ctx: &mut Ctx, trmlist: &mut Vec<Pt>, x: f64, y: f64, base: f64) -> Canv {
    trmlist.remove(0);
    trmlist.pop();
    let pts = offset(trmlist, x, y);
    let col = ink(js::fixed(base + ctx.rand() * 0.1, 3));
    let fun = |_: f64| 1f64.sin();
    stroke(ctx, &pts, StrokeArgs { col: Some(col), wid: Some(2.5), fun: Some(&fun), noi: Some(0.9), out: Some(0.0), ..Default::default() })
}

pub fn tree04(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(300.0);
    let wid = a.wid.unwrap_or(6.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let mut canv = Canv::new();
    let mut txcanv = Canv::new();
    let mut twcanv = Canv::new();

    let mut tr = branch(ctx, BranchArgs { hei, wid, ang: -PI / 2.0, ..Default::default() });
    txcanv.extend(barkify(ctx, x, y, &mut tr));
    let trlist = concat_rev(&tr[0], &tr[1]);
    let n = trlist.len() as f64;

    let mut trmlist = Vec::new();
    for i in 0..trlist.len() {
        let fi = i as f64;
        if (fi >= n * 0.3 && fi <= n * 0.7 && ctx.rand() < 0.1) || fi == n / 2.0 - 1.0 {
            let ba = PI * 0.2 - PI * 1.4 * ((fi > n / 2.0) as u8 as f64);
            let bh = hei * (ctx.rand() + 1.0) * 0.3;
            let mut br = branch(ctx, BranchArgs { hei: bh, wid: wid * 0.5, ang: ba, ..Default::default() });
            br[0].remove(0);
            br[1].remove(0);
            let t = trlist[i];
            let mut moved = [offset(&br[0], t[0], t[1]), offset(&br[1], t[0], t[1])];
            txcanv.extend(barkify(ctx, x, y, &mut moved));
            for j in 0..br[0].len() {
                if ctx.rand() < 0.2 || j == br[0].len() - 1 {
                    let args = TwigArgs {
                        wid: hei / 300.0,
                        ang: if ba > -PI / 2.0 { ba } else { ba + PI },
                        sca: (0.5 * hei) / 300.0,
                        dir: if ba > -PI / 2.0 { 1.0 } else { -1.0 },
                        ..Default::default()
                    };
                    twcanv.extend(twig(ctx, br[0][j][0] + t[0] + x, br[0][j][1] + t[1] + y, 1.0, args));
                }
            }
            let brlist = concat_rev(&br[0], &br[1]);
            trmlist.extend(offset(&brlist, t[0], t[1]));
        } else {
            trmlist.push(trlist[i]);
        }
    }
    canv.push(poly(&trmlist, PolyArgs { xof: x, yof: y, fil: Some(WHITE), str: Some(Some(col)), wid: 0.0 }));
    canv.extend(trunk_stroke(ctx, &mut trmlist, x, y, 0.4));
    canv.extend(txcanv);
    canv.extend(twcanv);
    canv
}

pub fn tree05(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(300.0);
    let wid = a.wid.unwrap_or(5.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let mut canv = Canv::new();
    let mut txcanv = Canv::new();
    let mut twcanv = Canv::new();

    let mut tr = branch(ctx, BranchArgs { hei, wid, ang: -PI / 2.0, ben: 0.0, ..Default::default() });
    txcanv.extend(barkify(ctx, x, y, &mut tr));
    let trlist = concat_rev(&tr[0], &tr[1]);
    let n = trlist.len() as f64;

    let mut trmlist = Vec::new();
    for i in 0..trlist.len() {
        let fi = i as f64;
        let p = (fi - n * 0.5).abs() / (n * 0.5);
        if (fi >= n * 0.2 && fi <= n * 0.8 && i % 3 == 0 && ctx.rand() > p) || fi == n / 2.0 - 1.0 {
            let bar = ctx.rand() * 0.2;
            let ba = -bar * PI - (1.0 - bar * 2.0) * PI * ((fi > n / 2.0) as u8 as f64);
            let bh = hei * (0.3 * p - ctx.rand() * 0.05);
            let mut br = branch(ctx, BranchArgs { hei: bh, wid: wid * 0.5, ang: ba, ben: 0.5, ..Default::default() });
            br[0].remove(0);
            br[1].remove(0);
            let t = trlist[i];
            for j in 0..br[0].len() {
                if j % 20 == 0 || j == br[0].len() - 1 {
                    let args = TwigArgs {
                        wid: hei / 300.0,
                        ang: if ba > -PI / 2.0 { ba } else { ba + PI },
                        sca: (0.2 * hei) / 300.0,
                        dir: if ba > -PI / 2.0 { 1.0 } else { -1.0 },
                        lea: (true, 5.0),
                    };
                    twcanv.extend(twig(ctx, br[0][j][0] + t[0] + x, br[0][j][1] + t[1] + y, 0.0, args));
                }
            }
            let brlist = concat_rev(&br[0], &br[1]);
            trmlist.extend(offset(&brlist, t[0], t[1]));
        } else {
            trmlist.push(trlist[i]);
        }
    }
    canv.push(poly(&trmlist, PolyArgs { xof: x, yof: y, fil: Some(WHITE), str: Some(Some(col)), wid: 0.0 }));
    canv.extend(trunk_stroke(ctx, &mut trmlist, x, y, 0.4));
    canv.extend(txcanv);
    canv.extend(twcanv);
    canv
}

struct Frac06<'a> {
    txcanv: &'a mut Canv,
    twcanv: &'a mut Canv,
}

fn frac_tree06(ctx: &mut Ctx, f: &mut Frac06, xoff: f64, yoff: f64, dep: f64, hei: f64, wid: f64, ang: f64, ben: f64) -> Vec<Pt> {
    let mut tr = branch(ctx, BranchArgs { hei, wid, ang, ben, det: hei / 20.0 });
    f.txcanv.extend(barkify(ctx, xoff, yoff, &mut tr));
    let trlist = concat_rev(&tr[0], &tr[1]);
    let n = trlist.len() as f64;
    let half = js::to_int32(n / 2.0) as f64;

    let mut trmlist = Vec::new();
    for i in 0..trlist.len() {
        let fi = i as f64;
        let _p = (fi - n * 0.5).abs() / (n * 0.5);
        if ((ctx.rand() < 0.025 && fi >= n * 0.2 && fi <= n * 0.8) || fi == half - 1.0 || fi == half + 1.0) && dep > 0.0 {
            let bar = 0.02 + ctx.rand() * 0.08;
            let ba = bar * PI - bar * 2.0 * PI * ((fi > n / 2.0) as u8 as f64);
            let t = trlist[i];
            let bh = hei * (0.7 + ctx.rand() * 0.2);
            let brlist = frac_tree06(ctx, f, t[0] + xoff, t[1] + yoff, dep - 1.0, bh, wid * 0.6, ang + ba, 0.55);
            for b in &brlist {
                if ctx.rand() < 0.03 {
                    let tang = ba * (ctx.rand() * 0.5 + 0.75);
                    let args = TwigArgs { ang: tang, sca: 0.3, dir: if ba > 0.0 { 1.0 } else { -1.0 }, lea: (false, 0.0), ..Default::default() };
                    f.twcanv.extend(twig(ctx, b[0] + t[0] + xoff, b[1] + t[1] + yoff, 2.0, args));
                }
            }
            trmlist.extend(offset(&brlist, t[0], t[1]));
        } else {
            trmlist.push(trlist[i]);
        }
    }
    trmlist
}

pub fn tree06(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(100.0);
    let wid = a.wid.unwrap_or(6.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let mut canv = Canv::new();
    let mut txcanv = Canv::new();
    let mut twcanv = Canv::new();
    let mut f = Frac06 { txcanv: &mut txcanv, twcanv: &mut twcanv };
    let mut trmlist = frac_tree06(ctx, &mut f, x, y, 3.0, hei, wid, -PI / 2.0, 0.0);

    canv.push(poly(&trmlist, PolyArgs { xof: x, yof: y, fil: Some(WHITE), str: Some(Some(col)), wid: 0.0 }));
    canv.extend(trunk_stroke(ctx, &mut trmlist, x, y, 0.4));
    canv.extend(txcanv);
    canv.extend(twcanv);
    canv
}

pub fn tree07(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(60.0);
    let wid = a.wid.unwrap_or(4.0);
    let sqrt_ben = |x: f64| x.sqrt() * 0.2;
    let ben: Fun = a.ben.unwrap_or(&sqrt_ben);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 1.0 });

    let reso = 10.0;
    let nslist = trunk_noise(ctx, 10);
    let leaf = col;
    let mut canv = Canv::new();
    let mut line1 = Vec::new();
    let mut line2 = Vec::new();
    let mut t: Vec<Vec<Pt>> = Vec::new();
    let leaf_shape = |x: f64| {
        if x <= 1.0 {
            2.75 * x * (1.0 - x).powf(1.0 / 1.8)
        } else {
            2.75 * (x - 2.0) * (x - 1.0).powf(1.0 / 1.8)
        }
    };
    for i in 0..10 {
        let fi = i as f64;
        let nx = x + ben(fi / reso) * 100.0;
        let ny = y - (fi * hei) / reso;
        if fi >= reso / 4.0 {
            let bx = nx + (ctx.rand() - 0.5) * wid * 1.2 * (reso - fi) * 0.5;
            let by = ny + (ctx.rand() - 0.5) * wid * 0.5;
            let len = ctx.rand() * 50.0 + 20.0;
            let bw = ctx.rand() * 12.0 + 12.0;
            let ang = (-ctx.rand() * PI) / 6.0;
            let c = rgba(leaf.r, leaf.g, leaf.b, js::fixed(leaf.a, 3));
            let args = BlobArgs { len: Some(len), wid: Some(bw), ang: Some(ang), col: Some(c), fun: Some(&leaf_shape), ..Default::default() };
            let bpl = blob_pts(ctx, bx, by, args);
            t.extend(triangulate(&bpl, 50.0, false));
        }
        line1.push([nx + (nslist[i][0] - 0.5) * wid - wid / 2.0, ny]);
        line2.push([nx + (nslist[i][1] - 0.5) * wid + wid / 2.0, ny]);
    }
    let mut all = triangulate(&concat_rev(&line1, &line2), 50.0, true);
    all.extend(t);
    for tri in &all {
        let m = mid_pt(tri);
        let c = js::to_int32(ctx.noise(m[0] * 0.02, m[1] * 0.02, 0.0) * 200.0 + 50.0);
        let c = c.clamp(0, 255) as u8;
        let co = rgba(c, c, c, 0.8);
        canv.push(poly(tri, PolyArgs { fil: Some(co), str: Some(co), wid: 0.0, ..Default::default() }));
    }
    canv
}

fn frac_tree08(ctx: &mut Ctx, xoff: f64, yoff: f64, dep: f64, ang: f64, len: f64, ben: f64) -> Canv {
    let fun = |x: f64| if dep == 0.0 { (0.5 * PI * x).cos() } else { 1.0 };
    let spt = [xoff, yoff];
    let ept = [xoff + ang.cos() * len, yoff + ang.sin() * len];
    let neg = ctx.choice(&[false, true]);
    let mut trmlist = div(&[[xoff, yoff], [xoff + len, yoff]], 10.0);
    let n = trmlist.len() as f64;
    for (i, p) in trmlist.iter_mut().enumerate() {
        let s = (i as f64 / n * PI).sin();
        p[1] += (if neg { -s } else { s }) * 2.0;
    }
    for p in trmlist.iter_mut() {
        let d = distance(*p, spt);
        let a = (p[1] - spt[1]).atan2(p[0] - spt[0]);
        p[0] = spt[0] + d * (a + ang).cos();
        p[1] = spt[1] + d * (a + ang).sin();
    }
    let mut tcanv = stroke(ctx, &trmlist, StrokeArgs { fun: Some(&fun), wid: Some(0.8), col: Some(ink(0.5)), ..Default::default() });
    if dep != 0.0 {
        let nben = ben + ctx.choice(&[-1.0, 1.0]) * PI * 0.001 * dep * dep;
        if ctx.rand() < 0.5 {
            let c0 = ctx.norm_rand(-1.0, 0.5);
            let c1 = ctx.norm_rand(0.5, 1.0);
            let nang = ang + ben + PI * ctx.choice(&[c0, c1]) * 0.2;
            let nlen = len * ctx.norm_rand(0.8, 0.9);
            tcanv.extend(frac_tree08(ctx, ept[0], ept[1], dep - 1.0, nang, nlen, nben));
            let c0 = ctx.norm_rand(-1.0, -0.5);
            let c1 = ctx.norm_rand(0.5, 1.0);
            let nang = ang + ben + PI * ctx.choice(&[c0, c1]) * 0.2;
            let nlen = len * ctx.norm_rand(0.8, 0.9);
            tcanv.extend(frac_tree08(ctx, ept[0], ept[1], dep - 1.0, nang, nlen, nben));
        } else {
            let nlen = len * ctx.norm_rand(0.8, 0.9);
            tcanv.extend(frac_tree08(ctx, ept[0], ept[1], dep - 1.0, ang + ben, nlen, nben));
        }
    }
    tcanv
}

pub fn tree08(ctx: &mut Ctx, x: f64, y: f64, a: TreeArgs) -> Canv {
    let hei = a.hei.unwrap_or(80.0);
    let wid = a.wid.unwrap_or(1.0);
    let col = a.col.unwrap_or(Rgba { r: 100, g: 100, b: 100, a: 0.5 });

    let mut canv = Canv::new();
    let mut twcanv = Canv::new();
    let ang = ctx.norm_rand(-1.0, 1.0) * PI * 0.2;
    let tr = branch(ctx, BranchArgs { hei, wid, ang: -PI / 2.0 + ang, ben: PI * 0.2, det: hei / 20.0 });
    let trlist = concat_rev(&tr[0], &tr[1]);

    for i in 0..trlist.len() {
        if ctx.rand() < 0.2 {
            let dep = (4.0 * ctx.rand()).floor();
            let fang = -PI / 2.0 - ang * ctx.rand();
            twcanv.extend(frac_tree08(ctx, x + trlist[i][0], y + trlist[i][1], dep, fang, 15.0, 0.0));
        } else if i == trlist.len() / 2 {
            twcanv.extend(frac_tree08(ctx, x + trlist[i][0], y + trlist[i][1], 3.0, -PI / 2.0 + ang, 15.0, 0.0));
        }
    }
    canv.push(poly(&trlist, PolyArgs { xof: x, yof: y, fil: Some(WHITE), str: Some(Some(col)), wid: 0.0 }));
    let pts = offset(&trlist, x, y);
    let scol = ink(js::fixed(0.6 + ctx.rand() * 0.1, 3));
    let fun = |_: f64| 1f64.sin();
    canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(scol), wid: Some(2.5), fun: Some(&fun), noi: Some(0.9), out: Some(0.0), ..Default::default() }));
    canv.extend(twcanv);
    canv
}
