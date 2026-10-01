use crate::landscape::world::{Chunk, World, insert_chunk};
use crate::render::{self, Font, Paper, RChunk};
use smithay_client_toolkit::reexports::calloop::channel;
use std::collections::VecDeque;
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
    pub index: i64,
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
            unsafe {
                libc::setpriority(libc::PRIO_PROCESS, libc::gettid() as libc::id_t, 15);
            }
            let mut world = World::new(&seed);
            let mut chunks: Vec<RChunk> = Vec::new();
            let mut started = false;
            let paper = Paper::new();
            let font = Font::find();
            let mut queue: VecDeque<Request> = VecDeque::new();
            loop {
                if queue.is_empty() {
                    match rx.recv() {
                        Ok(r) => queue.push_back(r),
                        Err(_) => return,
                    }
                }
                queue.extend(rx.try_iter());
                let epoch = queue.iter().map(|r| r.epoch).max().unwrap();
                queue.retain(|r| r.epoch == epoch);
                let req = queue.pop_front().unwrap();

                let g = req.geo;
                let px0 = req.index as f64 * g.piece_w as f64;
                let right = (px0 + g.piece_w as f64) / g.zoom;
                let mut add = |c: Chunk| insert_chunk(&mut chunks, RChunk::new(c), |c| c.y);
                if !started {
                    world.chunkloader(0.0, 3000.0, &mut add);
                    started = true;
                }
                world.chunkloader((px0 / g.zoom).max(0.0), right + LOOKAHEAD, &mut add);
                chunks.retain(|c| c.x1 >= req.keep_from);

                let mut data = vec![0u8; g.piece_w as usize * g.h as usize * 4];
                render::paint(&chunks, g.zoom, px0, g.piece_w, g.h, &paper, font.as_ref(), &mut data);
                if out.send(Piece { epoch, index: req.index, data }).is_err() {
                    return;
                }
            }
        })
        .expect("spawn painter thread");
    tx
}
