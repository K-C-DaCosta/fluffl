use fluffl::{
    console::*,
    prelude::*,
    window::{event_util::*, *},
    *,
};


use std::{
    io::Write,
    thread,
    ffi::CString,
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
    pub vao: glow::VertexArray,
    pub prog: glow::Program,
    pub t: f32,
}

pub struct FixedString<const N: usize> {
    data: [u8; N],
    len: usize,
}
impl<const N: usize> std::fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl<const N: usize> FixedString<N> {
    pub fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }
    pub fn with_str_cb<F>(mut self, mut cb: F) -> Self
    where
        F: FnMut(&mut [u8]) -> std::io::Result<()>,
    {
        let _ = cb(self.data.as_mut_slice());
        self.recompute_len();
        self
    }

    fn recompute_len(&mut self) {
        let idx = self
            .data
            .iter()
            .enumerate()
            .find(|&(_, &byte)| byte == 0)
            .map(|(idx, _)| idx)
            .unwrap_or(N - 1);
        self.len = idx + 1;
    }

    pub fn with_str(mut self, s: &str) -> Self {
        self.set_str(s);
        self
    }

    pub fn set_str(&mut self, s: &str) {
        let bytes = s.bytes();
        if s.len() < N {
            self.data.iter_mut().zip(bytes).for_each(|(out_b, in_b)| {
                *out_b = in_b;
            });
            self.len = s.bytes().len();
        }
    }

    pub fn as_str(&self) -> &str {
        let byte_slice = &self.data[0..self.len];
        std::str::from_utf8(byte_slice).expect("should not panic")
    }
}

#[test]
pub fn fixed_string_test() {
    let card_number = 0;
    let device_number = 123;
    let fs = FixedString::<32>::new()
        .with_str_cb(|mut bytes| write!(&mut bytes, "hw:{card_number},{device_number}"));
    println!("str = {fs}");
}

fn alsa_hardware_name(card_number: u32, device_number: u32) -> FixedString<32> {
    let fs = FixedString::new()
        .with_str_cb(|mut bytes| write!(bytes, "hw:{card_number},{device_number}"));
    fs
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

    //spawn audio thread
    thread::spawn(audio_thread);


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

    FlufflWindow::run(window, GlobalVariablesOrWhatever { vao, prog, t: 0. }, ml);
}

pub async fn ml(
    win_ptr: FlufflWindowPtr,
    mut running: FlufflRunning,
    main_state: FlufflState<GlobalVariablesOrWhatever>,
) {
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
                console_log!("mouse move: [x:{},y:{},dx:{},dy:{}]\n", x, y, dx, dy);
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
            other => {
                console_log!("other = {:?}\n", other);
            }
        }
    }
}

fn audio_thread(){

    // use alsa::{
    //     Output,
    //     ValueOr,
    //     pcm::{PCM,HwParams,Format,Access,State},  
    // };
    

    // let pcm = alsa::PCM::open(
    //     CString::new("default").unwrap().as_c_str(),
    //     alsa::Direction::Playback,
    //     true,
    // ).unwrap();

    // let hwp = alsa::pcm::HwParams::any(&pcm).expect("hw params failed");
    // hwp.set_channels(1).expect("set_channels(..) failed");
    // hwp.set_rate(44100, ValueOr::Nearest).expect("set_format(..) failed");
    // hwp.set_format(Format::float()).expect("set_format(..) failed");
    // hwp.set_access(Access::RWInterleaved).expect("set_access(..) failed");
    // pcm.hw_params(&hwp).unwrap();

    // let hwp = pcm.hw_params_current().unwrap();
    // let swp = pcm.sw_params_current().unwrap();
    // swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
    // pcm.sw_params(&swp).unwrap();

    // println!("PCM status: {:?}, {:?}", pcm.state(), pcm.hw_params_current().unwrap());
    // let mut outp = Output::buffer_open().unwrap();
    // pcm.dump(&mut outp).unwrap();
    // println!("== PCM dump ==\n{}", outp);

    // let mut buf = [0f32; 1024];
    // for (i, a) in buf.iter_mut().enumerate() {
    //     *a = (i as f32 * 2.0 * ::std::f32::consts::PI / 128.0).sin()*0.1;
    // }
    // let io = pcm.io_f32().unwrap();
    // for _ in 0..2*44100/1024 { // 2 seconds of playback
    //     println!("PCM state: {:?}", pcm.state());
    //     assert_eq!(io.writei(&buf[..]).unwrap(), 1024);
    // }
    // if pcm.state() != State::Running { pcm.start().unwrap() };

    // let mut outp2 = Output::buffer_open().unwrap();
    // pcm.status().unwrap().dump(&mut outp2).unwrap();
    // println!("== PCM status dump ==\n{}", outp2);

    // pcm.drain().unwrap();


}