use fluffl::{
    extras::{shapes::ShapePainter2D, text_writer::*},
    io::*,
    window::{event_util::*, glow, glow::*, *},
    FlufflError, GlowGL, *,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

static mut RAND_STATE: u64 = 0;

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
#[derive(Copy, Clone)]
pub struct Ball {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    rad: f32,
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

    pub fn render(&self, painter: &mut ShapePainter2D) {
        painter.draw_circle(&self.pos[..], self.rad, &self.color[..], 0., 0.7);
    }

    // velocity is constant most of the time so this is good enough
    pub fn step(&mut self, dt: f32) {
        self.pos[0] += self.vel[0] * dt;
        self.pos[1] += self.vel[1] * dt;
    }
}
#[derive(Copy, Clone)]
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

    pub fn rad(&self) -> f32 {
        self.dims[1] * 0.8
    }

    pub fn get_segment(&self) -> [[f32; 2]; 2] {
        let a = [self.pos[0], self.pos[1] + self.dims[1] * 0.5];
        let b = [self.pos[0] + self.dims[0], self.pos[1] + self.dims[1] * 0.5];
        [a, b]
    }

    pub fn render(&self, painter: &mut ShapePainter2D, ticks: f32) {
        let [a, b] = self.get_segment();

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
#[derive(Copy, Clone)]
enum GuiState {
    Menu,
    Game,
}

pub struct BrickAppState {
    //game shit
    player_paddle: Brick,
    ball_list: Vec<Ball>,
    brick_list: Vec<Brick>,
    gui_state: GuiState,
    ball_fired: bool,
    //graphics stuff
    painter: ShapePainter2D,
    writer: Option<TextWriter>,
    //other
    mouse_pos: [f32; 2],
    time: f32,
}
impl BrickAppState {
    fn new(gl: &GlowGL) -> Self {
        let mut player_paddle = Brick::new([400. - 256. / 2., 600. - 10. - 16.], [256., 16.]);
        player_paddle.glow_weight = 0.3;

        Self {
            gui_state: GuiState::Menu,
            ball_list: vec![Ball::new([0., 0.], 20.0)],
            brick_list: vec![],
            mouse_pos: [0.; 2],
            painter: ShapePainter2D::new(gl),
            player_paddle: player_paddle,
            time: 0.0,
            writer: None,
            ball_fired: false,
        }
    }
    pub fn init_bricks(&mut self) {
        const BRICK_WIDTH: f32 = 55.;
        const BRICK_HEIGHT: f32 = 16.;

        self.brick_list.clear();

        let color_palette = [
            convert_color(0x794c74ff),
            convert_color(0x794c74ff),
            convert_color(0x794c74ff),
            convert_color(0x794c74ff),
            convert_color(0xc56183ff),
            convert_color(0xc56183ff),
            convert_color(0xc56183ff),
            convert_color(0xfadcaaff),
            convert_color(0xfadcaaff),
            convert_color(0xb2deecff),
        ];

        for j in 0..7 {
            for i in 0..10 {
                let x = (BRICK_WIDTH + 20.) * (i as f32) + 30.;
                let y = (BRICK_HEIGHT + 20.) * (j as f32) + 10.;
                let mut brick = Brick::new([x, y], [BRICK_WIDTH, BRICK_HEIGHT]);
                let rand_palette_index = ((color_palette.len() as f32) * hacky_rand()) as usize;
                brick.color = color_palette[rand_palette_index];
                brick.glow_weight = 0.0;
                self.brick_list.push(brick);
            }
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

    let mut app_state = BrickAppState::new(&gl);
    app_state.init_bricks();

    //load font here:
    let font_data = load_file!("../../wasm_bins/resources/plasmatic.bcode").unwrap();
    let atlas = HieroAtlas::deserialize(font_data).ok().unwrap();
    app_state.writer = Some(TextWriter::new(&gl).with_atlas(atlas).build());

    FlufflWindow::main_loop(window, app_state, core_loop);
    Ok(())
}

pub async fn core_loop(
    window_ptr: FlufflWindowPtr,
    running: Rc<Cell<bool>>,
    app_state: Rc<RefCell<BrickAppState>>,
) {
    handle_events(window_ptr.clone(), app_state.clone(), running.clone());

    let screen_bounds = window_ptr.window().get_bounds();
    let gl = window_ptr.window().gl();
    let time = app_state.borrow().time;
    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        let gui_state = app_state.borrow().gui_state;
        match gui_state {
            GuiState::Menu => {
                app_state.borrow_mut().writer.as_mut().map(|writer| {
                    writer.draw_text_line(
                        "Rust bricks",
                        screen_bounds.0 as f32 * 0.5 - 155.,
                        128.,
                        64.,
                        Some(screen_bounds),
                    );
                    let height = 16. + (((time * 2.0).sin() + 1.0) * 0.5) * 16.0;
                    writer.draw_text_line(
                        "Press spacebar to start",
                        screen_bounds.0 as f32 * 0.5 - 100. - height * 1.4,
                        screen_bounds.1 as f32 * 0.7,
                        height,
                        Some(screen_bounds),
                    );
                });
            }
            GuiState::Game => {
                draw_game_stage(&gl, app_state.clone(), window_ptr.clone());
            }
        }
    }
    app_state.borrow_mut().time += 0.01;
}

pub fn draw_game_stage(
    gl: &GlowGL,
    app_state: Rc<RefCell<BrickAppState>>,
    window_ptr: FlufflWindowPtr,
) {
    let brick_state = &mut *app_state.borrow_mut();
    let mouse_pos = brick_state.mouse_pos;
    let t = brick_state.time;

    unsafe {
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);
    }

    let ball_rad = brick_state.ball_list[0].rad;
    let fired_status = brick_state.ball_fired;

    if fired_status == false {
        //if the player hasn't fired 'tie' ball to paddle
        brick_state.ball_list[0].pos = [
            mouse_pos[0],
            window_ptr.window().height() as f32
                - brick_state.player_paddle.dims[1] * 0.5
                - ball_rad * 2.2,
        ];
    } else {
        let ball = brick_state.ball_list[0];
        if let Some(normal) = ball_v_brick_collision(ball, brick_state.player_paddle){
            let new_vel = reflect(ball.vel, normal);
            brick_state.ball_list[0].pos[0] += normal[0]*1.5;
            brick_state.ball_list[0].pos[1] += normal[1]*1.5;
            brick_state.ball_list[0].vel = new_vel;
        }
     

        // do collision detection
        for ball_index in 0..brick_state.ball_list.len() {
            for brick_index in 0..brick_state.brick_list.len() {
                let ball = brick_state.ball_list[ball_index];
                let brick = brick_state.brick_list[brick_index];

                if let Some(normal) = ball_v_brick_collision(ball, brick) {
                    let new_vel = reflect(brick_state.ball_list[ball_index].vel, normal);
                    brick_state.ball_list[ball_index].pos[0] += normal[0];
                    brick_state.ball_list[ball_index].pos[1] += normal[1];
                    brick_state.ball_list[ball_index].vel = new_vel;
                   
                }

           
            }

            
        }
    }

    //draw player paddle
    brick_state.player_paddle.pos[0] = mouse_pos[0] - brick_state.player_paddle.dims[0] / 2.;
    brick_state
        .player_paddle
        .render(&mut brick_state.painter, t);

    //draw bricks
    for brick in brick_state.brick_list.iter_mut() {
        brick.render(&mut brick_state.painter, 0.0);
    }

    //draw ball
    for ball in brick_state.ball_list.iter_mut() {
        ball.render(&mut brick_state.painter);
        ball.step(0.1);
    }

    unsafe {
        gl.disable(glow::BLEND);
    }
}

pub fn handle_events(
    window_ptr: FlufflWindowPtr,
    app_state: Rc<RefCell<BrickAppState>>,
    running: Rc<Cell<bool>>,
) {
    for event in window_ptr.window_mut().get_events().iter_mut() {
        match event {
            EventKind::Quit => running.set(false),
            EventKind::KeyDown { code } => {
                if let KeyCode::SPACE = code {
                    let state = app_state.borrow().gui_state;
                    if let GuiState::Menu = state {
                        app_state.borrow_mut().gui_state = GuiState::Game;
                    }
                }
            }
            EventKind::MouseDown { button_code, .. } => {
                let gui_state = app_state.borrow().gui_state;
                let fired_status = app_state.borrow().ball_fired;

                if let (GuiState::Game, false, MouseCode::LEFT_BUTTON) =
                    (gui_state, fired_status, button_code)
                {
                    app_state.borrow_mut().ball_fired = true;
                    app_state.borrow_mut().ball_list[0].vel = [0., -50.0];
                }
            }

            EventKind::MouseMove { x, y, .. } => {
                let state_ref = &mut *app_state.borrow_mut();
                state_ref.mouse_pos = [x as f32, y as f32];
            }
            _ => (),
        }
    }
}

/// A hacky rand() function thats good enough for some breakout clone
/// see: https://en.wikipedia.org/wiki/Random_number_generation
pub fn hacky_rand() -> f32 {
    const A: u64 = 82828220110;
    const B: u64 = 31415999;
    const M: u64 = 653000;
    unsafe {
        RAND_STATE = (A * RAND_STATE + B) % M;
        RAND_STATE as f32 / M as f32
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

pub fn ball_v_brick_collision(ball: Ball, brick: Brick) -> Option<[f32; 2]> {
    let [a, b] = brick.get_segment();

    let seg_disp = [b[0] - a[0], b[1] - a[1]];
    let proj_arm = [ball.pos[0] - a[0], ball.pos[1] - a[1]];

    let t = (dot(seg_disp, proj_arm) / dot(seg_disp, seg_disp))
        .max(0.)
        .min(1.);
    let proj_pos = [seg_disp[0] * t + a[0], seg_disp[1] * t + a[1]];

    let mut perp_disp = sub(ball.pos, proj_pos);
    let perp_dist = dot(perp_disp, perp_disp).sqrt();

    let total_rad = ball.rad + brick.rad();
    if perp_dist < total_rad {
        let overlap = total_rad - perp_dist;

        perp_disp[0] *= (overlap / perp_dist) * 0.5;
        perp_disp[1] *= (overlap / perp_dist) * 0.5;

        Some(perp_disp)
    } else {
        None
    }
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn scale(a: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] * t, a[1] * t]
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

fn reflect(dir: [f32; 2], normal: [f32; 2]) -> [f32; 2] {
    let t = 2.0 * dot(normal, dir) / dot(normal, normal);
    sub(dir, scale(normal, t))
}
