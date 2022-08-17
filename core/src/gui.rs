use std::{
    collections::{HashMap, VecDeque},
    fmt,
    ops::DerefMut,
    vec,
};

use arrayvec::Array;
use glow::HasContext;

use crate::{
    collections::{
        flat_nary_tree::{LinearTree, NodeID},
        linked_list::{LinkedList, PackedLinkedList},
    },
    extras::ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder},
    math::{self, ComponentWriter, Mat4, Vec2, Vec4, FP32},
    window::event_util::EventKind,
    GlowGL,
};

const GUI_SHADER_SOURCE: &'static str = r"
    #ifndef HEADER
        #version 300 es
        precision mediump float;
        uniform mat4 modelview;
        uniform mat4 proj;  
    #endif

    #ifndef VERTEX_SHADER
        layout(location = 0) in vec4 attr_pos;
        void main(){
            gl_Position = proj*modelview*attr_pos;
        }
    #endif
    
    #ifndef FRAGMENT_SHADER
        out vec4 final_color; 
        void main(){
            final_color = vec4(0.3,0.2,0.1,1.0);
        }
    #endif
";

pub mod components;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GuiComponentKey(u32);

impl fmt::Display for GuiComponentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

pub struct GUIManager {
    gl: GlowGL,

    gui_shader_program: ogl::OglProg,
    proj_loc: Option<glow::UniformLocation>,
    modelview_loc: Option<glow::UniformLocation>,

    vao: ogl::OglArray,

    component_key_state: u32,

    focused_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<GuiComponentKey>,

    ///stores the position given component key
    key_to_node_table: HashMap<GuiComponentKey, NodeID>,

    component_signal_queue: VecDeque<components::ComponentEventSignal>,

    window_events: VecDeque<EventKind>,
}

impl GUIManager {
    pub fn new(gl: GlowGL) -> Self {
        //compile the shader
        let gui_shader_program = ogl::OglProg::compile_program(&gl, GUI_SHADER_SOURCE)
            .expect("GUI SHADER CODE FAILED TO COMPILE");

        //collect uniforms
        let [modelview_loc, proj_loc] = unsafe {
            [
                gl.get_uniform_location(gui_shader_program.prog(), "modelview"),
                gl.get_uniform_location(gui_shader_program.prog(), "proj"),
            ]
        };

        let x0 = Vec4::from_array([0.0, 0.0, 0.0, 1.0]);
        let dx = Vec4::from_array([100.0, 0.0, 0.0, 0.0]);
        let dy = Vec4::from_array([0.0, 100.0, 0.0, 0.0]);

        let tl = x0;
        let tr = x0 + dx;
        let bl = x0 + dy;
        let br = x0 + dx*2. + dy;

        let mut vec_data = Vec::<f32>::new();
        let mut writer = ComponentWriter::from(&mut vec_data);

        writer.write(&tl);
        writer.write(&tr);
        writer.write(&bl);

        writer.write(&tr);
        writer.write(&br);
        writer.write(&bl);

        println!("{:?}", vec_data);

        let buf = ogl::OglBuf::<Vec<f32>>::new(&gl)
            .with_target(glow::ARRAY_BUFFER)
            .with_usage(glow::DYNAMIC_DRAW)
            .with_num_comps(4)
            .with_data(vec_data)
            .with_index(0)
            .build();

        

        let vao = ogl::OglArray::new(&gl).init(vec![BufferPair::new("verts", Box::new(buf))]);

        let x = vao.get("verts").unwrap();

        Self {
            modelview_loc,
            proj_loc,
            vao,
            gui_shader_program,
            focused_component: None,
            gui_component_tree: LinearTree::new(),
            key_to_node_table: HashMap::new(),
            component_key_state: 0,
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),

            gl,
        }
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }
    pub fn render(&mut self, window_width: f32, window_height: f32) {
        let mut translate = Vec4::<f32>::zero();

        let gl = self.gl.clone();

        self.gui_shader_program.bind(true);

        let projmat = math::calc_ortho_window_f32(window_width, window_height);

        while let Some(event) = self.window_events.pop_front() {
            if let EventKind::MouseMove { x, y, .. } = event {
                *translate.deref_mut() = [x as f32, y as f32, 0.0, 0.0];
                let modelview = math::translate4(translate);

                unsafe {
                    gl.uniform_matrix_4_f32_slice(
                        self.modelview_loc.as_ref(),
                        true,
                        modelview.as_slice(),
                    );
                }
            }
        }

        self.vao.bind(true);

        unsafe {
            gl.uniform_matrix_4_f32_slice(self.proj_loc.as_ref(), true, projmat.as_slice());
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }

    fn gen_component_key(&mut self) -> GuiComponentKey {
        let generated_key = GuiComponentKey(self.component_key_state);

        //increment state
        self.component_key_state += 1;

        generated_key
    }
}
