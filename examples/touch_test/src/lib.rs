use fluffl::{
    extras::text_writer::*,
    io::*,
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
<resizable>false</resizable> 
<canvas_id>fluffl</canvas_id>
</window>
";

struct AppState {
    writer: Option<TextWriter>,
    touch_positions: HashMap<i32, [f32; 2]>,
}

impl AppState {
    fn new() -> Self {
        Self {
            writer: None,
            touch_positions: HashMap::new(),
        }
    }
}
pub async fn fluffl_main() -> Result<(), FlufflError> {
    let window = FlufflWindow::init(CONFIG).expect("init failed");
    let gl = window.gl();
    let mut app_state = AppState::new();

    // load font called "plasmatic"
    let data = load_file!("../../wasm_bins/resources/plasmatic.bcode").expect("load failed");
    let atlas = HieroAtlas::deserialize(data)
        .ok()
        .expect("font unpacked failed");
    let writer = TextWriter::new(&gl).with_atlas(atlas).build();
    app_state.writer = Some(writer);

    unsafe {
        gl.clear_color(0.2, 0.2, 0.2, 1.0);
        gl.viewport(0, 0, 800, 600);
    }

    FlufflWindow::main_loop(window, app_state, main_loop);
    Ok(())
}

async fn main_loop(
    win_ptr: FlufflWindowPtr,
    running: Rc<Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
) {
    let bounds = win_ptr.window().get_bounds();
    let gl = win_ptr.window().gl();

    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }
    let state =  &mut *app_state.borrow_mut();
    let writer =state.writer.as_mut().unwrap(); 

    for event in win_ptr.window_mut().get_events().iter_mut() {
        match event {
            EventKind::Quit => {
                running.set(false);
            }
            EventKind::MouseDown {x,y,button_code} => {
                
            }
            EventKind::MouseUp { x,y,button_code} =>{

            }
            EventKind::MouseMove { x,y,dx,dy}=>{
            
            }
            EventKind::TouchDown { x,y,dx,dy,finger_id} =>{
                state.touch_positions.insert(finger_id , [x,y]);
            }
            EventKind::TouchMove { x,y,dx,dy,finger_id} => {
                state.touch_positions.insert(finger_id , [x,y]);
            }
            EventKind::TouchUp { x,y,dx,dy,finger_id}=>{
                state.touch_positions.remove(&finger_id);
            }
            _ => (),
        }
    }
    //draw text at centered at id positions
    for (id, &[x,y] ) in state.touch_positions.iter() {
        let id_str = format!("id:{}",id);
        let aabb = writer.calc_text_aabb(id_str.as_str(), x, y, 32.);
        writer.draw_text_line(id_str.as_str(), x-aabb.w/2., y-aabb.h/2., 32., Some(bounds));
    } 

}
