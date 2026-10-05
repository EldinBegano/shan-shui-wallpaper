use crate::landscape::world::Chunk;
use crate::landscape::{Col, Ctx};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

pub const WORLD_HEIGHT: f64 = 800.0 / 1.142;

struct RShape {
    start: u32,
    end: u32,
    fill: [u8; 4],
    stroke: [u8; 4],
    wid: f32,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
}

pub struct RChunk {
    pub y: f64,
    pub x0: f64,
    pub x1: f64,
    ox: f64,
    pts: Vec<f32>,
    shapes: Vec<RShape>,
}

fn pack(c: Col) -> [u8; 4] {
    match c {
        None => [0; 4],
        Some(c) => [c.r, c.g, c.b, (c.a.clamp(0.0, 1.0) * 255.0).round() as u8],
    }
}

impl RChunk {
    pub fn new(c: Chunk) -> RChunk {
        let ox = c.x;
        let mut pts = Vec::new();
        let mut shapes = Vec::with_capacity(c.canv.len());
        let (mut cx0, mut cx1) = (f64::INFINITY, f64::NEG_INFINITY);
        for s in c.canv {
            let fill = pack(s.fil);
            let stroke = if s.wid > 0.0 { pack(s.str) } else { [0; 4] };
            if fill[3] == 0 && stroke[3] == 0 {
                continue;
            }
            let start = pts.len() as u32;
            let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
            for p in &s.pts {
                pts.push((p[0] - ox) as f32);
                pts.push(p[1] as f32);
                x0 = x0.min(p[0]);
                x1 = x1.max(p[0]);
                y0 = y0.min(p[1]);
                y1 = y1.max(p[1]);
            }
            if s.pts.len() < 2 {
                pts.truncate(start as usize);
                continue;
            }
            let pad = if stroke[3] > 0 { s.wid * 2.0 } else { 0.0 };
            cx0 = cx0.min(x0 - pad);
            cx1 = cx1.max(x1 + pad);
            shapes.push(RShape {
                start,
                end: pts.len() as u32,
                fill,
                stroke,
                wid: s.wid as f32,
                x0: (x0 - pad - ox) as f32,
                x1: (x1 + pad - ox) as f32,
                y0: (y0 - pad) as f32,
                y1: (y1 + pad) as f32,
            });
        }
        pts.shrink_to_fit();
        RChunk { y: c.y, x0: cx0, x1: cx1, ox, pts, shapes }
    }

    pub fn mem_bytes(&self) -> usize {
        self.pts.len() * 4 + self.shapes.len() * std::mem::size_of::<RShape>()
    }
}

pub struct Paper {
    px: Vec<[u8; 3]>,
}

const PAPER: usize = 512;

impl Paper {
    pub fn new() -> Paper {
        let mut ctx = Ctx::new("paper");
        let mut px = vec![[0u8; 3]; PAPER * PAPER];
        let reso = PAPER;
        for i in 0..=reso / 2 {
            for j in 0..=reso / 2 {
                let mut c = 245.0 + ctx.noise(i as f64 * 0.1, j as f64 * 0.1, 0.0) * 10.0;
                c -= ctx.rand() * 20.0;
                let rgb = [c.round() as u8, (c * 0.95).round() as u8, (c * 0.85).round() as u8];
                for (x, y) in [(i, j), (reso - i, j), (i, reso - j), (reso - i, reso - j)] {
                    if x < reso && y < reso {
                        px[y * reso + x] = rgb;
                    }
                }
            }
        }
        Paper { px }
    }
}

fn paint_of(c: [u8; 4]) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(c[0], c[1], c[2], c[3]);
    p.anti_alias = true;
    p
}

