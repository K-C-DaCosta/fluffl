use fluffl::{
    extras::shapes::ShapePainter2D,
    window::{event_util::*, glow, glow::*, *},
    FlufflError, GlowGL,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

static mut RAND_STATE: f32 = 1.0;

static FLUFFL_CONFIG: &'static str = "
    <window>
        <width>800</width>
        <height>600</height>
        <title>brick_demo</title>
        <fullscreen>false</fullscreen>
        <resizable>false</resizable>
        <canvas_id>fluffl</canvas_id>
    </window>
";

pub struct Ball {
    pos: [f32; 2],
    vel: [f32; 2],
    rad: f32,
    color: [f32; 4],
}

impl Ball {
    pub fn new(pos: [f32; 2], rad: f32) -> Self {
        Self {
            color: [0.2, 0.1, 0.1, 1.],
            vel: [0., 0.],
            pos,
            rad,
        }
    }

    // velocity is constant most of the time so this is good
    // enough
    pub fn step(&mut self, dt: f32) {
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
    }
}

pub struct Brick {
    color: [f32; 4],
    pos: [f32; 2],
    dims: [f32; 2],
    glow_weight: f32,
}

impl Brick {
    pub fn new(pos: [f32; 2], dims: [f32; 2]) -> Self {
        Self {
            pos,
            dims,
            color: [0.5, 0.5, 0.5, 1.],
            glow_weight: 0.0,
        }
    }

    pub fn render(&self, painter: &mut ShapePainter2D, ticks: f32) {
        let a = [self.pos[0], self.pos[1] + self.dims[1] * 0.5];
        let b = [self.pos[0] + self.dims[0], self.pos[1] + self.dims[1] * 0.5];

        painter.draw_rectangle(
            &a[..],
            &b[..],
            &self.color[..],
            self.dims[1] * 0.5,
            5.,
            (ticks.sin() + 1.0) * 0.5 * self.glow_weight,
            0.,
        );
    }
}

pub struct BrickAppState {
    //game shit
    player_paddle: Brick,
    ball_list: Vec<Ball>,
    brick_list: Vec<Brick>,
    //graphics stuff
    painter: ShapePainter2D,
    //other
    mouse_pos: [f32; 2],
    time: f32,
}
impl BrickAppState {
    fn new(gl: &GlowGL) -> Self {
        const BRICK_WIDTH: f32 = 55.;
        const BRICK_HEIGHT: f32 = 16.;

        let mut player_paddle = Brick::new([400. - 256. / 2., 600. - 10. - 16.], [256., 16.]);
        player_paddle.glow_weight = 0.3;

        let mut brick_list = Vec::new();

        let color_palette = [
            convert_color(0x364f6Bff),
            convert_color(0x3fc1c9ff),
            convert_color(0xf5f5f5ff),
            convert_color(0xfc5185ff),
        ];

        for j in 0..7 {
            for i in 0..10 {
                let x = (BRICK_WIDTH + 20.) * (i as f32) + 30.;
                let y = (BRICK_HEIGHT + 20.) * (j as f32) + 10.;
                let mut brick = Brick::new([x, y], [BRICK_WIDTH, BRICK_HEIGHT]);
                let rand_palette_index = (hacky_rand()*4.0) as usize ;
                brick.color = color_palette[rand_palette_index] ;
                brick.glow_weight = 0.0;
                brick_list.push(brick);
            }
        }

        Self {
            ball_list: Vec::new(),
            brick_list,
            mouse_pos: [0.; 2],
            painter: ShapePainter2D::new(gl),
            player_paddle: player_paddle,
            time: 0.0,
        }
    }
}

/// this app mostly just does setup
pub async fn fluffl_main() -> Result<(), FlufflError> {
    let window = FlufflWindow::init(FLUFFL_CONFIG)?;
    let gl = window.gl();

    unsafe {
        gl.clear_color(0., 0., 0., 1.);
        gl.viewport(0, 0, window.width() as i32, window.height() as i32);
    }

    let app_state = BrickAppState::new(&gl);
    FlufflWindow::main_loop(window, app_state, core_loop);
    Ok(())
}

pub async fn core_loop(
    window_ptr: FlufflWindowPtr,
    running: Rc<Cell<bool>>,
    app_state: Rc<RefCell<BrickAppState>>,
) {
    handle_events(window_ptr.clone(), app_state.clone(), running.clone()).await;

    let gl = window_ptr.window().gl();
    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

        let brick_state = &mut *app_state.borrow_mut();
        let mouse_pos = brick_state.mouse_pos;
        let t = brick_state.time;

        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);

        for brick in brick_state.brick_list.iter_mut() {
            brick.render(&mut brick_state.painter, 0.0);
        }

        brick_state.player_paddle.pos[0] = mouse_pos[0] - 256. / 2.;
        brick_state
            .player_paddle
            .render(&mut brick_state.painter, t);

        gl.disable(glow::BLEND);

        brick_state.time += 0.05;
    }
}

pub async fn handle_events(
    window_ptr: FlufflWindowPtr,
    app_state: Rc<RefCell<BrickAppState>>,
    running: Rc<Cell<bool>>,
) {
    for event in window_ptr.window_mut().get_events().iter_mut() {
        match event {
            EventKind::Quit => running.set(false),
            EventKind::MouseMove { x, y, .. } => {
                let state_ref = &mut *app_state.borrow_mut();
                state_ref.mouse_pos = [x as f32, y as f32];
            }
            _ => (),
        }
    }
}
/// A hacky rand() function thats good enough for some breakout clone
pub fn hacky_rand() -> f32 {
    unsafe {
        RAND_STATE += 2.0;
        if RAND_STATE > 1E9f32 {
            RAND_STATE = 0.0;
        }

        ((RAND_STATE * 500.0).sin()*5647.0).fract()
    }
}

pub fn convert_color(mut rgba: u32) -> [f32; 4] {
    let mut colors = [1.; 4];
    for k in 0..4 {
        colors[4 - k - 1] = (rgba & 0xff) as f32 / 255.;
        rgba >>= 8;
    }
    colors
}
