use crate::landscape::world::{Chunk, World, insert_chunk};
use crate::render::{self, Paper, RChunk};
use smithay_client_toolkit::reexports::calloop::channel;
use std::ops::Range;
use std::sync::mpsc;

pub const LOOKAHEAD: f64 = 2000.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Geometry {
    pub piece_w: u32,
    pub h: u32,
    pub zoom: f64,
}

pub struct Request {
    pub epoch: u64,
    pub pieces: Range<i64>,
    pub geo: Geometry,
    pub keep_from: f64,
}

pub struct Piece {
    pub epoch: u64,
    pub index: i64,
    pub data: Vec<u8>,
}

pub fn spawn(seed: String, out: channel::Sender<Piece>) -> mpsc::Sender<Request> {
    let (tx, rx) = mpsc::channel::<Request>();
    std::thread::Builder::new()
        .name("painter".into())
        .spawn(move || {
            // Threads spawned below inherit this nice value.
            unsafe {
                libc::setpriority(libc::PRIO_PROCESS, libc::gettid() as libc::id_t, 15);
            }
            let workers = std::thread::available_parallelism().map_or(1, |n| n.get());
            let mut world = World::new(&seed);
            let mut chunks: Vec<RChunk> = Vec::new();
            let mut started = false;
            let paper = Paper::new();
            loop {
                let Ok(req) = rx.recv() else { return };
                let mut reqs = vec![req];
                reqs.extend(rx.try_iter());
                let epoch = reqs.iter().map(|r| r.epoch).max().unwrap();
                reqs.retain(|r| r.epoch == epoch);
                let g = reqs[0].geo;
                let indices: Vec<i64> = reqs.iter().flat_map(|r| r.pieces.clone()).collect();
                let (Some(&lo), Some(&hi)) = (indices.iter().min(), indices.iter().max()) else { continue };
                let keep_from = reqs.iter().map(|r| r.keep_from).fold(f64::INFINITY, f64::min);

                let px0 = lo as f64 * g.piece_w as f64;
                let right = (hi + 1) as f64 * g.piece_w as f64 / g.zoom;
                let mut add = |c: Chunk| insert_chunk(&mut chunks, RChunk::new(c), |c| c.y);
                if !started {
                    world.chunkloader(0.0, 3000.0, &mut add);
                    started = true;
                }
                world.chunkloader((px0 / g.zoom).max(0.0), right + LOOKAHEAD, &mut add);
                chunks.retain(|c| c.x1 >= keep_from);

                // At startup a whole screen of pieces is wanted at once; paint them side by side.
                let paint = |index: i64| {
                    let mut data = vec![0u8; g.piece_w as usize * g.h as usize * 4];
                    render::paint(&chunks, g.zoom, index as f64 * g.piece_w as f64, g.piece_w, g.h, &paper, &mut data);
                    Piece { epoch, index, data }
                };
                for group in indices.chunks(workers) {
                    let pieces: Vec<Piece> = std::thread::scope(|s| {
                        let rest: Vec<_> = group[1..].iter().map(|&i| s.spawn(move || paint(i))).collect();
                        let first = paint(group[0]);
                        std::iter::once(first).chain(rest.into_iter().map(|h| h.join().unwrap())).collect()
                    });
                    for p in pieces {
                        if out.send(p).is_err() {
                            return;
                        }
                    }
                }
            }
        })
        .expect("spawn painter thread");
    tx
}
