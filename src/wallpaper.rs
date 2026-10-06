use crate::Args;
use crate::render::WORLD_HEIGHT;
use crate::scroll::{self, Geometry, Piece, Request};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData, Region},
    delegate_dispatch2, delegate_registry,
    dispatch2::Dispatch2,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            EventLoop, LoopHandle,
            channel::{self, Event},
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, Proxy, QueueHandle,
            globals::registry_queue_init,
            protocol::{wl_buffer, wl_output, wl_shm, wl_surface},
        },
        protocols::wp::{
            fractional_scale::v1::client::{
                wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
                wp_fractional_scale_v1::{self, WpFractionalScaleV1},
            },
            viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
    shm::{Shm, ShmHandler, raw::RawPool},
};
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const PIECES: i64 = 3;
const RECREATE_DELAY: Duration = Duration::from_millis(500);

struct Buf {
    wl: wl_buffer::WlBuffer,
    offset: usize,
    k: Option<i64>,
    busy: bool,
}

// w, h are buffer pixels; dw, dh the surface size they are shown at.
struct Screen {
    w: u32,
    h: u32,
    dw: u32,
    dh: u32,
    piece_w: u32,
    zoom: f64,
}

impl Screen {
    fn buf_w(&self) -> u32 {
        self.piece_w * PIECES as u32
    }
    fn stride(&self) -> usize {
        self.buf_w() as usize * 4
    }
    fn buf_len(&self) -> usize {
        self.stride() * self.h as usize
    }
    fn geometry(&self) -> Geometry {
        Geometry { piece_w: self.piece_w, h: self.h, zoom: self.zoom }
    }
}

struct Surface {
    layer: LayerSurface,
    viewport: WpViewport,
    scale: Option<WpFractionalScaleV1>,
}

impl Surface {
    fn destroy(self) {
        self.viewport.destroy();
        if let Some(s) = self.scale {
            s.destroy();
        }
    }
}

struct App {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    viewporter: WpViewporter,
    fractional: Option<WpFractionalScaleManagerV1>,
    qh: QueueHandle<App>,
    handle: LoopHandle<'static, App>,

    speed: f64,
    interval: Duration,

    surface: Option<Surface>,
    size: (u32, u32),
    scale: u32,
    scale_known: bool,
    screen: Option<Screen>,
    pool: Option<RawPool>,
    bufs: Vec<Buf>,
    buf_gen: u64,
    shown: Option<usize>,
    offset: f64,
    world_x: f64,

    painter: mpsc::Sender<Request>,
    epoch: u64,
    pieces: BTreeMap<i64, Vec<u8>>,
    requested: i64,

    clock: Option<Instant>,
    last_commit: Option<Instant>,
    last_px: Option<i64>,
    stalled: bool,
    frame_pending: bool,
    tick_armed: bool,
    debug: bool,
    exit: bool,
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init::<App>(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let viewporter: WpViewporter = globals.bind(&qh, 1..=1, Proto).map_err(|e| format!("wp_viewporter: {e}"))?;
    let fractional: Option<WpFractionalScaleManagerV1> = globals.bind(&qh, 1..=1, Proto).ok();

    let mut event_loop: EventLoop<'static, App> = EventLoop::try_new()?;
    let handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(handle.clone()).map_err(|e| e.to_string())?;

    let (piece_tx, piece_rx) = channel::channel::<Piece>();
    handle
        .insert_source(piece_rx, |ev, _, app: &mut App| {
            if let Event::Msg(p) = ev {
                app.on_piece(p);
            }
        })
        .map_err(|e| e.to_string())?;
    let painter = scroll::spawn(args.seed.clone(), piece_tx);

    let mut app = App {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor,
        layer_shell,
        shm,
        viewporter,
        fractional,
        qh,
        handle,
        speed: args.speed,
        interval: Duration::from_secs_f64(1.0 / args.fps),
        surface: None,
        size: (0, 0),
        scale: 120,
        scale_known: false,
        screen: None,
        pool: None,
        bufs: Vec::new(),
        buf_gen: 0,
        shown: None,
        offset: 0.0,
        world_x: args.x.unwrap_or(0.0),
        painter,
        epoch: 0,
        pieces: BTreeMap::new(),
        requested: -1,
        clock: None,
        last_commit: None,
        last_px: None,
        stalled: false,
        frame_pending: false,
        tick_armed: false,
        debug: std::env::var_os("SHAN_SHUI_DEBUG").is_some(),
        exit: false,
    };
    app.create_surface();
    while !app.exit {
        event_loop.dispatch(None, &mut app)?;
    }
    Ok(())
}

impl App {
    fn create_surface(&mut self) {
        if self.surface.is_some() || self.output_state.outputs().next().is_none() {
            return;
        }
        let qh = &self.qh;
        let surface = self.compositor.create_surface(qh);
        let viewport = self.viewporter.get_viewport(&surface, qh, Proto);
        let scale = self.fractional.as_ref().map(|f| f.get_fractional_scale(&surface, qh, Proto));
        match Region::new(&self.compositor) {
            Ok(region) => surface.set_input_region(Some(region.wl_region())),
            Err(e) => eprintln!("input region: {e}"),
        }

        let layer = self.layer_shell.create_layer_surface(qh, surface, Layer::Background, Some("shan-shui"), None);
        layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(0, 0);
        layer.commit();
        self.surface = Some(Surface { layer, viewport, scale });
        self.scale_known = false;
    }

