use fluffl::{
    console::*,
    prelude::*,
    text_writer::*,
    window::{event_util::*, *},
    *,
};

use std::collections::HashMap;

static CONFIG: &str = r#"
    {
        "width":800, 
        "height":600,
        "resizable":true,
        "canvas_id":fluffl
    }
"#;

struct AppState {
    writer: Option<TextWriter>,
    touch_positions: HashMap<i32, [f32; 2]>,
    mouse_pos: [f32; 2],
    mouse_disp: [f32; 2],
    mouse_down: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            writer: None,
            touch_positions: HashMap::new(),
            mouse_pos: [0.; 2],
            mouse_disp: [0.; 2],
            mouse_down: false,
        }
    }
}

#[fluffl(Debug)]
pub async fn main() {
    let window = FlufflWindow::init(CONFIG).expect("init failed");
    let gl = window.gl();
    let mut app_state = AppState::new();
    let writer = TextWriter::new(&gl)
        .with_atlas(
            text_writer::default_font::UROOB
                .to_hiero_atlas()
                .expect("failed to deserialize UROOB"),
        )
        .build();
    app_state.writer = Some(writer);

    unsafe {
        gl.clear_color(0.2, 0.2, 0.2, 1.0);
        gl.viewport(0, 0, 800, 600);
    }

    FlufflWindow::run(
        window,
        app_state,
        |win_ptr, mut running, app_state| async move {
            let bounds = win_ptr.window().get_bounds();
            let gl = win_ptr.window().gl();

            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }

            let state = &mut *app_state.borrow_mut();
            let writer = state.writer.as_mut().unwrap();
            let mut go_fullscreen = false;

            for event in win_ptr.window_mut().get_events().flush_iter_mut() {
                match event {
                    EventKind::Quit => {
                        running.set(false);
                    }
                    EventKind::MouseDown { x, y, .. } => {
                        // console_log!("pos:[{},{}]\n",x,y);
                        state.mouse_pos = [x, y];
                        state.mouse_disp = [0.; 2];
                        state.mouse_down = true;
                    }
                    EventKind::MouseUp { x, y, .. } => {
                        state.mouse_pos = [x, y];
                        state.mouse_disp = [0.; 2];
                        state.mouse_down = false;
                        // console_log!("pos:[{},{}]\n",x,y);
                        if x < 64.0 && y < 64.0 {
                            console_log!("Going fullscreen!\n");
                            go_fullscreen = true;
                        }
                    }
                    EventKind::MouseMove { x, y, dx, dy } => {
                        state.mouse_pos = [x, y];
                        state.mouse_disp = [dx, dy];
                        // console_log!("pos:[{},{}]\n",x,y);
                    }
                    EventKind::TouchDown {
                        x, y, finger_id, ..
                    } => {
                        state.touch_positions.insert(finger_id, [x, y]);
                    }
                    EventKind::TouchMove {
                        x, y, finger_id, ..
                    } => {
                        state.touch_positions.insert(finger_id, [x, y]);
                    }
                    EventKind::TouchUp { finger_id, .. } => {
                        state.touch_positions.remove(&finger_id);
                    }
                    EventKind::Resize { width, height } => {
                        console_log!("resized:[{},{}]\n", width, height);
                        unsafe {
                            gl.viewport(0, 0, width, height);
                        }
                    }
                    _ => (),
                }
            }

            if go_fullscreen {
                win_ptr.window_mut().set_fullscreen(true);
            }

            let id_str = format!("[w:{},h:{}]", bounds.0, bounds.1);
            writer.draw_text_line_preserved(
                id_str.as_str(),
                bounds.0 as f32 / 2.,
                bounds.1 as f32 / 2.,
                32.,
                Some(bounds),
            );

            //draws the word 'mouse' tied to the cursor position only when is down
            if state.mouse_down {
                let [x, y] = state.mouse_pos;
                let pos_str = format!("[{},{}]", x, y);
                let aabb = writer.calc_text_aabb_preserved(pos_str.as_str(), x, y, 48.);
                let tx = (x - aabb.w() / 2.).max(0.).min(bounds.0 as f32 - aabb.w());
                let ty = (y - aabb.h() / 2.).max(0.).min(bounds.1 as f32 - aabb.h());
                writer.draw_text_line_preserved(pos_str.as_str(), tx, ty, 48., Some(bounds));
            }

            //draw text at centered at id positions
            for (id, &[x, y]) in state.touch_positions.iter() {
                let id_str = format!("id:{},[{},{}]", id, x, y);
                let aabb = writer.calc_text_aabb_preserved(id_str.as_str(), x, y, 32.);
                writer.draw_text_line_preserved(
                    id_str.as_str(),
                    (x - aabb.w() / 2.).max(0.).min(bounds.0 as f32 - aabb.w()),
                    (y - aabb.h() / 2.).max(0.).min(bounds.1 as f32 - aabb.h()),
                    32.,
                    Some(bounds),
                );
            }
        },
    );
}
