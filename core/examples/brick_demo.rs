use fluffl::{
    audio::*,
    extras::{
        audio::{music_player::*, ogg::*, *},
        shapes::ShapePainter2D,
        text_writer::*,
    },
    io::*,
    prelude::*,
    window::{event_util::*, *},
    GlowGL, *,
};

/// Audio types can get really long so this must be done
type ShortState = MusicPlayer<OggBuffer>;
type ShortCallback = DeviceCB<ShortState>;
type ShortDeviceContext = FlufflAudioDeviceContext<ShortCallback, ShortState>;

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
    glow_decay: f32,
    brick_health: i32,
}

impl Brick {
    pub fn new(pos: [f32; 2], dims: [f32; 2]) -> Self {
        Self {
            pos,
            dims,
            color: [0.5, 0.5, 0.5, 1.],
            glow_weight: 0.0,
            glow_decay: 0.999,
            brick_health: 2,
        }
    }

    pub fn rad(&self) -> f32 {
        self.dims[1].min(self.dims[0]) * 0.8
    }

    pub fn get_segment(&self) -> [[f32; 2]; 2] {
        let a = [self.pos[0], self.pos[1] + self.dims[1] * 0.5];
        let b = [self.pos[0] + self.dims[0], self.pos[1] + self.dims[1] * 0.5];
        [a, b]
    }

