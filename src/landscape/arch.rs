use super::man::{self, Hat, Item, ManArgs};
use super::*;
use crate::js;

fn hut(ctx: &mut Ctx, xoff: f64, yoff: f64, hei: f64, wid: f64) -> Canv {
    let tex = 300.0;
    let reso = [10, 10];
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    for i in 0..reso[0] {
        let heir = hei + hei * 0.2 * ctx.rand();
        let mut row = Vec::new();
        for j in 0..reso[1] {
            let nx = wid * (i as f64 / (reso[0] - 1) as f64 - 0.5) * (j as f64 / (reso[1] - 1) as f64).powf(0.7);
            let ny = heir * (j as f64 / (reso[1] - 1) as f64);
            row.push([nx, ny]);
        }
        ptlist.push(row);
    }
    let mut canv = Canv::new();
    let first = &ptlist[0];
    let last = &ptlist[ptlist.len() - 1];
    let outline: Vec<Pt> = first[..first.len() - 1].iter().chain(last[..last.len() - 1].iter().rev()).copied().collect();
    canv.push(poly(&outline, PolyArgs { xof: xoff, yof: yoff, fil: Some(WHITE), str: Some(NONE), wid: 0.0 }));
    canv.push(poly(first, PolyArgs { xof: xoff, yof: yoff, fil: Some(NONE), str: Some(ink(0.3)), wid: 2.0 }));
    canv.push(poly(last, PolyArgs { xof: xoff, yof: yoff, fil: Some(NONE), str: Some(ink(0.3)), wid: 2.0 }));
    let col = |ctx: &mut Ctx, _: f64| rgba(120, 120, 120, js::fixed(0.3 + ctx.rand() * 0.3, 3));
    let dis = |ctx: &mut Ctx| ctx.wtrand(|a| a * a);
    let noi = |_: f64| 5.0;
    canv.extend(texture(
        ctx,
        &ptlist,
        TexArgs { xof: xoff, yof: yoff, tex: Some(tex), wid: Some(1.0), len: Some(0.25), col: Some(&col), dis: Some(&dis), noi: Some(&noi), ..Default::default() },
    ));
    canv
}

struct Deco {
    style: u8,
    hsp: [f64; 2],
    vsp: [f64; 2],
}

#[derive(Clone, Copy)]
struct Corners {
    pul: Pt,
    pur: Pt,
    pdl: Pt,
    pdr: Pt,
}

fn deco(d: &Deco, c: Corners) -> Vec<Vec<Pt>> {
    let Corners { pul, pur, pdl, pdr } = c;
    let (hsp, vsp) = (d.hsp, d.vsp);
    let mut plist = Vec::new();
    let dl = div(&[pul, pdl], vsp[1]);
    let dr = div(&[pur, pdr], vsp[1]);
    let du = div(&[pul, pur], hsp[1]);
    let dd = div(&[pdl, pdr], hsp[1]);
    let h0 = hsp[0] as usize;
    let v0 = vsp[0] as usize;

    if d.style == 1 || d.style == 3 {
        let mlu = du[h0];
        let mru = du[du.len() - 1 - h0];
        let mld = dd[h0];
        let mrd = dd[du.len() - 1 - h0];
        let mut i = v0;
        while i < dl.len() - v0 {
            let mml = div(&[mlu, mld], vsp[1])[i];
            let mmr = div(&[mru, mrd], vsp[1])[i];
            if d.style == 1 {
                plist.push(div(&[mml, dl[i]], 5.0));
                plist.push(div(&[mmr, dr[i]], 5.0));
            } else {
                let mmu = div(&[mlu, mru], vsp[1])[i];
                let mmd = div(&[mld, mrd], vsp[1])[i];
                plist.push(div(&[mml, mmr], 5.0));
                plist.push(div(&[mmu, mmd], 5.0));
            }
            i += v0;
        }
        plist.push(div(&[mlu, mld], 5.0));
        plist.push(div(&[mru, mrd], 5.0));
    } else if d.style == 2 {
        let mut i = h0;
        while i < du.len() - h0 {
            plist.push(div(&[du[i], dd[i]], 5.0));
            i += h0;
        }
    }
    plist
}

