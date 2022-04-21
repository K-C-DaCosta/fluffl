use fluffl::{
    console::*,
    extras::text_writer::*,
    io::*,
    prelude::*,
    window::{event_util::*, glow::*, *},
    *,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

static CONFIG: &'static str = "
<window>
<width>800</width> 
<height>600</height>
<resizable>true</resizable> 
<canvas_id>fluffl</canvas_id>
</window>
";

struct AppState {
    writer: Option<TextWriter>,
    touch_positions: HashMap<i32, [f32; 2]>,
    mouse_pos: [f32; 2],
    mouse_disp: [i32; 2],
    mouse_down: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            writer: None,
            touch_positions: HashMap::new(),
            mouse_pos: [0.; 2],
            mouse_disp: [0; 2],
            mouse_down: false,
        }
    }
}

#[fluffl(Debug)]
pub async fn main() -> Result<(), FlufflError> {
    let window = FlufflWindow::init(CONFIG).expect("init failed");
    let gl = window.gl();
    let mut app_state = AppState::new();

    // load font called "plasmatic"
    let data = load_file!("./wasm_bins/resources/plasmatic.bcode").expect("load failed");
    let atlas = HieroAtlas::deserialize(data)
        .ok()
        .expect("font unpacked failed");
    let writer = TextWriter::new(&gl).with_atlas(atlas).build();
    app_state.writer = Some(writer);

    unsafe {
        gl.clear_color(0.2, 0.2, 0.2, 1.0);
        gl.viewport(0, 0, 800, 600);
    }

    FlufflWindow::main_loop(
        window,
        app_state,
        |win_ptr, running, app_state| async move {
            let bounds = win_ptr.window().get_bounds();
            let gl = win_ptr.window().gl();

            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }

            let state = &mut *app_state.inner.borrow_mut();
            let writer = state.writer.as_mut().unwrap();
            let mut go_fullscreen = false;

            for event in win_ptr.window_mut().get_events().flush_iter_mut() {
                match event {
                    EventKind::Quit => {
                        running.set(false);
                    }
                    EventKind::MouseDown { x, y, button_code } => {
                        // console_log!("pos:[{},{}]\n",x,y);
                        state.mouse_pos = [x as f32, y as f32];
                        state.mouse_disp = [0, 0];
                        state.mouse_down = true;
                    }
                    EventKind::MouseUp { x, y, button_code } => {
                        state.mouse_pos = [x as f32, y as f32];
                        state.mouse_disp = [0, 0];
                        state.mouse_down = false;
                        // console_log!("pos:[{},{}]\n",x,y);

                        if x < 64 && y < 64 {
                            console_log!("Going fullscreen!\n");
                            go_fullscreen = true;
                        }
                    }
                    EventKind::MouseMove { x, y, dx, dy } => {
                        state.mouse_pos = [x as f32, y as f32];
                        state.mouse_disp = [dx, dy];
                        // console_log!("pos:[{},{}]\n",x,y);
                    }
                    EventKind::TouchDown {
                        x,
                        y,
                        dx,
                        dy,
                        finger_id,
                    } => {
                        state.touch_positions.insert(finger_id, [x, y]);
                    }
                    EventKind::TouchMove {
                        x,
                        y,
                        dx,
                        dy,
                        finger_id,
                    } => {
                        state.touch_positions.insert(finger_id, [x, y]);
                    }
                    EventKind::TouchUp {
                        x,
                        y,
                        dx,
                        dy,
                        finger_id,
                    } => {
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
            writer.draw_text_line(
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
                let aabb = writer.calc_text_aabb(pos_str.as_str(), x, y, 48.);
                let tx = (x - aabb.w / 2.).max(0.).min(bounds.0 as f32 - aabb.w);
                let ty = (y - aabb.h / 2.).max(0.).min(bounds.1 as f32 - aabb.h);
                writer.draw_text_line(pos_str.as_str(), tx, ty, 48., Some(bounds));
            }

            //draw text at centered at id positions
            for (id, &[x, y]) in state.touch_positions.iter() {
                let id_str = format!("id:{},[{},{}]", id, x, y);
                let aabb = writer.calc_text_aabb(id_str.as_str(), x, y, 32.);
                writer.draw_text_line(
                    id_str.as_str(),
                    (x - aabb.w / 2.).max(0.).min(bounds.0 as f32 - aabb.w),
                    (y - aabb.h / 2.).max(0.).min(bounds.1 as f32 - aabb.h),
                    32.,
                    Some(bounds),
                );
            }
        },
    );
    Ok(())
}