    pub fn render(&mut self, painter: &mut ShapePainter2D, _ticks: f32) {
        let [a, b] = self.get_segment();
        self.glow_weight *= self.glow_decay;
        //(ticks.sin() + 1.0) * 0.5
        painter.draw_rectangle(
            &a[..],
            &b[..],
            &self.color[..],
            self.dims[1] * 0.5,
            5.,
            self.glow_weight,
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
    //audio stuff
    boss_intro_track: Option<ShortDeviceContext>,
    boss_main_track: Option<ShortDeviceContext>,
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
            boss_intro_track: None,
            boss_main_track: None,
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

#[fluffl(Debug)]
pub async fn main() {
    


    let window = FlufflWindow::init(FLUFFL_CONFIG).expect("init failed");
    let gl = window.gl();

    //load boss intro and put it into memory
    let file_bytes = load_file!("./wasm_bins/resources/BossIntro.ogg").unwrap();
    let boss_intro = ogg::OggFile::new().with_data(file_bytes).parse().unwrap();

    //load the main track into memory
    let file_bytes = load_file!("./wasm_bins/resources/BossMain.ogg").unwrap();
    let boss_main = ogg::OggFile::new().with_data(file_bytes).parse().unwrap();

    //setup the into track
    let intro_device: ShortDeviceContext = FlufflAudioDeviceContext::new(
        AudioDeviceCore::new()
            .with_specs(DesiredSpecs {
                sample_rate: boss_intro.sample_rate().map(|rate| rate),
                channels: Some(1),
                buffer_size: Some(4028),
            })
            .with_state(MusicPlayer {
                ticks: 0,
                state: PlayState::Paused,
                volume: 0.5,
                music_src: boss_intro.into(),
                repeat_track: false,
            })
            .with_callback(music_callback),
        window.audio_context(),
    );

    //setup the main boss track
    let main_device: ShortDeviceContext = FlufflAudioDeviceContext::new(
        AudioDeviceCore::new()
            .with_specs(DesiredSpecs {
                sample_rate: boss_main.sample_rate().map(|rate| rate),
                channels: Some(1),
                buffer_size: Some(512),
            })
            .with_state(MusicPlayer {
                ticks: 0,
                state: PlayState::Paused,
                volume: 0.5,
                music_src: boss_main.into(),
                repeat_track: true,
            })
            .with_callback(music_callback),
        window.audio_context(),
    );

    unsafe {
        gl.clear_color(0., 0., 0., 1.);
        gl.viewport(0, 0, window.width() as i32, window.height() as i32);
    }

    let mut app_state = BrickAppState::new(&gl);
    app_state.init_bricks();

    //set music here
    app_state.boss_intro_track = Some(intro_device);
    app_state.boss_main_track = Some(main_device);

    //load font here:
    let font_data = load_file!("./wasm_bins/resources/plasmatic.bcode").unwrap();
    let atlas = HieroAtlas::deserialize(font_data).ok().unwrap();
    app_state.writer = Some(TextWriter::new(&gl).with_atlas(atlas).build());

    FlufflWindow::main_loop(
        window,
        app_state,
        |window_ptr: FlufflWindowPtr, running: FlufflRunning, app_state: FlufflState<_>| async move {
            handle_events(window_ptr.clone(), app_state.clone(), running.clone());
            let screen_bounds = window_ptr.window().get_bounds();
            let gl = window_ptr.window().gl();
            let time = app_state.inner.borrow().time;
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                let gui_state = app_state.inner.borrow().gui_state;
                match gui_state {
                    GuiState::Menu => {
                        app_state.inner.borrow_mut().writer.as_mut().map(|writer| {
                            writer.draw_text_line_preserved(
                                "Rust Bricks",
                                screen_bounds.0 as f32 * 0.5 - 155.,
                                70.,
                                64.,
                                Some(screen_bounds),
                            );
                            writer.draw_text_line_preserved(
                                "Music by: \"SketchyLogic\" on OpenGameArt",
                                screen_bounds.0 as f32 * 0.5 - 242.4,
                                screen_bounds.1 as f32 * 0.5 - 32.0,
                                32.0,
                                Some(screen_bounds),
                            );
                            writer.draw_text_line_preserved(
                                "Source At: https://github.com/K-C-DaCosta/fluffl",
                                screen_bounds.0 as f32 * 0.5 - 300.4,
                                screen_bounds.1 as f32 * 0.5 + 1.0,
                                32.0,
                                Some(screen_bounds),
                            );
                            let height = 24. + (((time * 2.0).sin() + 1.0) * 0.5) * 8.0;
                            writer.draw_text_line_preserved(
                                "Press [spacebar] to start",
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
            app_state.inner.borrow_mut().time += 0.01;
        },
    );
}

pub fn draw_game_stage(
    gl: &GlowGL,
    app_state: FlufflState<BrickAppState>,
    window_ptr: FlufflWindowPtr,
) {
    let brick_state = &mut *app_state.inner.borrow_mut();
    let mouse_pos = brick_state.mouse_pos;
    let t = brick_state.time;

    let win_dims = window_ptr.window().get_bounds_f32();

    unsafe {
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);
    }

    let ball_rad = brick_state.ball_list[0].rad;
    let fired_status = brick_state.ball_fired;

    // get intro track's state
    let mut intro_state = PlayState::Playing;
    brick_state
        .boss_intro_track
        .as_mut()
        .unwrap()
        .modify_state(|music_state| {
            intro_state = music_state?.state;
            Some(())
        });

    //get main track's state
    let mut main_state = PlayState::Playing;
    brick_state
        .boss_main_track
        .as_mut()
        .unwrap()
        .modify_state(|state_opt| {
            main_state = state_opt?.state;
            Some(())
        });

    // start main music track when intro is finished
    // because main music thread goes on repeat this if statement should only fire once
    if intro_state.is_paused() && main_state.is_paused() {
        let main_track = brick_state.boss_main_track.as_mut().unwrap();
        main_track.modify_state(|opt_state| {
            let mp = opt_state?;
            mp.ticks = 0;
            mp.state = PlayState::RampUp(1);
            Some(())
        });
        main_track.resume();
    }

    if fired_status == false {
        //if the player hasn't fired 'tie' ball to paddle
        brick_state.ball_list[0].pos = [
            mouse_pos[0],
            window_ptr.window().height() as f32
                - brick_state.player_paddle.dims[1] * 0.5
                - ball_rad * 2.2,
        ];
    } else {
        let border_segments = [
            [[0., 0.], [0., win_dims.1]],
            [[0., 0.], [win_dims.0, 0.]],
            [[win_dims.0, 0.], [win_dims.0, win_dims.1]],
        ];

        let ball = brick_state.ball_list[0];

        for &seg in border_segments.iter() {
            if let Some(normal) = ball_v_capsule_collision(ball.pos, ball.rad, seg, 10.0) {
                let new_vel = reflect(ball.vel, normal);
                brick_state.ball_list[0].pos[0] += normal[0] * 1.5;
                brick_state.ball_list[0].pos[1] += normal[1] * 1.5;
                brick_state.ball_list[0].vel = new_vel;
            }
        }
        if let Some(normal) = ball_v_brick_collision(ball, brick_state.player_paddle) {
            let new_vel = reflect(ball.vel, normal);
            brick_state.ball_list[0].pos[0] += normal[0] * 1.5;
            brick_state.ball_list[0].pos[1] += normal[1] * 1.5;
            brick_state.ball_list[0].vel = new_vel;

            brick_state.player_paddle.glow_weight = 1.5;
            brick_state.player_paddle.glow_decay = 0.99;
        }

        let mut removable_bricks = Vec::new();

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

                    brick_state.brick_list[brick_index].brick_health -= 1;
                    brick_state.brick_list[brick_index].glow_weight += 0.5;

                    removable_bricks.push(brick_index);
                }
            }
        }

        //delete dead bricks
        if removable_bricks.is_empty() == false {
            let index = removable_bricks[0];
            if brick_state.brick_list[index].brick_health <= 0 {
                brick_state.brick_list.remove(index);
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
    app_state: FlufflState<BrickAppState>,
    running: FlufflRunning,
) {
    for event in window_ptr.window_mut().get_events().flush_iter_mut() {
        match event {
            EventKind::Quit => running.set(false),
            EventKind::KeyDown { code } => {
                if let KeyCode::SPACE = code {
                    let state = app_state.inner.borrow().gui_state;
                    if let GuiState::Menu = state {
                        //spacebar was pressed so change state to "Game"
                        app_state.inner.borrow_mut().gui_state = GuiState::Game;

                        // start playing the into music track here
                        app_state
                            .inner
                            .borrow_mut()
                            .boss_intro_track
                            .as_mut()
                            .map(|track| {
                                track.modify_state(|state_opt| {
                                    let music_player = state_opt?;
                                    music_player.ticks = 0;
                                    music_player.state = PlayState::RampUp(3600);
                                    Some(())
                                });
                                track.resume();
                            });
                    }
                }
            }
            EventKind::MouseDown { button_code, .. } => {
                let gui_state = app_state.inner.borrow().gui_state;
                let fired_status = app_state.inner.borrow().ball_fired;

                if let (GuiState::Game, false, MouseCode::LEFT_BUTTON) =
                    (gui_state, fired_status, button_code)
                {
                    app_state.inner.borrow_mut().ball_fired = true;
                    app_state.inner.borrow_mut().ball_list[0].vel = [0., -100.0];
                }
            }

            EventKind::MouseMove { x, y, .. } => {
                let state_ref = &mut *app_state.inner.borrow_mut();
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
    ball_v_capsule_collision(ball.pos, ball.rad, brick.get_segment(), brick.rad())
}

pub fn ball_v_capsule_collision(
    ball_pos: [f32; 2],
    ball_rad: f32,
    capsule: [[f32; 2]; 2],
    cap_rad: f32,
) -> Option<[f32; 2]> {
    let [a, b] = capsule;

    let seg_disp = [b[0] - a[0], b[1] - a[1]];
    let proj_arm = [ball_pos[0] - a[0], ball_pos[1] - a[1]];

    let t = (dot(seg_disp, proj_arm) / dot(seg_disp, seg_disp))
        .max(0.)
        .min(1.);
    let proj_pos = [seg_disp[0] * t + a[0], seg_disp[1] * t + a[1]];

    let mut perp_disp = sub(ball_pos, proj_pos);
    let perp_dist = dot(perp_disp, perp_disp).sqrt();

    let total_rad = ball_rad + cap_rad;
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