struct BoxArgs<'a> {
    hei: f64,
    wid: f64,
    rot: f64,
    per: f64,
    tra: bool,
    bot: bool,
    wei: f64,
    dec: Option<&'a Deco>,
}

impl Default for BoxArgs<'_> {
    fn default() -> Self {
        BoxArgs { hei: 20.0, wid: 120.0, rot: 0.7, per: 4.0, tra: true, bot: true, wei: 3.0, dec: None }
    }
}

fn box_(ctx: &mut Ctx, xoff: f64, yoff: f64, a: BoxArgs) -> Canv {
    let BoxArgs { hei, wid, rot, per, tra, bot, wei, dec } = a;
    let mid = -wid * 0.5 + wid * rot;
    let bmid = -wid * 0.5 + wid * (1.0 - rot);
    let mut ptlist = Vec::new();
    ptlist.push(div(&[[-wid * 0.5, -hei], [-wid * 0.5, 0.0]], 5.0));
    ptlist.push(div(&[[wid * 0.5, -hei], [wid * 0.5, 0.0]], 5.0));
    if bot {
        ptlist.push(div(&[[-wid * 0.5, 0.0], [mid, per]], 5.0));
        ptlist.push(div(&[[wid * 0.5, 0.0], [mid, per]], 5.0));
    }
    ptlist.push(div(&[[mid, -hei], [mid, per]], 5.0));
    if tra {
        if bot {
            ptlist.push(div(&[[-wid * 0.5, 0.0], [bmid, -per]], 5.0));
            ptlist.push(div(&[[wid * 0.5, 0.0], [bmid, -per]], 5.0));
        }
        ptlist.push(div(&[[bmid, -hei], [bmid, -per]], 5.0));
    }
    let surf = ((rot < 0.5) as u8 as f64) * 2.0 - 1.0;
    if let Some(d) = dec {
        ptlist.extend(deco(
            d,
            Corners { pul: [surf * wid * 0.5, -hei], pur: [mid, -hei + per], pdl: [surf * wid * 0.5, 0.0], pdr: [mid, per] },
        ));
    }
    let polist = [[-wid * 0.5, -hei], [wid * 0.5, -hei], [wid * 0.5, 0.0], [mid, per], [-wid * 0.5, 0.0]];
    let mut canv = Canv::new();
    if !tra {
        canv.push(poly(&polist, PolyArgs { xof: xoff, yof: yoff, str: Some(NONE), fil: Some(WHITE), wid: 0.0 }));
    }
    let one = |_: f64| 1.0;
    for p in &ptlist {
        let pts = offset(p, xoff, yoff);
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.4)), noi: Some(1.0), wid: Some(wei), fun: Some(&one), ..Default::default() }));
    }
    canv
}

struct RailArgs {
    hei: f64,
    wid: f64,
    rot: f64,
    per: f64,
    seg: f64,
    wei: f64,
    tra: bool,
    fro: bool,
}

impl Default for RailArgs {
    fn default() -> Self {
        RailArgs { hei: 20.0, wid: 180.0, rot: 0.7, per: 4.0, seg: 4.0, wei: 1.0, tra: true, fro: true }
    }
}

