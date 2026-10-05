use super::arch;
use super::tree::{self, TreeArgs};
use super::*;
use crate::js;
use std::f64::consts::PI;

fn foot(ctx: &mut Ctx, ptlist: &[Vec<Pt>], xof: f64, yof: f64) -> Canv {
    let mut ftlist: Vec<Vec<Pt>> = Vec::new();
    let span = 10;
    let mut ni = 0;
    let n = ptlist.len();
    for i in 0..n.saturating_sub(2) {
        if i == ni {
            ni = (ni + ctx.choice(&[1, 2])).min(n - 1);
            let row = &ptlist[i];
            let rl = row.len();
            let mut a = Vec::new();
            let mut b = Vec::new();
            let mut j = 0;
            while (j as f64) < js::min(rl as f64 / 8.0, 10.0) {
                let nz = ctx.noise(j as f64 * 0.1, i as f64, 0.0);
                a.push([row[j][0] + nz * 10.0, row[j][1]]);
                let nz = ctx.noise(j as f64 * 0.1, i as f64, 0.0);
                b.push([row[rl - 1 - j][0] - nz * 10.0, row[rl - 1 - j][1]]);
                j += 1;
            }
            a.reverse();
            b.reverse();
            for j in 0..span {
                let p = j as f64 / span as f64;
                let x1 = ptlist[i][0][0] * (1.0 - p) + ptlist[ni][0][0] * p;
                let mut y1 = ptlist[i][0][1] * (1.0 - p) + ptlist[ni][0][1] * p;
                let x2 = ptlist[i][rl - 1][0] * (1.0 - p) + ptlist[ni][rl - 1][0] * p;
                let mut y2 = ptlist[i][rl - 1][1] * (1.0 - p) + ptlist[ni][rl - 1][1] * p;
                let vib = -1.7 * (p - 1.0) * p.powf(1.0 / 5.0);
                y1 += vib * 5.0 + ctx.noise(xof * 0.05, i as f64, 0.0) * 5.0;
                y2 += vib * 5.0 + ctx.noise(xof * 0.05, i as f64, 0.0) * 5.0;
                a.push([x1, y1]);
                b.push([x2, y2]);
            }
            ftlist.push(a);
            ftlist.push(b);
        }
    }
    let mut canv = Canv::new();
    for f in &ftlist {
        canv.push(poly(f, PolyArgs { xof, yof, fil: Some(WHITE), str: Some(NONE), wid: 0.0 }));
    }
    for f in &ftlist {
        let pts = offset(f, xof, yof);
        let col = ink(js::fixed(0.1 + ctx.rand() * 0.1, 3));
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(col), wid: Some(1.0), ..Default::default() }));
    }
    canv
}

fn vegetate(
    ctx: &mut Ctx,
    canv: &mut Canv,
    ptlist: &[Vec<Pt>],
    tree_fn: &mut dyn FnMut(&mut Ctx, f64, f64) -> Canv,
    growth: &dyn Fn(&mut Ctx, usize, usize) -> bool,
    proof: &dyn Fn(&[Pt], usize) -> bool,
) {
    let mut veglist = Vec::new();
    for i in 0..ptlist.len() {
        for j in 0..ptlist[i].len() {
            if growth(ctx, i, j) {
                veglist.push(ptlist[i][j]);
            }
        }
    }
    for i in 0..veglist.len() {
        if proof(&veglist, i) {
            canv.extend(tree_fn(ctx, veglist[i][0], veglist[i][1]));
        }
    }
}

fn always(_: &[Pt], _: usize) -> bool {
    true
}

fn noise_alpha(ctx: &mut Ctx, x: f64, y: f64, base: f64) -> f64 {
    js::fixed(ctx.noise(0.01 * x, 0.01 * y, 0.0) * 0.5 * 0.3 + base, 3)
}

fn gray(a: f64) -> Rgba {
    Rgba { r: 100, g: 100, b: 100, a }
}

