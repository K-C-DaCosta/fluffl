use fluffl::{
    console::*,
    prelude::*,
    window::{event_util::*, *},
    *,
};

static VERTEX_SHADER_SOURCE: &'static str = r"
    #version 300 es
    precision mediump float;

    layout(location = 0) in vec4 attr_pos;
    layout(location = 1) in vec4 attr_color;

    uniform float t; 
    out vec4 color;

    void main(){
        color =  attr_color;
        float t = (sin(t)+1.0)*0.5 - 0.5;
        float s = sin(t);
        float c = cos(t);
        mat3 rot = mat3(
            vec3(c,s,0.0),
            vec3(-s,c,0.0),
            vec3(0.,0.,1.0)
        );
        gl_Position = vec4(rot*attr_pos.xyz,1.0);
    }
";

static FRAGMENT_SHADER_SOURCE: &'static str = r"
    #version 300 es
    precision mediump float;
    in vec4 color; 
    out vec4 final_color;
    void main(){
        final_color = color;
    }
";

pub struct GlobalVariablesOrWhatever {
    pub vao: VertexArray,
    pub prog: Program,
    pub t: f32,
}

/// A simple triangle demo, nothing but opengl and some basic event handling
#[fluffl(Debug)]
pub async fn main() {
    let config_text = "
        <window>
            <width>512</width>
            <height>512</height>
            <title>triangle</title>
        </window>";
    let window = FlufflWindow::init(config_text).unwrap();

    let (win_w, win_h) = window.get_bounds();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.0);
        window.gl().viewport(0, 0, win_w as i32, win_h as i32);
    }

    let (vao, prog) = unsafe {
        let vert_source = String::from(VERTEX_SHADER_SOURCE);
        let frag_source = String::from(FRAGMENT_SHADER_SOURCE);
        let geo_data: Vec<f32> = vec![1., 0., 0., 1., 0., 1., 0., 1., -1., 0., 0., 1.];
        let color_data: Vec<f32> = vec![1., 0., 0., 1., 0., 1., 0., 1., 0., 0., 1., 1.];
        let gl = window.gl();

        let vert_shader: glow::Shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vert_shader, vert_source.as_str());
        gl.compile_shader(vert_shader);
        if gl.get_shader_compile_status(vert_shader) == false {
            console_log!(
                "Vert shader Failed!\n Reason:\n{}\n",
                gl.get_shader_info_log(vert_shader)
            );
        }

        let frag_shader: glow::Shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(frag_shader, frag_source.as_str());
        gl.compile_shader(frag_shader);
        if gl.get_shader_compile_status(frag_shader) == false {
            console_log!(
                "Frag shader Failed!\n Reason:\n{}\n",
                gl.get_shader_info_log(frag_shader)
            );
        }

        let program: glow::Program = gl.create_program().unwrap();
        gl.use_program(Some(program));
        gl.attach_shader(program, vert_shader);
        gl.attach_shader(program, frag_shader);
        gl.link_program(program);

        gl.delete_shader(vert_shader);
        gl.delete_shader(frag_shader);

        if gl.get_program_link_status(program) == false {
            console_log!(
                "Program failed to link!\n Reason:\n{}\n",
                gl.get_program_info_log(program)
            );
        }

        let vao: glow::VertexArray = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let buffer: glow::Buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

        //upload geo data gpu
        let data = &geo_data[..];
        let data_as_bytes: &[u8] =
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_as_bytes, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        let buffer: glow::Buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        //upload vert color data to gpu
        let data = &color_data[..];
        let data_as_bytes: &[u8] =
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_as_bytes, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        (vao, program)
    };

    FlufflWindow::main_loop(
        window,
        GlobalVariablesOrWhatever { vao, prog, t: 0. },
        |win_ptr, mut running, main_state| async move {
            // let audio_ctx = win_ptr.window().audio_context().clone();
            let gl = win_ptr.window().gl();

            //increment t
            main_state.borrow_mut().t += 0.01;

            unsafe {
                {
                    let main_state = main_state.borrow();
                    let t_location = gl.get_uniform_location(main_state.prog, "t");

                    let t = main_state.t;
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    gl.use_program(Some(main_state.prog.clone()));

                    gl.bind_vertex_array(Some(main_state.vao.clone()));
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);

                    gl.uniform_1_f32(t_location.as_ref(), t);
                    gl.use_program(None);
                }
            }

            for event in win_ptr.window_mut().get_events().flush_iter_mut() {
                match event {
                    EventKind::Quit => {
                        running.set(false);
                    }
                    EventKind::Resize { width, height } => unsafe {
                        gl.viewport(0, 0, width, height);
                    },
                    EventKind::MouseMove { x, y, dx, dy } => {
                        console_log!("mouse move: [x:{},y:{},dx:{},dy:{}\n", x, y, dx, dy);
                    }
                    EventKind::MouseUp { button_code, x, y } => {
                        console_log!("mouse up at: [x:{},y:{}]\n", x, y);
                        console_log!("{}\n", button_code);
                    }
                    EventKind::MouseDown { button_code, x, y } => {
                        console_log!("mouse down at: [x:{},y:{}]\n", x, y);
                        console_log!("{}\n", button_code);
                    }
                    EventKind::MouseWheel { button_code } => {
                        console_log!("{}\n", button_code);
                    }
                    _ => (),
                }
            }
        },
    );
}