fn rail(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64, a: RailArgs) -> Canv {
    let RailArgs { hei, wid, rot, per, seg, wei, tra, fro } = a;
    let mid = -wid * 0.5 + wid * rot;
    let bmid = -wid * 0.5 + wid * (1.0 - rot);
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    if fro {
        ptlist.push(div(&[[-wid * 0.5, 0.0], [mid, per]], seg));
        ptlist.push(div(&[[mid, per], [wid * 0.5, 0.0]], seg));
    }
    if tra {
        ptlist.push(div(&[[-wid * 0.5, 0.0], [bmid, -per]], seg));
        ptlist.push(div(&[[bmid, -per], [wid * 0.5, 0.0]], seg));
    }
    if fro {
        ptlist.push(div(&[[-wid * 0.5, -hei], [mid, -hei + per]], seg));
        ptlist.push(div(&[[mid, -hei + per], [wid * 0.5, -hei]], seg));
    }
    if tra {
        ptlist.push(div(&[[-wid * 0.5, -hei], [bmid, -hei - per]], seg));
        ptlist.push(div(&[[bmid, -hei - per], [wid * 0.5, -hei]], seg));
    }
    let n = ptlist.len();
    if tra {
        let open = (ctx.rand() * n as f64).floor() as usize;
        ptlist[open].pop();
        ptlist[(open + n) % n].pop();
    }
    let mut canv = Canv::new();
    for i in 0..n / 2 {
        for j in 0..ptlist[i].len() {
            ptlist[i][j][1] += (ctx.noise(i as f64, j as f64 * 0.5, seed) - 0.5) * hei;
            let k = (n / 2 + i) % n;
            let kj = j % ptlist[k].len();
            ptlist[k][kj][1] += (ctx.noise(i as f64 + 0.5, j as f64 * 0.5, seed) - 0.5) * hei;
            let mut ln = div(&[ptlist[i][j], ptlist[k][kj]], 2.0);
            ln[0][0] += (ctx.rand() - 0.5) * hei * 0.5;
            canv.push(poly(&ln, PolyArgs { xof: xoff, yof: yoff, fil: Some(NONE), str: Some(ink(0.5)), wid: 2.0 }));
        }
    }
    let one = |_: f64| 1.0;
    for p in &ptlist {
        let pts = offset(p, xoff, yoff);
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.5)), noi: Some(0.5), wid: Some(wei), fun: Some(&one), ..Default::default() }));
    }
    canv
}

fn flip_if(on: bool, mut pts: Vec<Pt>) -> Vec<Pt> {
    if on {
        for p in pts.iter_mut() {
            p[0] = 0.0 - (p[0] - 0.0);
        }
    }
    pts
}

struct RoofArgs {
    hei: f64,
    wid: f64,
    rot: f64,
    per: f64,
    cor: f64,
    wei: f64,
}

fn roof(ctx: &mut Ctx, xoff: f64, yoff: f64, a: RoofArgs) -> Canv {
    let RoofArgs { hei, wid, rot, per, cor, wei } = a;
    let f = rot < 0.5;
    let rrot = if rot < 0.5 { 1.0 - rot } else { rot };
    let mid = -wid * 0.5 + wid * rrot;
    let quat = (mid + wid * 0.5) * 0.5 - mid;

    let mut ptlist = Vec::new();
    ptlist.push(div(&flip_if(f, vec![[-wid * 0.5 + quat, -hei - per / 2.0], [-wid * 0.5 + quat * 0.5, -hei / 2.0 - per / 4.0], [-wid * 0.5 - cor, 0.0]]), 5.0));
    ptlist.push(div(&flip_if(f, vec![[mid + quat, -hei], [(mid + quat + wid * 0.5) / 2.0, -hei / 2.0], [wid * 0.5 + cor, 0.0]]), 5.0));
    ptlist.push(div(&flip_if(f, vec![[mid + quat, -hei], [mid + quat / 2.0, -hei / 2.0 + per / 2.0], [mid + cor, per]]), 5.0));
    ptlist.push(div(&flip_if(f, vec![[-wid * 0.5 - cor, 0.0], [mid + cor, per]]), 5.0));
    ptlist.push(div(&flip_if(f, vec![[wid * 0.5 + cor, 0.0], [mid + cor, per]]), 5.0));
    ptlist.push(div(&flip_if(f, vec![[-wid * 0.5 + quat, -hei - per / 2.0], [mid + quat, -hei]]), 5.0));

    let mut canv = Canv::new();
    let polist = flip_if(f, vec![[-wid * 0.5, 0.0], [-wid * 0.5 + quat, -hei - per / 2.0], [mid + quat, -hei], [wid * 0.5, 0.0], [mid, per]]);
    canv.push(poly(&polist, PolyArgs { xof: xoff, yof: yoff, str: Some(NONE), fil: Some(WHITE), wid: 0.0 }));
    let one = |_: f64| 1.0;
    for p in &ptlist {
        let pts = offset(p, xoff, yoff);
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.4)), noi: Some(1.0), wid: Some(wei), fun: Some(&one), ..Default::default() }));
    }
    canv
}