pub fn paint(chunks: &[RChunk], zoom: f64, px0: f64, w: u32, h: u32, paper: &Paper, out: &mut [u8]) {
    const M: u32 = 4;
    let (w_out, px_out) = (w, px0);
    let (w, px0) = (w + 2 * M, px0 - M as f64);
    let mut pm = Pixmap::new(w, h).expect("pixmap size");
    pm.fill(tiny_skia::Color::WHITE);
    let m = 2.0 / zoom;
    let wx0 = px0 / zoom - m;
    let wx1 = (px0 + w as f64) / zoom + m;
    let (wy0, wy1) = (-m, h as f64 / zoom + m);

    let mut pb = PathBuilder::new();
    for ch in chunks {
        if ch.x1 < wx0 || ch.x0 > wx1 {
            continue;
        }
        let (lx0, lx1) = ((wx0 - ch.ox) as f32, (wx1 - ch.ox) as f32);
        let base = ch.ox * zoom - px0;
        let z = zoom as f32;
        for s in &ch.shapes {
            if s.x1 < lx0 || s.x0 > lx1 || (s.y1 as f64) < wy0 || s.y0 as f64 > wy1 {
                continue;
            }
            pb.clear();
            let pts = &ch.pts[s.start as usize..s.end as usize];
            pb.move_to((base + (pts[0] * z) as f64) as f32, pts[1] * z);
            for p in pts[2..].chunks_exact(2) {
                pb.line_to((base + (p[0] * z) as f64) as f32, p[1] * z);
            }
            let Some(path) = std::mem::take(&mut pb).finish() else {
                continue;
            };
            if s.fill[3] > 0 {
                pm.fill_path(&path, &paint_of(s.fill), FillRule::Winding, Transform::identity(), None);
            }
            if s.stroke[3] > 0 {
                let st = Stroke { width: s.wid * z, miter_limit: 4.0, line_cap: LineCap::Butt, line_join: LineJoin::Miter, dash: None };
                pm.stroke_path(&path, &paint_of(s.stroke), &st, Transform::identity(), None);
            }
            pb = path.clear();
        }
    }

    let data = pm.data();
    for y in 0..h as usize {
        let prow = &paper.px[(y % PAPER) * PAPER..][..PAPER];
        let row = &data[(y * w as usize + M as usize) * 4..][..w_out as usize * 4];
        let orow = &mut out[y * w_out as usize * 4..][..w_out as usize * 4];
        let mut pxi = (px_out as i64).rem_euclid(PAPER as i64) as usize;
        for (i, o) in orow.chunks_exact_mut(4).enumerate() {
            let s = &row[i * 4..i * 4 + 3];
            let p = prow[pxi];
            o[0] = ((s[2] as u32 * p[2] as u32 + 127) / 255) as u8;
            o[1] = ((s[1] as u32 * p[1] as u32 + 127) / 255) as u8;
            o[2] = ((s[0] as u32 * p[0] as u32 + 127) / 255) as u8;
            o[3] = 255;
            pxi = if pxi + 1 == PAPER { 0 } else { pxi + 1 };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::landscape::world::{World, insert_chunk};

    #[test]
    fn pieces_have_no_seams() {
        let (w, h) = (960u32, 1080u32);
        let zoom = h as f64 / WORLD_HEIGHT;
        let mut world = World::new("seam");
        let mut chunks = Vec::new();
        world.chunkloader(0.0, 3000.0, &mut |c| insert_chunk(&mut chunks, RChunk::new(c), |c| c.y));
        let paper = Paper::new();
        let px0 = 300.0;
        let mut whole = vec![0u8; (2 * w * h * 4) as usize];
        paint(&chunks, zoom, px0, 2 * w, h, &paper, &mut whole);
        for i in 0..2u32 {
            let mut piece = vec![0u8; (w * h * 4) as usize];
            paint(&chunks, zoom, px0 + (i * w) as f64, w, h, &paper, &mut piece);
            let (mut edge, mut inner) = (0, 0);
            for y in 0..h as usize {
                let a = &whole[(y * 2 * w as usize + (i * w) as usize) * 4..][..w as usize * 4];
                let b = &piece[y * w as usize * 4..][..w as usize * 4];
                for (k, (p, q)) in a.iter().zip(b).enumerate() {
                    let d = (*p as i32 - *q as i32).abs();
                    let x = k / 4;
                    if x < 3 || x + 3 >= w as usize { edge = edge.max(d) } else { inner = inner.max(d) }
                }
            }
            let worst = edge.max(inner);
            assert!(worst <= 8 && edge <= inner.max(4), "piece {i}: edges differ by {edge}, inside by {inner}");
        }
    }
}
