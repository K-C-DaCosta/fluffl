use fluffl::{
    extras::shapes::ShapePainter2D,
    window::{event_util::*, glow, glow::*, *},
    FlufflError, GlowGL,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

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

pub struct BrickAppState {
    mouse_pos: [f32; 2],
    painter: ShapePainter2D,
    time:f32,
}
impl BrickAppState {
    fn new(gl: &GlowGL) -> Self {
        Self {
            mouse_pos: [0.; 2],
            painter: ShapePainter2D::new(gl),
            time:0.0, 
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
    
        let roundness = 32.*(brick_state.time.sin()+1.0)*0.5 ; 
        brick_state.painter.draw_rectangle(
            &mouse_pos[..],
            &[mouse_pos[0] + 128., mouse_pos[1] -0.][..],
            &[1.,0.,0.,1.][..],
            16.0,
            4.0,
        );
        brick_state.time+=0.05;
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