pub fn mountain(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64) -> Canv {
    let hei = 100.0 + ctx.rand() * 400.0;
    let wid = 400.0 + ctx.rand() * 200.0;
    let tex = 200.0;
    let veg = true;

    let mut canv = Canv::new();
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    let h = hei;
    let w = wid;
    let reso = [10, 50];

    let mut hoff = 0.0;
    for j in 0..reso[0] {
        hoff += (ctx.rand() * yoff) / 100.0;
        let mut row = Vec::with_capacity(reso[1]);
        for i in 0..reso[1] {
            let x = (i as f64 / reso[1] as f64 - 0.5) * PI;
            let mut y = x.cos();
            y *= ctx.noise(x + 10.0, j as f64 * 0.15, seed);
            let p = 1.0 - j as f64 / reso[0] as f64;
            row.push([(x / PI) * w * p, -y * h * p + hoff]);
        }
        ptlist.push(row);
    }
    let pl = &ptlist;

    vegetate(
        ctx,
        &mut canv,
        pl,
        &mut |ctx, x, y| {
            let a = noise_alpha(ctx, x, y, 0.5);
            tree::tree02(ctx, x + xoff, y + yoff - 5.0, TreeArgs { col: Some(gray(a)), clu: Some(2.0), ..Default::default() })
        },
        &|ctx, i, j| {
            let ns = ctx.noise(j as f64 * 0.1, seed, 0.0);
            i == 0 && ns * ns * ns < 0.1 && pl[i][j][1].abs() / h > 0.2
        },
        &always,
    );

    let mut bg = ptlist[0].clone();
    bg.push([0.0, reso[0] as f64 * 4.0]);
    canv.push(poly(&bg, PolyArgs { xof: xoff, yof: yoff, fil: Some(WHITE), str: Some(NONE), wid: 0.0 }));
    let pts = offset(&ptlist[0], xoff, yoff);
    canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.3)), noi: Some(1.0), wid: Some(3.0), ..Default::default() }));

    canv.extend(foot(ctx, &ptlist, xoff, yoff));
    let sha = ctx.choice(&[0.0, 0.0, 0.0, 0.0, 5.0]);
    canv.extend(texture(ctx, &ptlist, TexArgs { xof: xoff, yof: yoff, tex: Some(tex), sha: Some(sha), ..Default::default() }));

    vegetate(
        ctx,
        &mut canv,
        pl,
        &mut |ctx, x, y| {
            let a = noise_alpha(ctx, x, y, 0.5);
            tree::tree02(ctx, x + xoff, y + yoff, TreeArgs { col: Some(gray(a)), ..Default::default() })
        },
        &|ctx, i, j| {
            let ns = ctx.noise(i as f64 * 0.1, j as f64 * 0.1, seed + 2.0);
            ns * ns * ns < 0.1 && pl[i][j][1].abs() / h > 0.5
        },
        &always,
    );

    if veg {
        vegetate(
            ctx,
            &mut canv,
            pl,
            &mut |ctx, x, y| {
                let mut ht = ((h + y) / h) * 70.0;
                ht = ht * 0.3 + ctx.rand() * ht * 0.7;
                let wid = ctx.rand() * 3.0 + 1.0;
                let a = noise_alpha(ctx, x, y, 0.3);
                tree::tree01(ctx, x + xoff, y + yoff, TreeArgs { hei: Some(ht), wid: Some(wid), col: Some(gray(a)), ..Default::default() })
            },
            &|ctx, i, j| {
                let ns = ctx.noise(i as f64 * 0.2, j as f64 * 0.05, seed);
                j % 2 == 1 && ns * ns * ns * ns < 0.012 && pl[i][j][1].abs() / h < 0.3
            },
            &|veglist, i| {
                let mut counter = 0;
                for j in 0..veglist.len() {
                    if i != j && sq(veglist[i][0] - veglist[j][0]) + sq(veglist[i][1] - veglist[j][1]) < 30.0 * 30.0 {
                        counter += 1;
                    }
                    if counter > 2 {
                        return true;
                    }
                }
                false
            },
        );

        vegetate(
            ctx,
            &mut canv,
            pl,
            &mut |ctx, x, y| {
                let mut ht = ((h + y) / h) * 120.0;
                ht = ht * 0.5 + ctx.rand() * ht * 0.5;
                let bc = ctx.rand() * 0.1;
                let bp = 1.0;
                let ben = move |x: f64| (x * bc).powf(bp);
                let a = noise_alpha(ctx, x, y, 0.3);
                tree::tree03(ctx, x + xoff, y + yoff, TreeArgs { hei: Some(ht), ben: Some(&ben), col: Some(gray(a)), ..Default::default() })
            },
            &|ctx, i, j| {
                let ns = ctx.noise(i as f64 * 0.2, j as f64 * 0.05, seed);
                (j == 0 || j == pl[i].len() - 1) && ns * ns * ns * ns < 0.012
            },
            &always,
        );
    }

    vegetate(
        ctx,
        &mut canv,
        pl,
        &mut |ctx, x, y| {
            let tt = ctx.choice(&[0, 0, 1, 1, 1, 2]);
            if tt == 1 {
                let wid = ctx.norm_rand(40.0, 70.0);
                let sto = ctx.choice(&[1.0, 2.0, 2.0, 3.0]);
                let rot = ctx.rand();
                let sty = ctx.choice(&[1, 2, 3]);
                arch::arch02(ctx, x + xoff, y + yoff, arch::Arch02Args { wid: Some(wid), sto: Some(sto), rot: Some(rot), sty: Some(sty), ..Default::default() })
            } else if tt == 2 {
                let sto = ctx.choice(&[1.0, 1.0, 1.0, 2.0, 2.0]);
                arch::arch04(ctx, x + xoff, y + yoff, sto)
            } else {
                Canv::new()
            }
        },
        &|ctx, i, j| {
            let ns = ctx.noise(i as f64 * 0.2, j as f64 * 0.05, seed + 10.0);
            i != 0 && (j == 1 || j == pl[i].len() - 2) && ns * ns * ns * ns < 0.008
        },
        &always,
    );

    vegetate(
        ctx,
        &mut canv,
        pl,
        &mut |ctx, x, y| {
            let sto = ctx.choice(&[5.0, 7.0]);
            let wid = 40.0 + ctx.rand() * 20.0;
            arch::arch03(ctx, x + xoff, y + yoff, sto, wid)
        },
        &|ctx, i, j| i == 1 && (j as f64 - pl[i].len() as f64 / 2.0).abs() < 1.0 && ctx.rand() < 0.02,
        &always,
    );

    vegetate(
        ctx,
        &mut canv,
        pl,
        // The web version stands power-line pylons here. They are left out, but still
        // drawn and thrown away so a seed paints the same landscape around them.
        &mut |ctx, x, y| {
            arch::transmission_tower01(ctx, x + xoff, y + yoff);
            Canv::new()
        },
        &|ctx, i, j| {
            let ns = ctx.noise(i as f64 * 0.2, j as f64 * 0.05, seed + 20.0 * PI);
            i % 2 == 0 && (j == 1 || j == pl[i].len() - 2) && ns * ns * ns * ns < 0.002
        },
        &always,
    );

    vegetate(
        ctx,
        &mut canv,
        pl,
        &mut |ctx, x, y| {
            let wid = 20.0 + ctx.rand() * 20.0;
            let hei = 20.0 + ctx.rand() * 20.0;
            rock(ctx, x + xoff, y + yoff, seed, RockArgs { wid: Some(wid), hei: Some(hei), sha: Some(2.0), ..Default::default() })
        },
        &|ctx, i, j| (j == 0 || j == pl[i].len() - 1) && ctx.rand() < 0.1,
        &always,
    );

    canv
}