struct PagroofArgs {
    hei: f64,
    wid: f64,
    per: f64,
    cor: f64,
    sid: f64,
    wei: f64,
}

fn pagroof(ctx: &mut Ctx, xoff: f64, yoff: f64, a: PagroofArgs) -> Canv {
    let PagroofArgs { hei, wid, per, cor, sid, wei } = a;
    let mut ptlist: Vec<Vec<Pt>> = Vec::new();
    let mut polist = vec![[0.0, -hei]];
    let mut canv = Canv::new();
    let mut i = 0.0;
    while i < sid {
        let fx = wid * ((i * 1.0) / (sid - 1.0) - 0.5);
        let fy = per * (1.0 - ((i * 1.0) / (sid - 1.0) - 0.5).abs() * 2.0);
        let fxx = (wid + cor) * ((i * 1.0) / (sid - 1.0) - 0.5);
        if i > 0.0 {
            let prev = ptlist[ptlist.len() - 1][2];
            ptlist.push(vec![prev, [fxx, fy]]);
        }
        ptlist.push(vec![[0.0, -hei], [fx * 0.5, (-hei + fy) * 0.5], [fxx, fy]]);
        polist.push([fxx, fy]);
        i += 1.0;
    }
    canv.push(poly(&polist, PolyArgs { xof: xoff, yof: yoff, str: Some(NONE), fil: Some(WHITE), wid: 0.0 }));
    let one = |_: f64| 1.0;
    for p in &ptlist {
        let pts = offset(&div(p, 5.0), xoff, yoff);
        canv.extend(stroke(ctx, &pts, StrokeArgs { col: Some(ink(0.4)), noi: Some(1.0), wid: Some(wei), fun: Some(&one), ..Default::default() }));
    }
    canv
}

pub fn arch01(ctx: &mut Ctx, xoff: f64, yoff: f64, seed: f64, wid: f64, hei: f64, per: f64) -> Canv {
    let p = 0.4 + ctx.rand() * 0.2;
    let h0 = hei * p;
    let h1 = hei * (1.0 - p);

    let mut canv = Canv::new();
    canv.extend(hut(ctx, xoff, yoff - hei, h0, wid));
    canv.extend(box_(ctx, xoff, yoff, BoxArgs { hei: h1, wid: (wid * 2.0) / 3.0, per, bot: false, ..Default::default() }));

    let seg = js::to_int32(3.0 + ctx.rand() * 3.0) as f64;
    canv.extend(rail(ctx, xoff, yoff, seed, RailArgs { tra: true, fro: false, hei: 10.0, wid, per: per * 2.0, seg, ..Default::default() }));

    let mcnt = ctx.choice(&[0, 1, 1, 2]);
    if mcnt == 1 {
        let x = xoff + ctx.norm_rand(-wid / 3.0, wid / 3.0);
        let fli = ctx.choice(&[true, false]);
        canv.extend(man::man(ctx, x, yoff, ManArgs { fli: Some(fli), sca: Some(0.42), ..Default::default() }));
    } else if mcnt == 2 {
        let x = xoff + ctx.norm_rand(-wid / 4.0, -wid / 5.0);
        canv.extend(man::man(ctx, x, yoff, ManArgs { fli: Some(false), sca: Some(0.42), ..Default::default() }));
        let x = xoff + ctx.norm_rand(wid / 5.0, wid / 4.0);
        canv.extend(man::man(ctx, x, yoff, ManArgs { fli: Some(true), sca: Some(0.42), ..Default::default() }));
    }
    let seg = js::to_int32(3.0 + ctx.rand() * 3.0) as f64;
    canv.extend(rail(ctx, xoff, yoff, seed, RailArgs { tra: false, fro: true, hei: 10.0, wid, per: per * 2.0, seg, ..Default::default() }));
    canv
}

#[derive(Default)]
pub struct Arch02Args {
    pub wid: Option<f64>,
    pub rot: Option<f64>,
    pub sto: Option<f64>,
    pub sty: Option<u8>,
}

