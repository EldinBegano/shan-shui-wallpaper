use super::*;
use std::f64::consts::PI;

fn expand(ctx: &mut Ctx, ptlist: &[Pt], wfun: &dyn Fn(f64) -> f64) -> (Vec<Pt>, Vec<Pt>) {
    let mut vtx0 = Vec::new();
    let mut vtx1 = Vec::new();
    let _n0 = ctx.rand() * 10.0;
    let n = ptlist.len();
    for i in 1..n - 1 {
        let w = wfun(i as f64 / n as f64);
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
    let l = n - 1;
    let a0 = (ptlist[1][1] - ptlist[0][1]).atan2(ptlist[1][0] - ptlist[0][0]) - PI / 2.0;
    let a1 = (ptlist[l][1] - ptlist[l - 1][1]).atan2(ptlist[l][0] - ptlist[l - 1][0]) - PI / 2.0;
    let w0 = wfun(0.0);
    let w1 = wfun(1.0);
    vtx0.insert(0, [ptlist[0][0] + w0 * a0.cos(), ptlist[0][1] + w0 * a0.sin()]);
    vtx1.insert(0, [ptlist[0][0] - w0 * a0.cos(), ptlist[0][1] - w0 * a0.sin()]);
    vtx0.push([ptlist[l][0] + w1 * a1.cos(), ptlist[l][1] + w1 * a1.sin()]);
    vtx1.push([ptlist[l][0] - w1 * a1.cos(), ptlist[l][1] - w1 * a1.sin()]);
    (vtx0, vtx1)
}

fn tranpoly(p0: Pt, p1: Pt, ptlist: &[Pt]) -> Vec<Pt> {
    let plist: Vec<Pt> = ptlist.iter().map(|v| [-v[0], v[1]]).collect();
    let ang = (p1[1] - p0[1]).atan2(p1[0] - p0[0]) - PI / 2.0;
    let scl = distance(p0, p1);
    plist
        .iter()
        .map(|&v| {
            let d = distance(v, [0.0, 0.0]);
            let a = v[1].atan2(v[0]);
            [p0[0] + d * scl * (ang + a).cos(), p0[1] + d * scl * (ang + a).sin()]
        })
        .collect()
}

fn flipper(fli: bool, plist: Vec<Pt>) -> Vec<Pt> {
    if fli { plist.iter().map(|v| [-v[0], v[1]]).collect() } else { plist }
}

#[derive(Clone, Copy)]
pub enum Hat {
    Hat01,
    Hat02,
}

#[derive(Clone, Copy)]
pub enum Item {
    Stick01,
}

fn hat01(ctx: &mut Ctx, p0: Pt, p1: Pt, fli: bool) -> Canv {
    let mut canv = Canv::new();
    let seed = ctx.rand();
    let shape = vec![[-0.3, 0.5], [0.3, 0.8], [0.2, 1.0], [0.0, 1.1], [-0.3, 1.15], [-0.55, 1.0], [-0.65, 0.5]];
    canv.push(poly(&tranpoly(p0, p1, &flipper(fli, shape)), PolyArgs { fil: Some(ink(0.8)), ..Default::default() }));
    let mut qlist1 = Vec::new();
    for i in 0..10 {
        let fi = i as f64;
        qlist1.push([-0.3 - ctx.noise(fi * 0.2, seed, 0.0) * fi * 0.1, 0.5 - fi * 0.3]);
    }
    canv.push(poly(&tranpoly(p0, p1, &flipper(fli, qlist1)), PolyArgs { str: Some(ink(0.8)), wid: 1.0, ..Default::default() }));
    canv
}

fn hat02(ctx: &mut Ctx, p0: Pt, p1: Pt, fli: bool) -> Canv {
    let _seed = ctx.rand();
    let shape = vec![
        [-0.3, 0.5],
        [-1.1, 0.5],
        [-1.2, 0.6],
        [-1.1, 0.7],
        [-0.3, 0.8],
        [0.3, 0.8],
        [1.0, 0.7],
        [1.3, 0.6],
        [1.2, 0.5],
        [0.3, 0.5],
    ];
    vec![poly(&tranpoly(p0, p1, &flipper(fli, shape)), PolyArgs { fil: Some(ink(0.8)), ..Default::default() })]
}

fn stick01(ctx: &mut Ctx, p0: Pt, p1: Pt, fli: bool) -> Canv {
    let seed = ctx.rand();
    let mut qlist1 = Vec::new();
    let l = 12.0;
    for i in 0..12 {
        let fi = i as f64;
        qlist1.push([-ctx.noise(fi * 0.1, seed, 0.0) * 0.1 * ((fi / l) * PI).sin() * 5.0, 0.0 + fi * 0.3]);
    }
    vec![poly(&tranpoly(p0, p1, &flipper(fli, qlist1)), PolyArgs { str: Some(ink(0.5)), wid: 1.0, ..Default::default() })]
}

#[derive(Default)]
pub struct ManArgs {
    pub sca: Option<f64>,
    pub hat: Option<Hat>,
    pub ite: Option<Item>,
    pub fli: Option<bool>,
    pub len: Option<[f64; 9]>,
}

const PAR: [&[usize]; 9] = [&[0], &[0, 1], &[0, 1, 2], &[0, 3], &[0, 3, 4], &[0, 1, 5], &[0, 1, 5, 6], &[0, 1, 7], &[0, 1, 7, 8]];

pub fn man(ctx: &mut Ctx, xoff: f64, yoff: f64, a: ManArgs) -> Canv {
    let sca = a.sca.unwrap_or(0.5);
    let hat = a.hat.unwrap_or(Hat::Hat01);
    let ite = a.ite;
    let fli = a.fli.unwrap_or(true);
    let r2 = ctx.norm_rand(0.0, 0.0);
    let r3 = (PI / 4.0) * ctx.rand();
    let r4 = ((PI * 3.0) / 4.0) * ctx.rand();
    let r7 = (-PI * 3.0) / 4.0 - (PI / 4.0) * ctx.rand();
    let ang = [0.0, -PI / 2.0, r2, r3, r4, (PI * 3.0) / 4.0, -PI / 4.0, r7, -PI / 4.0];
    let len = a.len.unwrap_or([0.0, 30.0, 20.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0]).map(|v| v * sca);

    let mut canv = Canv::new();
    let mut yoff = yoff;
    let grot = |ind: usize| PAR[ind].iter().fold(0.0, |rot, &p| rot + ang[p]);
    let gpos = |ind: usize| {
        let mut pos = [0.0, 0.0];
        for &p in PAR[ind] {
            let a = grot(p);
            pos[0] += len[p] * a.cos();
            pos[1] += len[p] * a.sin();
        }
        pos
    };
    let pts: Vec<Pt> = (0..9).map(gpos).collect();
    yoff -= pts[4][1];
    let to_global = |v: Pt| [(if fli { -1.0 } else { 1.0 }) * v[0] + xoff, v[1] + yoff];
    let glob = |l: &[Pt]| l.iter().map(|&v| to_global(v)).collect::<Vec<Pt>>();

    let cloth = |ctx: &mut Ctx, canv: &mut Canv, plist: &[Pt], fun: &dyn Fn(f64) -> f64| {
        let tlist = bezmh(plist, 2.0);
        let (tlist1, mut tlist2) = expand(ctx, &tlist, fun);
        tlist2.reverse();
        let outline: Vec<Pt> = tlist1.iter().chain(tlist2.iter()).copied().collect();
        canv.push(poly(&glob(&outline), PolyArgs { fil: Some(WHITE), ..Default::default() }));
        canv.extend(stroke(ctx, &glob(&tlist1), StrokeArgs { wid: Some(1.0), col: Some(ink(0.5)), ..Default::default() }));
        canv.extend(stroke(ctx, &glob(&tlist2), StrokeArgs { wid: Some(1.0), col: Some(ink(0.6)), ..Default::default() }));
    };

    let fsleeve = |x: f64| sca * 8.0 * ((0.5 * x * PI).sin() * (x * PI).sin().powf(0.1) + (1.0 - x) * 0.4);
    let fbody = |x: f64| sca * 11.0 * ((0.5 * x * PI).sin() * (x * PI).sin().powf(0.1) + (1.0 - x) * 0.5);
    let fhead = |x: f64| sca * 7.0 * (0.25 - sq(x - 0.5)).powf(0.3);

    if let Some(Item::Stick01) = ite {
        canv.extend(stick01(ctx, to_global(pts[8]), to_global(pts[6]), fli));
    }

    cloth(ctx, &mut canv, &[pts[1], pts[7], pts[8]], &fsleeve);
    cloth(ctx, &mut canv, &[pts[1], pts[0], pts[3], pts[4]], &fbody);
    cloth(ctx, &mut canv, &[pts[1], pts[5], pts[6]], &fsleeve);
    cloth(ctx, &mut canv, &[pts[1], pts[2]], &fhead);

    let hlist = bezmh(&[pts[1], pts[2]], 2.0);
    let (mut hlist1, mut hlist2) = expand(ctx, &hlist, &fhead);
    hlist1.drain(0..(hlist1.len() as f64 * 0.1).floor() as usize);
    hlist2.drain(0..(hlist2.len() as f64 * 0.95).floor() as usize);
    let head: Vec<Pt> = hlist1.iter().chain(hlist2.iter().rev()).copied().collect();
    canv.push(poly(&glob(&head), PolyArgs { fil: Some(ink(0.6)), ..Default::default() }));

    let (h0, h1) = (to_global(pts[1]), to_global(pts[2]));
    canv.extend(match hat {
        Hat::Hat01 => hat01(ctx, h0, h1, fli),
        Hat::Hat02 => hat02(ctx, h0, h1, fli),
    });
    canv
}