pub struct FlatArgs {
    pub wid: f64,
    pub hei: f64,
    pub cho: f64,
}

pub fn flat_mount(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64, a: FlatArgs) -> Canv {
    let FlatArgs { wid, hei, cho } = a;
    let tex = 80.0;

    let mut canv = Canv::new();
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    let reso = [5, 50];
    let mut hoff = 0.0;
    let mut flat: Vec<Vec<Pt>> = Vec::new();
    for j in 0..reso[0] {
        hoff += (ctx.rand() * yoff) / 100.0;
        let mut row: Vec<Pt> = Vec::new();
        let mut frow: Vec<Pt> = Vec::new();
        for i in 0..reso[1] {
            let x = (i as f64 / reso[1] as f64 - 0.5) * PI;
            let mut y = (x * 2.0).cos() + 1.0;
            y *= ctx.noise(x + 10.0, j as f64 * 0.1, seed);
            let p = 1.0 - (j as f64 / reso[0] as f64) * 0.6;
            let nx = (x / PI) * wid * p;
            let mut ny = -y * hei * p + hoff;
            let h = 100.0;
            if ny < -h * cho + hoff {
                ny = -h * cho + hoff;
                if frow.len() % 2 == 0 {
                    frow.push([nx, ny]);
                }
            } else if frow.len() % 2 == 1 {
                frow.push(row[row.len() - 1]);
            }
            row.push([nx, ny]);
        }
        ptlist.push(row);
        flat.push(frow);
    }

    let mut bg = ptlist[0].clone();
    bg.push([0.0, reso[0] as f64 * 4.0]);
    canv.push(poly(&bg, PolyArgs { xof: xoff, yof: yoff, fil: Some(WHITE), str: Some(NONE), wid: 0.0 }));
    let pts = offset(&ptlist[0], xoff, yoff);
    canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.3)), noi: Some(1.0), wid: Some(3.0), ..Default::default() }));

    let dis = |ctx: &mut Ctx| if ctx.rand() > 0.5 { 0.1 + 0.4 * ctx.rand() } else { 0.9 - 0.4 * ctx.rand() };
    canv.extend(texture(ctx, &ptlist, TexArgs { xof: xoff, yof: yoff, tex: Some(tex), wid: Some(2.0), dis: Some(&dis), ..Default::default() }));

    let mut grlist1 = Vec::new();
    let mut grlist2 = Vec::new();
    let mut i = 0;
    while i < flat.len() {
        if flat[i].len() >= 2 {
            grlist1.push(flat[i][0]);
            grlist2.push(flat[i][flat[i].len() - 1]);
        }
        i += 2;
    }
    if grlist1.is_empty() {
        return canv;
    }
    let wb = [grlist1[0][0], grlist2[0][0]];
    for i in 0..3 {
        let p = 0.8 - i as f64 * 0.2;
        grlist1.insert(0, [wb[0] * p, grlist1[0][1] - 5.0]);
        grlist2.insert(0, [wb[1] * p, grlist2[0][1] - 5.0]);
    }
    let wb = [grlist1[grlist1.len() - 1][0], grlist2[grlist2.len() - 1][0]];
    for i in 0..3 {
        let p = 0.6 - (i * i) as f64 * 0.1;
        grlist1.push([wb[0] * p, grlist1[grlist1.len() - 1][1] + 1.0]);
        grlist2.push([wb[1] * p, grlist2[grlist2.len() - 1][1] + 1.0]);
    }
    let d = 5.0;
    let mut grlist1 = div(&grlist1, d);
    let grlist2 = div(&grlist2, d);
    grlist1.reverse();
    let mut grlist = grlist1.clone();
    grlist.extend_from_slice(&grlist2);
    grlist.push(grlist1[0]);
    let last = grlist.len() - 1;
    for i in 0..grlist.len() {
        if i == last {
            grlist[last][0] = grlist[0][0];
        }
        let v = (1.0 - ((i as f64 % d) - d / 2.0).abs() / (d / 2.0)) * 0.12;
        let f = 1.0 - v + ctx.noise(grlist[i][1] * 0.5, 0.0, 0.0) * v;
        grlist[i][0] *= f;
    }
    grlist[0][0] = grlist[last][0];

    canv.push(poly(&grlist, PolyArgs { xof: xoff, yof: yoff, str: Some(NONE), fil: Some(WHITE), wid: 2.0 }));
    let pts = offset(&grlist, xoff, yoff);
    canv.extend(stroke(ctx, &pts, StrokeArgs { wid: Some(3.0), col: Some(ink(0.2)), ..Default::default() }));

    let mut bd: [Option<f64>; 4] = [None; 4];
    for p in &grlist {
        if bd[0].is_none() || p[0] < bd[0].unwrap() {
            bd[0] = Some(p[0]);
        }
        if bd[1].is_none() || p[0] > bd[1].unwrap() {
            bd[1] = Some(p[0]);
        }
        if bd[2].is_none() || p[1] < bd[2].unwrap() {
            bd[2] = Some(p[1]);
        }
        if bd[3].is_none() || p[1] > bd[3].unwrap() {
            bd[3] = Some(p[1]);
        }
    }
    let grbd = Bound { xmin: bd[0].unwrap(), xmax: bd[1].unwrap(), ymin: bd[2].unwrap(), ymax: bd[3].unwrap() };
    canv.extend(flat_dec(ctx, xoff, yoff, grbd));
    canv
}