    // The compositor closes the surface when its output goes away (unplugged, or disabled
    // when the lid closes). Carry on at the same place on whichever output it picks next.
    fn surface_closed(&mut self) {
        if let Some(s) = self.surface.take() {
            s.destroy();
        }
        if let Some(s) = self.screen.take() {
            self.world_x = self.offset / s.zoom;
        }
        self.shown = None;
        self.frame_pending = false;
        self.clock = None;
        let _ = self.handle.insert_source(Timer::from_duration(RECREATE_DELAY), |_, _, app: &mut App| {
            app.create_surface();
            TimeoutAction::Drop
        });
    }

    fn configure(&mut self, w: u32, h: u32) {
        self.size = (w, h);
        if !self.scale_known
            && let Some(s) = self.guess_scale(w, h)
        {
            self.scale = s;
        }
        self.resize();
    }

    // Hyprland only sends the preferred scale once something is shown; until then take it
    // from the output whose logical size matches the surface, instead of painting twice.
    fn guess_scale(&self, w: u32, h: u32) -> Option<u32> {
        self.output_state.outputs().filter_map(|o| self.output_state.info(&o)).find_map(|i| {
            let (lw, lh) = i.logical_size?;
            let (mw, mh) = i.modes.iter().find(|m| m.current)?.dimensions;
            let (l, m) = (lw.max(lh) as u32, mw.max(mh) as u32);
            ((lw, lh) == (w as i32, h as i32) && l > 0).then(|| (m * 120 + l / 2) / l)
        })
    }

    fn set_scale(&mut self, scale: u32) {
        self.scale_known = true;
        if self.scale != scale {
            self.scale = scale;
            self.resize();
        }
    }

