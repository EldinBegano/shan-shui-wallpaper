mod js;
mod landscape;
mod noise;
mod prng;
mod render;
mod scroll;
mod wallpaper;

use landscape::world::{Chunk, World, insert_chunk};
use landscape::{Col, Shape};
use std::io::Write;

const USAGE: &str = "\
shan-shui-wallpaper — endless Chinese landscape painting as a scrolling wallpaper

usage: shan-shui-wallpaper [options]
  --seed <text>     landscape seed (default: current time, like the web version)
  --speed <px/s>    scroll speed in screen pixels per second (default 12)
  --fps <n>         frame rate cap (default 30; it steps one pixel at a time,
                    so at 12 px/s it only draws 12 frames a second)

debugging:
  --png <file> [--seed s] [--x <world x>] [--size WxH]   render one frame
  --dump <seed>     generator output as the web version's SVG (for diffing)
  --bench <seed>    time generation and painting
  --reach           how far chunks reach left of the slice that made them";

fn col_str(c: Col) -> String {
    match c {
        None => "none".into(),
        Some(c) => format!("rgba({},{},{},{})", c.r, c.g, c.b, js::num_str(c.a)),
    }
}

fn chunk_svg(canv: &[Shape]) -> String {
    let mut out = String::new();
    for s in canv {
        if let Some(l) = &s.label {
            out += &format!(
                "<text font-size='{}' font-family='Verdana' style='fill:{}' text-anchor='middle' transform='translate({},{}) rotate({})'>{}</text>",
                js::num_str(l.size),
                col_str(s.fil),
                js::num_str(l.x),
                js::num_str(l.y),
                js::num_str(l.ang),
                l.text
            );
            continue;
        }
        out += "<polyline points='";
        for p in &s.pts {
            out += &format!(" {},{}", js::to_fixed_str(p[0], 1), js::to_fixed_str(p[1], 1));
        }
        out += &format!("' style='fill:{};stroke:{};stroke-width:{}'/>", col_str(s.fil), col_str(s.str), js::num_str(s.wid));
    }
    out
}

fn harness_steps() -> Vec<f64> {
    let mut steps = vec![0.0];
    steps.extend((0..=20).map(|k| k as f64 * 200.0));
    steps.extend((0..=26).map(|k| 3800.0 - k as f64 * 200.0));
    steps
}

fn dump(seed: &str) {
    let mut world = World::new(seed);
    let mut chunks: Vec<Chunk> = Vec::new();
    for x in harness_steps() {
        world.chunkloader(x, x + 3000.0, &mut |c| insert_chunk(&mut chunks, c, |c| c.y));
    }
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for (i, c) in chunks.iter().enumerate() {
        if i > 0 {
            out.write_all(b"\n").unwrap();
        }
        write!(out, "{}|{}|{}|{}", c.tag, js::num_str(c.x), js::num_str(c.y), chunk_svg(&c.canv)).unwrap();
    }
}

fn bench(seed: &str) {
    let t = std::time::Instant::now();
    let mut world = World::new(seed);
    let mut n = 0;
    for x in harness_steps() {
        world.chunkloader(x, x + 3000.0, &mut |_| n += 1);
    }
    println!("generate (web harness sequence, {n} chunks): {:?}", t.elapsed());

    let (w, h) = (1920, 1080);
    let zoom = h as f64 / render::WORLD_HEIGHT;
    let t = std::time::Instant::now();
    let mut world = World::new(seed);
    let mut chunks = Vec::new();
    world.chunkloader(0.0, 3000.0, &mut |c| insert_chunk(&mut chunks, render::RChunk::new(c), |c| c.y));
    println!("generate first view: {:?}", t.elapsed());
    let paper = render::Paper::new();
    let font = render::Font::find();
    let mut out = vec![0u8; w * h * 4];
    let t = std::time::Instant::now();
    render::paint(&chunks, zoom, 0.0, w as u32, h as u32, &paper, font.as_ref(), &mut out);
    println!("paint {w}x{h}: {:?}", t.elapsed());
    let mem: usize = chunks.iter().map(|c| c.mem_bytes()).sum();
    println!("chunk memory for {} chunks: {:.1} MB", chunks.len(), mem as f64 / 1e6);
}

