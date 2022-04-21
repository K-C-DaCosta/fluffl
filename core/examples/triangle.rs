use fluffl::{
    prelude::*, 
    audio::*,
    console::*,
    extras::{
        audio::music_player::*,
        audio::{ogg::*, *},
    },
    io::*,
    window::{event_util::*, glow::*, *},
    *,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

type ShortBufferType = OggBuffer;
type ShortMusicPlayer = MusicPlayer<ShortBufferType>;
type ShortDeviceCB = DeviceCB<ShortMusicPlayer>;
type ShortDeviceContext = FlufflAudioDeviceContext<ShortDeviceCB, ShortMusicPlayer>;



static VERTEX_SHADER_SOURCE:&'static str = r"
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


static FRAGMENT_SHADER_SOURCE:&'static str = r"
    #version 300 es
    precision mediump float;
    in vec4 color; 
    out vec4 final_color;
    void main(){
        final_color = color;
    }
";

pub struct MainState {
    pub device_list: Vec<ShortDeviceContext>,
    pub current_device: usize,
    pub vao: VertexArray,
    pub prog: Program,
    pub t: f32,
}
impl MainState {
    pub fn next_device(&mut self) {
        let len = self.device_list.len();
        self.current_device = (self.current_device + 1) % len;
    }
    pub fn prev_device(&mut self) {
        let len = self.device_list.len();
        self.current_device = (self.current_device + len - 1) % len;
    }
    pub fn get_current_device_depricated(&self) -> ShortDeviceContext {
        self.device_list[self.current_device].clone()
    }

    pub fn get_current_device<F>(&self, mut cb: F)
    where
        F: FnMut(&ShortDeviceContext),
    {
        cb(&self.device_list[self.current_device])
    }

    pub fn delete_current_track(&mut self) {
        if self.device_list.len() > 0 {
            let device_id = self.current_device;
            self.device_list.remove(device_id);
            //clamp device index
            self.current_device = self.current_device.max(self.device_len()).min(0);
        }
    }

    pub fn device_len(&self) -> usize {
        self.device_list.len()
    }

    pub async fn load_music(
        &mut self,
        path: &str,
        audio_context: Arc<RefCell<FlufflAudioContext>>,
    ) -> Result<(), FlufflError> {
        let wav = {
            let wav_data: Vec<u8> = load_file!(path)?;
            ogg::OggFile::new()
                .with_data(wav_data)
                .parse()
                .ok()
                .unwrap()
        };
        let music_core: AudioDeviceCore<ShortDeviceCB, _> = AudioDeviceCore::new()
            .with_specs(DesiredSpecs {
                sample_rate: wav.sample_rate(),
                channels: Some(2),
                buffer_size: Some(4096),
            })
            .with_state(MusicPlayer {
                ticks: 0,
                state: PlayState::RampUp(48000),
                volume: 0.7,
                music_src: wav.into(),
                repeat_track: false,
            })
            .with_callback(music_callback);
        let music_device = FlufflAudioDeviceContext::new(music_core, audio_context);
        self.device_list.push(music_device);
        Ok(())
    }
}


#[fluffl(Debug)]
pub async fn main() {
    let config_text = "
        <window>
            <width>512</width>
            <height>512</height>
            <title>my_app</title>
        </window>";
    let window = FlufflWindow::init(config_text).unwrap();

    let (win_w,win_h) = window.get_bounds();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.0);
        window.gl().viewport(0, 0, win_w as i32, win_h as i32);
    }

    let (vao, prog) = unsafe {
        let vert_source = String::from(VERTEX_SHADER_SOURCE);
        let frag_source = String::from(FRAGMENT_SHADER_SOURCE);
        let geo_data:Vec<f32> = vec![
            1., 0., 0., 1.,
            0., 1., 0., 1.,
           -1., 0., 0., 1.
        ];
        let color_data:Vec<f32> = vec![
            1., 0., 0., 1.,
            0., 1., 0., 1.,
            0., 0., 1., 1.
        ];
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
        MainState {
            device_list: Vec::new(),
            current_device: 0,
            vao,
            prog,
            t: 0., 
        },
        |win_ptr,running,main_state| async move {
            // let audio_ctx = win_ptr.window().audio_context().clone();
            let gl = win_ptr.window().gl();

            //increment t 
            main_state.inner.borrow_mut().t+=0.01;
        
            unsafe {
                
                {
                    let main_state = main_state.inner.borrow(); 
                    let t_location = gl.get_uniform_location(main_state.prog,"t");

                    let t = main_state.t; 

                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    gl.use_program(Some(main_state.prog.clone()));
                  

                    
                    gl.bind_vertex_array(Some(main_state.vao.clone()));
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    
                    
                    gl.uniform_1_f32(t_location.as_ref(),t);
                    gl.use_program(None);

                }
               
                
            }
        
            for event in win_ptr.window_mut().get_events().flush_iter_mut() {
                match event {
                    EventKind::Quit =>  {
                        running.set(false);
                    },
                    // EventKind::KeyDown { code } => {
                        

                    //     if let KeyCode::NUM_1 = code {
                    //         // let _ = main_state
                    //         //     .inner
                    //         //     .borrow_mut()
                    //         //     .load_music("./resources/st3.ogg", audio_ctx.clone())
                    //         //     .await;
                    //     }
                    //     if let KeyCode::NUM_2 = code {
                    //         // main_state
                    //         //     .borrow_mut()
                    //         //     .load_music("./resources/out.wav", audio_ctx.clone())
                    //         //     .await;
                    //     }
                    //     if let KeyCode::KEY_A = code {
                    //         // main_state
                    //         //     .inner
                    //         //     .borrow_mut()
                    //         //     .socket
                    //         //     .send("hello world from wasm client!\n".as_bytes())
                    //         //     .ok()
                    //         //     .unwrap();
                    //     }
        
                    //     if let KeyCode::KEY_R = code {
                    //         main_state.inner.borrow().get_current_device(|device| {
                    //             device.modify_state(|state| {
                    //                 state.map(|mp| {
                    //                     mp.state = PlayState::RampUp(12000);
                    //                     mp.ticks = 0;
                    //                     mp.music_src.seek_to_start();
                    //                 });
                    //             })
                    //         });
                    //     }
        
                    //     if let KeyCode::PAGE_UP = code {
                    //         main_state.inner.borrow().get_current_device(|device| {
                    //             device.modify_state(|state| {
                    //                 state.map(|mp| {
                    //                     mp.volume = (mp.volume + 0.1).min(1.0).max(0.0);
                    //                 });
                    //             })
                    //         })
                    //     }
        
                    //     if let KeyCode::PAGE_DOWN = code {
                    //         main_state.inner.borrow().get_current_device(|device| {
                    //             device.modify_state(|state| {
                    //                 state.map(|mp| {
                    //                     mp.volume = (mp.volume - 0.1).min(1.0).max(0.0);
                    //                 });
                    //             })
                    //         });
                    //     }
        
                    //     if let KeyCode::SPACE = code {
                    //         if main_state.inner.borrow().device_len() > 0 {
                    //             main_state.inner.borrow().get_current_device(|dev| {
                    //                 dev.modify_state(|state| {
                    //                     state.map(|s| {
                    //                         if let PlayState::Paused = s.state {
                    //                             s.ticks = 0;
                    //                             s.state = PlayState::RampUp(12000);
                    //                         }
                    //                     });
                    //                 });
                    //                 dev.resume();
                    //             });
                    //         }
                    //     }
        
                    //     if let KeyCode::DELETE = code {
                    //         main_state.inner.borrow_mut().delete_current_track();
                    //     }
        
                    //     if let KeyCode::KEY_Y = code {
                    //         main_state.inner.borrow().get_current_device(|device| {
                    //             device.modify_state(|state| {
                    //                 state.map(|s| {
                    //                     if let PlayState::Playing = s.state {
                    //                         s.ticks = 0;
                    //                         s.state = PlayState::RampDown(12000);
                    //                     }
                    //                 });
                    //             });
                    //             device.resume();
                    //         });
                    //     }
        
                    //     if let KeyCode::ARROW_RIGHT = code {
                    //         main_state.inner.borrow_mut().next_device();
                    //     }
                    //     if let KeyCode::ARROW_LEFT = code {
                    //         main_state.inner.borrow_mut().prev_device();
                    //     }
        
                    //     let ncode: i128 = code.into();
                    //     console_log!("char = {}\n", (ncode as u8 as char).to_lowercase());
                    //     // if (ncode >= KeyCode::KEY_A.into()) && (ncode <= KeyCode::KEY_Z.into()) {
                    //     // }
                    // }
                    // EventKind::MouseMove { x, y, dx, dy } => {
                    //     console_log!("mouse move: [x:{},y:{},dx:{},dy:{}\n", x, y, dx, dy);
                    // }
                    // EventKind::MouseUp { button_code, x, y } => {
                    //     console_log!("mouse up at: [x:{},y:{}]\n", x, y);
                    //     console_log!("{}\n", button_code);
                    // }
                    // EventKind::MouseDown { button_code, x, y } => {
                    //     console_log!("mouse down at: [x:{},y:{}]\n", x, y);
                    //     console_log!("{}\n", button_code);
                    // }
                    // EventKind::MouseWheel { button_code } => {
                    //     console_log!("{}\n", button_code);
                    // }
                    _ => (),
                }
            }
        });
}