    fn resize(&mut self) {
        let (dw, dh) = self.size;
        if dw == 0 || dh == 0 {
            return;
        }
        let px = |v: u32| ((v as u64 * self.scale as u64 + 60) / 120) as u32;
        let (w, h) = (px(dw), px(dh));
        if let Some(s) = &self.screen
            && (s.w, s.h, s.dw, s.dh) == (w, h, dw, dh)
        {
            return;
        }
        if let Some(s) = &self.screen {
            self.world_x = self.offset / s.zoom;
        }
        if self.debug {
            eprintln!("resize {dw}x{dh} at scale {} -> buffer {w}x{h}", self.scale as f64 / 120.0);
        }
        let screen = Screen { w, h, dw, dh, piece_w: w.div_ceil(2), zoom: h as f64 / WORLD_HEIGHT };
        self.offset = self.world_x * screen.zoom;

        for b in self.bufs.drain(..) {
            b.wl.destroy();
        }
        self.buf_gen += 1;
        let mut pool = match RawPool::new(screen.buf_len() * 2, &self.shm) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("shm pool: {e}");
                self.exit = true;
                return;
            }
        };
        for i in 0..2 {
            let offset = i * screen.buf_len();
            let wl = pool.create_buffer(
                offset as i32,
                screen.buf_w() as i32,
                h as i32,
                screen.stride() as i32,
                wl_shm::Format::Xrgb8888,
                BufData { idx: i, generation: self.buf_gen },
                &self.qh,
            );
            self.bufs.push(Buf { wl, offset, k: None, busy: false });
        }
        self.pool = Some(pool);
        self.screen = Some(screen);
        self.shown = None;
        self.last_px = None;
        self.epoch += 1;
        self.pieces.clear();
        self.requested = self.step() - 1;
        self.request_ahead();
    }

    fn step(&self) -> i64 {
        let s = self.screen.as_ref().unwrap();
        (self.offset / s.piece_w as f64).floor() as i64
    }

    fn request_ahead(&mut self) {
        let s = self.screen.as_ref().unwrap();
        let k = match self.shown {
            Some(b) => self.bufs[b].k.unwrap(),
            None => self.step(),
        };
        let keep_from = k as f64 * s.piece_w as f64 / s.zoom;
        let last = k + PIECES;
        if self.requested < last {
            let req = Request { epoch: self.epoch, pieces: self.requested + 1..last + 1, geo: s.geometry(), keep_from };
            self.requested = last;
            if self.painter.send(req).is_err() {
                self.exit = true;
            }
        }
    }

    fn on_piece(&mut self, p: Piece) {
        if p.epoch != self.epoch {
            return;
        }
        self.pieces.insert(p.index, p.data);
        self.prepare_next();
        if self.shown.is_none() {
            self.arm_tick(Duration::ZERO);
        }
    }

    fn prepare_next(&mut self) {
        let Some(s) = &self.screen else { return };
        let next = match self.shown {
            Some(b) => self.bufs[b].k.unwrap() + 1,
            None => self.step(),
        };
        let Some(idle) = (0..self.bufs.len()).find(|&i| Some(i) != self.shown) else { return };
        if self.bufs[idle].k == Some(next) || self.bufs[idle].busy {
            return;
        }
        let from = self.shown.filter(|&b| self.bufs[b].k == Some(next - 1));
        let needed: Vec<i64> = if from.is_some() { vec![next + PIECES - 1] } else { (next..next + PIECES).collect() };
        if !needed.iter().all(|i| self.pieces.contains_key(i)) {
            return;
        }

        let (stride, pw, h) = (s.stride(), s.piece_w as usize * 4, s.h as usize);
        let dst = self.bufs[idle].offset;
        let mmap = self.pool.as_mut().unwrap().mmap();
        if let Some(b) = from {
            let src = self.bufs[b].offset;
            for y in 0..h {
                let row = src + y * stride + pw;
                mmap.copy_within(row..row + pw * (PIECES as usize - 1), dst + y * stride);
            }
        }
        for &i in &needed {
            let x = (i - next) as usize * pw;
            let piece = &self.pieces[&i];
            for y in 0..h {
                mmap[dst + y * stride + x..][..pw].copy_from_slice(&piece[y * pw..][..pw]);
            }
        }
        self.bufs[idle].k = Some(next);
        self.pieces.retain(|&i, _| i >= next + PIECES);
        if self.stalled {
            self.arm_tick(Duration::ZERO);
        }
    }

    fn arm_tick(&mut self, after: Duration) {
        if self.tick_armed {
            return;
        }
        self.tick_armed = true;
        let timer = Timer::from_duration(after);
        let _ = self.handle.insert_source(timer, |_, _, app: &mut App| {
            app.tick_armed = false;
            app.tick();
            TimeoutAction::Drop
        });
    }

    fn tick(&mut self) {
        if self.frame_pending {
            return;
        }
        let (Some(s), Some(surf)) = (&self.screen, &self.surface) else { return };
        let (w, h, dw, dh, pw) = (s.w, s.h, s.dw, s.dh, s.piece_w as i64);
        let (surface, viewport) = (surf.layer.wl_surface().clone(), surf.viewport.clone());
        let speed = self.speed * self.scale as f64 / 120.0;
        let now = Instant::now();

        let mut attach = None;
        let mut k = match self.shown {
            Some(b) => self.bufs[b].k.unwrap(),
            None => {
                let k = self.step();
                match (0..self.bufs.len()).find(|&i| self.bufs[i].k == Some(k)) {
                    Some(b) => attach = Some(b),
                    None => return,
                }
                k
            }
        };
        self.stalled = false;
        if self.shown.is_some() {
            let dt = self.clock.map_or(0.0, |t| (now - t).as_secs_f64().min(0.1));
            self.offset += speed * dt;
            if self.offset >= ((k + 1) * pw) as f64 {
                let idle = (0..self.bufs.len()).find(|&i| Some(i) != self.shown).unwrap();
                if self.bufs[idle].k == Some(k + 1) {
                    attach = Some(idle);
                    k += 1;
                } else {
                    self.offset = ((k + 1) * pw) as f64 - 0.01;
                    self.stalled = true;
                }
            }
        }
        self.clock = Some(now);

        let px = self.offset.floor() as i64;
        if attach.is_none() && Some(px) == self.last_px {
            if speed > 0.0 && !self.stalled {
                let wait = ((px + 1) as f64 - self.offset) / speed;
                self.arm_tick(Duration::from_secs_f64(wait.max(0.001)));
            }
            return;
        }

        if let Some(b) = attach {
            surface.attach(Some(&self.bufs[b].wl), 0, 0);
            surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
            self.bufs[b].busy = true;
            self.shown = Some(b);
        } else {
            surface.damage(0, 0, dw as i32, dh as i32);
        }
        viewport.set_source((px - k * pw) as f64, 0.0, w as f64, h as f64);
        viewport.set_destination(dw as i32, dh as i32);
        if speed > 0.0 {
            surface.frame(&self.qh, FrameCallbackData(surface.clone()));
            self.frame_pending = true;
        }
        surface.commit();
        self.last_commit = Some(now);
        self.last_px = Some(px);
        if self.debug {
            eprintln!("commit px={px} k={k} attach={attach:?} offset={:.2}", self.offset);
        }

        if attach.is_some() {
            self.request_ahead();
            self.prepare_next();
        }
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, factor: i32) {
        if self.fractional.is_none() {
            self.set_scale(factor.max(1) as u32 * 120);
        }
    }
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        if self.debug {
            eprintln!("frame callback");
        }
        self.frame_pending = false;
        let since = self.last_commit.map_or(Duration::ZERO, |t| t.elapsed());
        self.arm_tick(self.interval.saturating_sub(since));
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {
        self.create_surface();
    }
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for App {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.surface_closed();
    }

    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface, c: LayerSurfaceConfigure, _: u32) {
        self.configure(c.new_size.0, c.new_size.1);
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

struct BufData {
    idx: usize,
    generation: u64,
}

impl Dispatch2<wl_buffer::WlBuffer, App> for BufData {
    fn event(&self, app: &mut App, _: &wl_buffer::WlBuffer, ev: wl_buffer::Event, _: &Connection, _: &QueueHandle<App>) {
        if let wl_buffer::Event::Release = ev
            && self.generation == app.buf_gen
        {
            app.bufs[self.idx].busy = false;
            app.prepare_next();
        }
    }
}

struct Proto;

impl Dispatch2<WpViewporter, App> for Proto {
    fn event(&self, _: &mut App, _: &WpViewporter, _: <WpViewporter as Proxy>::Event, _: &Connection, _: &QueueHandle<App>) {}
}

impl Dispatch2<WpViewport, App> for Proto {
    fn event(&self, _: &mut App, _: &WpViewport, _: <WpViewport as Proxy>::Event, _: &Connection, _: &QueueHandle<App>) {}
}

impl Dispatch2<WpFractionalScaleManagerV1, App> for Proto {
    fn event(&self, _: &mut App, _: &WpFractionalScaleManagerV1, _: <WpFractionalScaleManagerV1 as Proxy>::Event, _: &Connection, _: &QueueHandle<App>) {}
}

impl Dispatch2<WpFractionalScaleV1, App> for Proto {
    fn event(&self, app: &mut App, _: &WpFractionalScaleV1, ev: wp_fractional_scale_v1::Event, _: &Connection, _: &QueueHandle<App>) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = ev {
            app.set_scale(scale);
        }
    }
}

delegate_registry!(App);
delegate_dispatch2!(App);