fn reach() {
    let mut worst: f64 = 0.0;
    for k in 0..40 {
        let seed = format!("reach{k}");
        let mut world = World::new(&seed);
        let mut x = 0.0;
        let mut first = true;
        while x < 20000.0 {
            let (a, b) = if first { (0.0, 3000.0) } else { (x, x + 3000.0) };
            first = false;
            world.chunkloader(a, b, &mut |c| {
                let slice = c.slice;
                let r = render::RChunk::new(c);
                if slice >= 0.0 {
                    worst = worst.max(slice - r.x0);
                }
            });
            x += 1000.0;
        }
    }
    println!("chunks reach at most {worst:.0} world units left of their slice");
}

fn png(args: &Args) {
    let (w, h) = args.size.unwrap_or((1920, 1080));
    let zoom = h as f64 / render::WORLD_HEIGHT;
    let x = args.x.unwrap_or(0.0);
    let mut world = World::new(&args.seed);
    let mut chunks = Vec::new();
    world.chunkloader(0.0, 3000.0, &mut |c| insert_chunk(&mut chunks, render::RChunk::new(c), |c| c.y));
    let right = x + w as f64 / zoom + scroll::LOOKAHEAD;
    world.chunkloader(x.max(0.0), right, &mut |c| insert_chunk(&mut chunks, render::RChunk::new(c), |c| c.y));
    let mut out = vec![0u8; w * h * 4];
    render::paint(&chunks, zoom, x * zoom, w as u32, h as u32, &render::Paper::new(), render::Font::find().as_ref(), &mut out);
    let mut pm = tiny_skia::Pixmap::new(w as u32, h as u32).unwrap();
    for (d, s) in pm.data_mut().chunks_exact_mut(4).zip(out.chunks_exact(4)) {
        d.copy_from_slice(&[s[2], s[1], s[0], 255]);
    }
    pm.save_png(args.png.as_ref().unwrap()).expect("write png");
}

pub struct Args {
    pub seed: String,
    pub speed: f64,
    pub fps: f64,
    png: Option<String>,
    x: Option<f64>,
    size: Option<(usize, usize)>,
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args {
        seed: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis().to_string(),
        speed: 12.0,
        fps: 30.0,
        png: None,
        x: None,
        size: None,
    };
    let mut it = argv.iter();
    let num = |v: Option<&String>, name: &str| -> f64 {
        v.and_then(|s| s.parse().ok()).unwrap_or_else(|| {
            eprintln!("{name} needs a number\n\n{USAGE}");
            std::process::exit(2)
        })
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--seed" => args.seed = it.next().cloned().unwrap_or_default(),
            "--speed" => args.speed = num(it.next(), "--speed"),
            "--fps" => args.fps = num(it.next(), "--fps").max(1.0),
            "--x" => args.x = Some(num(it.next(), "--x").max(0.0)),
            "--png" => args.png = it.next().cloned(),
            "--size" => {
                let s = it.next().cloned().unwrap_or_default();
                let (w, h) = s.split_once('x').unwrap_or(("", ""));
                args.size = Some((w.parse().unwrap_or(1920), h.parse().unwrap_or(1080)));
            }
            "--dump" => return dump(it.next().map(|s| s.as_str()).unwrap_or("1234")),
            "--bench" => return bench(it.next().map(|s| s.as_str()).unwrap_or("1234")),
            "--reach" => return reach(),
            "-h" | "--help" => return println!("{USAGE}"),
            _ => {
                eprintln!("unknown option {a}\n\n{USAGE}");
                std::process::exit(2);
            }
        }
    }
    if args.png.is_some() {
        return png(&args);
    }
    if let Err(e) = wallpaper::run(args) {
        eprintln!("shan-shui-wallpaper: {e}");
        std::process::exit(1);
    }
}