pub fn arch02(ctx: &mut Ctx, xoff: f64, yoff: f64, a: Arch02Args) -> Canv {
    let hei = 10.0;
    let wid = a.wid.unwrap_or(50.0);
    let rot = a.rot.unwrap_or(0.3);
    let per = 5.0;
    let sto = a.sto.unwrap_or(3.0);
    let sty = a.sty.unwrap_or(1);
    let hsp = [[0.0, 0.0], [1.0, 5.0], [1.0, 5.0], [1.0, 4.0]][sty as usize];
    let vsp = [[0.0, 0.0], [1.0, 2.0], [1.0, 2.0], [1.0, 3.0]][sty as usize];
    let dec = Deco { style: sty, hsp, vsp };

    let mut canv = Canv::new();
    let mut hoff = 0.0;
    let mut i = 0.0;
    while i < sto {
        let bw = wid * 0.85f64.powf(i);
        canv.extend(box_(ctx, xoff, yoff - hoff, BoxArgs { tra: false, hei, wid: bw, rot, wei: 1.5, per, dec: Some(&dec), ..Default::default() }));
        // The web version hangs a "Pizza Hut" sign on a third of these; the sign is
        // left out, but its draw is kept so a seed still paints the same landscape.
        if sto == 1.0 {
            ctx.rand();
        }
        let rw = wid * 0.9f64.powf(i);
        canv.extend(roof(ctx, xoff, yoff - hoff - hei, RoofArgs { hei, wid: rw, rot, wei: 1.5, per, cor: 5.0 }));
        hoff += hei * 1.5;
        i += 1.0;
    }
    canv
}

pub fn arch03(ctx: &mut Ctx, xoff: f64, yoff: f64, sto: f64, wid: f64) -> Canv {
    let hei = 10.0;
    let rot = 0.7;
    let per = 5.0;
    let dec = Deco { style: 1, hsp: [1.0, 4.0], vsp: [1.0, 2.0] };

    let mut canv = Canv::new();
    let mut hoff = 0.0;
    let mut i = 0.0;
    while i < sto {
        let bw = wid * 0.85f64.powf(i);
        canv.extend(box_(ctx, xoff, yoff - hoff, BoxArgs { tra: false, hei, wid: bw, rot, wei: 1.5, per: per / 2.0, dec: Some(&dec), ..Default::default() }));
        canv.extend(rail(
            ctx,
            xoff,
            yoff - hoff,
            i * 0.2,
            RailArgs { seg: 5.0, wid: wid * 0.85f64.powf(i) * 1.1, hei: hei / 2.0, per: per / 2.0, rot, wei: 0.5, tra: false, ..Default::default() },
        ));
        canv.extend(pagroof(ctx, xoff, yoff - hoff - hei, PagroofArgs { hei: hei * 1.5, wid: wid * 0.9f64.powf(i), wei: 1.5, per, cor: 10.0, sid: 4.0 }));
        hoff += hei * 1.5;
        i += 1.0;
    }
    canv
}

pub fn arch04(ctx: &mut Ctx, xoff: f64, yoff: f64, sto: f64) -> Canv {
    let hei = 15.0;
    let wid = 30.0;
    let rot = 0.7;
    let per = 5.0;

    let mut canv = Canv::new();
    let mut hoff = 0.0;
    let mut i = 0.0;
    while i < sto {
        let bw = wid * 0.85f64.powf(i);
        canv.extend(box_(ctx, xoff, yoff - hoff, BoxArgs { tra: true, hei, wid: bw, rot, wei: 1.5, per: per / 2.0, dec: None, ..Default::default() }));
        canv.extend(rail(
            ctx,
            xoff,
            yoff - hoff,
            i * 0.2,
            RailArgs { seg: 3.0, wid: wid * 0.85f64.powf(i) * 1.2, hei: hei / 3.0, per: per / 2.0, rot, wei: 0.5, tra: true, ..Default::default() },
        ));
        canv.extend(pagroof(ctx, xoff, yoff - hoff - hei, PagroofArgs { hei: hei * 1.0, wid: wid * 0.9f64.powf(i), wei: 1.5, per, cor: 10.0, sid: 4.0 }));
        hoff += hei * 1.2;
        i += 1.0;
    }
    canv
}