#[derive(Clone, Copy)]
struct Bound {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

fn flat_dec(ctx: &mut Ctx, xoff: f64, yoff: f64, g: Bound) -> Canv {
    let mut canv = Canv::new();
    let tt = ctx.choice(&[0, 0, 1, 2, 3, 4]);
    let ymid = (g.ymin + g.ymax) / 2.0;

    let mut j = 0.0;
    while j < ctx.rand() * 5.0 {
        let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
        let y = yoff + ymid + ctx.norm_rand(-10.0, 10.0) + 10.0;
        let seed = ctx.rand() * 100.0;
        let wid = 10.0 + ctx.rand() * 20.0;
        let hei = 10.0 + ctx.rand() * 20.0;
        canv.extend(rock(ctx, x, y, seed, RockArgs { wid: Some(wid), hei: Some(hei), sha: Some(2.0), ..Default::default() }));
        j += 1.0;
    }
    let mut j = 0.0;
    while j < ctx.choice(&[0.0, 0.0, 1.0, 2.0]) {
        let xr = xoff + ctx.norm_rand(g.xmin, g.xmax);
        let yr = yoff + ymid + ctx.norm_rand(-5.0, 5.0) + 20.0;
        let mut k = 0.0;
        while k < 2.0 + ctx.rand() * 3.0 {
            let x = xr + js::min(js::max(ctx.norm_rand(-30.0, 30.0), g.xmin), g.xmax);
            let hei = 60.0 + ctx.rand() * 40.0;
            canv.extend(tree::tree08(ctx, x, yr, TreeArgs { hei: Some(hei), ..Default::default() }));
            k += 1.0;
        }
        j += 1.0;
    }

    let big_rock = |ctx: &mut Ctx, canv: &mut Canv, x: f64, y: f64, seed: f64| {
        let wid = 50.0 + ctx.rand() * 20.0;
        let hei = 40.0 + ctx.rand() * 20.0;
        canv.extend(rock(ctx, x, y, seed, RockArgs { wid: Some(wid), hei: Some(hei), sha: Some(5.0), ..Default::default() }));
    };

    if tt == 0 {
        let mut j = 0.0;
        while j < ctx.rand() * 3.0 {
            let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
            let y = yoff + ymid + ctx.norm_rand(-5.0, 5.0) + 20.0;
            let seed = ctx.rand() * 100.0;
            big_rock(ctx, &mut canv, x, y, seed);
            j += 1.0;
        }
    }
    if tt == 1 {
        let pmin = ctx.rand() * 0.5;
        let pmax = ctx.rand() * 0.5 + 0.5;
        let xmin = g.xmin * (1.0 - pmin) + g.xmax * pmin;
        let xmax = g.xmin * (1.0 - pmax) + g.xmax * pmax;
        let mut i = xmin;
        while i < xmax {
            let x = xoff + i + 20.0 * ctx.norm_rand(-1.0, 1.0);
            let hei = 100.0 + ctx.rand() * 200.0;
            canv.extend(tree::tree05(ctx, x, yoff + ymid + 20.0, TreeArgs { hei: Some(hei), ..Default::default() }));
            i += 30.0;
        }
        let mut j = 0.0;
        while j < ctx.rand() * 4.0 {
            let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
            let y = yoff + ymid + ctx.norm_rand(-5.0, 5.0) + 20.0;
            let seed = ctx.rand() * 100.0;
            big_rock(ctx, &mut canv, x, y, seed);
            j += 1.0;
        }
    } else if tt == 2 {
        let mut i = 0.0;
        while i < ctx.choice(&[1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0]) {
            let xr = ctx.norm_rand(g.xmin, g.xmax);
            let yr = ymid;
            canv.extend(tree::tree04(ctx, xoff + xr, yoff + yr + 20.0, TreeArgs::default()));
            let mut j = 0.0;
            while j < ctx.rand() * 2.0 {
                let x = xoff + js::max(g.xmin, js::min(g.xmax, xr + ctx.norm_rand(-50.0, 50.0)));
                let y = yoff + yr + ctx.norm_rand(-5.0, 5.0) + 20.0;
                let seed = j * i * ctx.rand() * 100.0;
                big_rock(ctx, &mut canv, x, y, seed);
                j += 1.0;
            }
            i += 1.0;
        }
    } else if tt == 3 {
        let mut i = 0.0;
        while i < ctx.choice(&[1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0]) {
            let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
            let hei = 60.0 + ctx.rand() * 60.0;
            canv.extend(tree::tree06(ctx, x, yoff + ymid, TreeArgs { hei: Some(hei), ..Default::default() }));
            i += 1.0;
        }
    } else if tt == 4 {
        let pmin = ctx.rand() * 0.5;
        let pmax = ctx.rand() * 0.5 + 0.5;
        let xmin = g.xmin * (1.0 - pmin) + g.xmax * pmin;
        let xmax = g.xmin * (1.0 - pmax) + g.xmax * pmax;
        let mut i = xmin;
        while i < xmax {
            let x = xoff + i + 20.0 * ctx.norm_rand(-1.0, 1.0);
            let y = yoff + ymid + ctx.norm_rand(-1.0, 1.0) + 0.0;
            let hei = ctx.norm_rand(40.0, 80.0);
            canv.extend(tree::tree07(ctx, x, y, TreeArgs { hei: Some(hei), ..Default::default() }));
            i += 20.0;
        }
    }

    let mut i = 0.0;
    while i < 50.0 * ctx.rand() {
        let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
        let y = yoff + ctx.norm_rand(g.ymin, g.ymax);
        canv.extend(tree::tree02(ctx, x, y, TreeArgs::default()));
        i += 1.0;
    }

    let ts = ctx.choice(&[0, 0, 0, 0, 1]);
    if ts == 1 && tt != 4 {
        let x = xoff + ctx.norm_rand(g.xmin, g.xmax);
        let y = yoff + ymid + 20.0;
        let seed = ctx.rand();
        let wid = ctx.norm_rand(160.0, 200.0);
        let hei = ctx.norm_rand(80.0, 100.0);
        let per = ctx.rand();
        canv.extend(arch::arch01(ctx, x, y, seed, wid, hei, per));
    }
    canv
}

pub struct DistArgs {
    pub hei: f64,
    pub len: f64,
}

pub fn dist_mount(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64, a: DistArgs) -> Canv {
    let DistArgs { hei, len } = a;
    let seg = 5.0;
    let span = 10.0;
    let mut canv = Canv::new();
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();

    let mut i = 0.0;
    while i < len / span / seg {
        let mut row: Vec<Pt> = Vec::new();
        let mut j = 0.0;
        while j < seg + 1.0 {
            let k = i * seg + j;
            let s = ((PI * k) / (len / span)).sin().powf(0.5);
            row.push([xoff + k * span, yoff - hei * ctx.noise(k * 0.05, seed, 0.0) * s]);
            j += 1.0;
        }
        let mut j = 0.0;
        while j < seg / 2.0 + 1.0 {
            let k = i * seg + j * 2.0;
            let s = ((PI * k) / (len / span)).sin().powf(1.0);
            row.insert(0, [xoff + k * span, yoff + 24.0 * ctx.noise(k * 0.05, 2.0, seed) * s]);
            j += 1.0;
        }
        ptlist.push(row);
        i += 1.0;
    }
    let get_col = |ctx: &mut Ctx, x: f64, y: f64| {
        let c = js::to_int32(ctx.noise(x * 0.02, y * 0.02, yoff) * 55.0 + 200.0).clamp(0, 255) as u8;
        rgba(c, c, c, 1.0)
    };
    for row in &ptlist {
        let last = row[row.len() - 1];
        let fil = get_col(ctx, last[0], last[1]);
        canv.push(poly(row, PolyArgs { fil: Some(fil), str: Some(NONE), wid: 1.0, ..Default::default() }));
        for tri in triangulate(row, 100.0, false) {
            let m = mid_pt(&tri);
            let co = get_col(ctx, m[0], m[1]);
            canv.push(poly(&tri, PolyArgs { fil: Some(co), str: Some(co), wid: 1.0, ..Default::default() }));
        }
    }
    canv
}

#[derive(Default, Clone, Copy)]
pub struct RockArgs {
    pub hei: Option<f64>,
    pub wid: Option<f64>,
    pub tex: Option<f64>,
    pub sha: Option<f64>,
}

pub fn rock(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64, a: RockArgs) -> Canv {
    let hei = a.hei.unwrap_or(80.0);
    let wid = a.wid.unwrap_or(100.0);
    let tex = a.tex.unwrap_or(40.0);
    let sha = a.sha.unwrap_or(10.0);

    let mut canv = Canv::new();
    let reso = [10, 50];
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    for i in 0..reso[0] {
        let mut nslist: Vec<f64> = (0..reso[1]).map(|j| ctx.noise(i as f64, j as f64 * 0.2, seed)).collect();
        loop_noise(&mut nslist);
        let mut row = Vec::with_capacity(reso[1]);
        for j in 0..reso[1] {
            let a = (j as f64 / reso[1] as f64) * PI * 2.0 - PI / 2.0;
            let mut l = (wid * hei) / (sq(hei * a.cos()) + sq(wid * a.sin())).sqrt();
            l *= 0.7 + 0.3 * nslist[j];
            let p = 1.0 - i as f64 / reso[0] as f64;
            let nx = a.cos() * l * p;
            let mut ny = -a.sin() * l * p;
            if PI < a || a < 0.0 {
                ny *= 0.2;
            }
            ny += hei * (i as f64 / reso[0] as f64) * 0.2;
            row.push([nx, ny]);
        }
        ptlist.push(row);
    }

    let mut bg = ptlist[0].clone();
    bg.push([0.0, 0.0]);
    canv.push(poly(&bg, PolyArgs { xof: xoff, yof: yoff, fil: Some(WHITE), str: Some(NONE), wid: 0.0 }));
    let pts = offset(&ptlist[0], xoff, yoff);
    canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.3)), noi: Some(1.0), wid: Some(3.0), ..Default::default() }));
    let col = |ctx: &mut Ctx, _: f64| rgba(180, 180, 180, js::fixed(0.3 + ctx.rand() * 0.3, 3));
    let dis = |ctx: &mut Ctx| if ctx.rand() > 0.5 { 0.15 + 0.15 * ctx.rand() } else { 0.85 - 0.15 * ctx.rand() };
    canv.extend(texture(
        ctx,
        &ptlist,
        TexArgs { xof: xoff, yof: yoff, tex: Some(tex), wid: Some(3.0), sha: Some(sha), col: Some(&col), dis: Some(&dis), ..Default::default() },
    ));
    canv
}
