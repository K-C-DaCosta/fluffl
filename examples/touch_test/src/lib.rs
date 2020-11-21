use fluffl::{
    FlufflError,
    window::{*,event_util::*,glow::*},
};
use std::{
    rc::Rc,
    cell::{Cell,RefCell},
};

static CONFIG:&'static str = "
<window>
<width>800</width> 
<height>600</height>
<resizable>false</resizable> 
<canvas_id>fluffl</canvas_id>
</window>
";

struct AppState{

}
impl AppState{
    fn new()->Self{
        Self{

        }
    }
}
pub async fn fluffl_main() ->Result<(), FlufflError> {
    let window = FlufflWindow::init(CONFIG).expect("init failed");
    let state = AppState::new();
    let gl = window.gl();


    unsafe{
        gl.clear_color(0.2, 0.2, 0.2,1.0);
        gl.viewport(0, 0, 800, 600);
    }

    FlufflWindow::main_loop(window, state,main_loop);
    Ok(())
}

async fn main_loop(win_ptr:FlufflWindowPtr,running:Rc<Cell<bool>>,app_state:Rc<RefCell<AppState>>){
    let gl = win_ptr.window().gl();
    unsafe{
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }
    for event in win_ptr.window_mut().get_events().iter_mut(){
        match event {
            EventKind::Quit=>{
                running.set(false);
            }
            _ => ()
        }
    }
}