pub fn boat01(ctx: &mut Ctx, xoff: f64, yoff: f64, sca: f64, fli: bool) -> Canv {
    let len = 120.0;
    let mut canv = Canv::new();
    let dir = if fli { -1.0 } else { 1.0 };
    canv.extend(man::man(
        ctx,
        xoff + 20.0 * sca * dir,
        yoff,
        ManArgs {
            ite: Some(Item::Stick01),
            hat: Some(Hat::Hat02),
            sca: Some(0.5 * sca),
            fli: Some(!fli),
            len: Some([0.0, 30.0, 20.0, 30.0, 10.0, 30.0, 30.0, 30.0, 30.0]),
        },
    ));
    let mut plist1 = Vec::new();
    let mut plist2 = Vec::new();
    let fun1 = |x: f64| (x * PI).sin().powf(0.5) * 7.0 * sca;
    let fun2 = |x: f64| (x * PI).sin().powf(0.5) * 10.0 * sca;
    let mut i = 0.0;
    while i < len * sca {
        plist1.push([i * dir, fun1(i / len)]);
        plist2.push([i * dir, fun2(i / len)]);
        i += 5.0 * sca;
    }
    let plist: Vec<Pt> = plist1.iter().chain(plist2.iter().rev()).copied().collect();
    canv.push(poly(&plist, PolyArgs { xof: xoff, yof: yoff, fil: Some(WHITE), ..Default::default() }));
    let pts: Vec<Pt> = plist.iter().map(|v| [xoff + v[0], yoff + v[1]]).collect();
    let fun = |x: f64| (x * PI * 2.0).sin();
    canv.extend(stroke(ctx, &pts, StrokeArgs { wid: Some(1.0), fun: Some(&fun), col: Some(ink(0.4)), ..Default::default() }));
    canv
}

pub fn transmission_tower01(ctx: &mut Ctx, xoff: f64, yoff: f64) -> Canv {
    let hei = 100.0;
    let wid = 20.0;
    let mut canv = Canv::new();
    let half = |_: f64| 0.5;
    let mut quickstroke = |ctx: &mut Ctx, pl: &[Pt]| {
        let pts = offset(&div(pl, 5.0), xoff, yoff);
        canv.extend(stroke(ctx, &pts, StrokeArgs { wid: Some(1.0), fun: Some(&half), col: Some(ink(0.4)), ..Default::default() }));
    };

    let p00 = [-wid * 0.05, -hei];
    let p01 = [wid * 0.05, -hei];
    let p10 = [-wid * 0.1, -hei * 0.9];
    let p11 = [wid * 0.1, -hei * 0.9];
    let p20 = [-wid * 0.2, -hei * 0.5];
    let p21 = [wid * 0.2, -hei * 0.5];
    let p30 = [-wid * 0.5, 0.0];
    let p31 = [wid * 0.5, 0.0];

    let bch = [[0.7, -0.85], [1.0, -0.675], [0.7, -0.5]];
    for b in bch {
        quickstroke(ctx, &[[-b[0] * wid, b[1] * hei], [b[0] * wid, b[1] * hei]]);
        quickstroke(ctx, &[[-b[0] * wid, b[1] * hei], [0.0, (b[1] - 0.05) * hei]]);
        quickstroke(ctx, &[[b[0] * wid, b[1] * hei], [0.0, (b[1] - 0.05) * hei]]);
        quickstroke(ctx, &[[-b[0] * wid, b[1] * hei], [-b[0] * wid, (b[1] + 0.1) * hei]]);
        quickstroke(ctx, &[[b[0] * wid, b[1] * hei], [b[0] * wid, (b[1] + 0.1) * hei]]);
    }
    let l10 = div(&[p00, p10, p20, p30], 5.0);
    let l11 = div(&[p01, p11, p21, p31], 5.0);
    for i in 0..l10.len() - 1 {
        quickstroke(ctx, &[l10[i], l11[i + 1]]);
        quickstroke(ctx, &[l11[i], l10[i + 1]]);
    }
    quickstroke(ctx, &[p00, p01]);
    quickstroke(ctx, &[p10, p11]);
    quickstroke(ctx, &[p20, p21]);
    quickstroke(ctx, &[p00, p10, p20, p30]);
    quickstroke(ctx, &[p01, p11, p21, p31]);
    canv
}
