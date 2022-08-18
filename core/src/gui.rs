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
    extras::ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
    math::{self, stack::MatStack, ComponentWriter, Mat4, Vec2, Vec4, FP32},
    window::event_util::EventKind,
    GlowGL,
};

const GUI_FRAME_SHADER_SOURCE: &'static str = r"

    #ifndef HEADER
        #version 300 es
        precision mediump float;
        
        uniform vec4 edge_color; 
        uniform vec4 position;
        uniform vec4 bounds;
        uniform vec4 roundness; 
        uniform vec4 background_color;
        uniform vec4 null_color; 
        uniform mat4 modelview;
        
        uniform mat4 proj;  
    #endif

    #ifndef VERTEX_SHADER
        layout(location = 0) in vec4 attr_pos;
        
        out vec4 world_space_pos;

        void main(){
            vec4 world_space = modelview*attr_pos;

            world_space_pos = world_space;  

            //convert worldspace to NDC 
            gl_Position = proj*world_space;
        }
    #endif
    
    #ifndef FRAGMENT_SHADER
        in vec4 world_space_pos;
        out vec4 final_color; 

        

        float sdRoundBox( in vec2 p, in vec2 b, in vec4 r ) 
        {            
            //make sure position is in the top-right
            p-=b;

            //sdf eval starts here 
            r.xy = (p.x>0.0)?r.xy : r.zw;
            r.x  = (p.y>0.0)?r.x  : r.y;
            vec2 q = abs(p)-b+r.x;
            return min(max(q.x,q.y),0.0) + length(max(q,0.0)) - r.x;
        }

        void main(){
            float max_depth = -5.0;
            float band = 0.1;

            vec4 pos = world_space_pos;
            float d = sdRoundBox(pos.xy - position.xy,bounds.xy*0.5,roundness);
            final_color = vec4(0);
            
            final_color += mix(null_color,background_color,smoothstep(max_depth+band,max_depth,d));
            final_color += edge_color*smoothstep(1.0,max_depth+band,d) - edge_color*smoothstep(max_depth+band,max_depth,d);  
            
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

struct ShaderUniforms {
    //vars
    position: Vec4<f32>,
    bounds: Vec4<f32>,
    roundness: Vec4<f32>,
    background_color: Vec4<f32>,
    null_color: Vec4<f32>,
    edge_color: Vec4<f32>,
    modelview: Mat4<f32>,
    proj: Mat4<f32>,

    //locations
    proj_loc: Option<glow::UniformLocation>,
    modelview_loc: Option<glow::UniformLocation>,
    position_loc: Option<glow::UniformLocation>,
    bounds_loc: Option<glow::UniformLocation>,
    background_color_loc: Option<glow::UniformLocation>,
    null_color_loc: Option<glow::UniformLocation>,
    roundness_loc: Option<glow::UniformLocation>,
    edge_color_loc: Option<glow::UniformLocation>,
}
impl ShaderUniforms {
    pub fn new() -> Self {
        Self {
            position: Vec4::zero(),
            bounds: Vec4::zero(),
            roundness: Vec4::zero(),
            background_color: Vec4::zero(),
            null_color: Vec4::zero(),
            modelview: Mat4::identity(),
            proj: Mat4::identity(),
            edge_color: Vec4::zero(),

            proj_loc: None,
            modelview_loc: None,
            position_loc: None,
            bounds_loc: None,
            background_color_loc: None,
            null_color_loc: None,
            roundness_loc: None,
            edge_color_loc: None,
        }
    }
    pub fn with_location_hooks(mut self, gl: &GlowGL, prog: &OglProg) -> Self {
        let prog = prog.prog();
        unsafe {
            self.modelview_loc = gl.get_uniform_location(prog, "modelview");
            self.proj_loc = gl.get_uniform_location(prog, "proj");
            self.position_loc = gl.get_uniform_location(prog, "position");
            self.bounds_loc = gl.get_uniform_location(prog, "bounds");
            self.background_color_loc = gl.get_uniform_location(prog, "background_color");
            self.null_color_loc = gl.get_uniform_location(prog, "null_color");
            self.roundness_loc = gl.get_uniform_location(prog, "roundness");
            self.edge_color_loc = gl.get_uniform_location(prog, "edge_color");
        }
        self
    }

    //uploads all uniforms to prog
    fn update(&self, gl: &GlowGL, prog: &OglProg) {
        unsafe {
            gl.uniform_4_f32_slice(self.roundness_loc.as_ref(), self.roundness.as_slice());
            gl.uniform_4_f32_slice(self.null_color_loc.as_ref(), self.null_color.as_slice());
            gl.uniform_4_f32_slice(
                self.background_color_loc.as_ref(),
                self.background_color.as_slice(),
            );
            gl.uniform_4_f32_slice(self.bounds_loc.as_ref(), self.bounds.as_slice());
            gl.uniform_4_f32_slice(
                self.background_color_loc.as_ref(),
                self.background_color.as_slice(),
            );
            gl.uniform_4_f32_slice(self.position_loc.as_ref(), self.position.as_slice());
            gl.uniform_matrix_4_f32_slice(
                self.modelview_loc.as_ref(),
                true,
                self.modelview.as_slice(),
            );
            gl.uniform_matrix_4_f32_slice(self.proj_loc.as_ref(), true, self.proj.as_slice());
            gl.uniform_4_f32_slice(self.edge_color_loc.as_ref(), self.edge_color.as_slice());
        }
    }

    fn set_edge_color(&mut self, gl: &GlowGL, prog: &OglProg, col: Vec4<f32>) {
        prog.bind(true);
        self.edge_color = col;
        unsafe {
            gl.uniform_4_f32_slice(self.edge_color_loc.as_ref(), self.edge_color.as_slice());
        }
    }

    fn set_roundness(&mut self, gl: &GlowGL, prog: &OglProg, tl: f32, tr: f32, bl: f32, br: f32) {
        prog.bind(true);
        // br tr bl tl
        self.roundness = Vec4::from_array([br, tr, bl, tl]);

        unsafe {
            gl.uniform_4_f32_slice(self.roundness_loc.as_ref(), self.roundness.as_slice());
        }
    }

    fn update_proj(&mut self, gl: &GlowGL, prog: &OglProg, window_width: f32, window_height: f32) {
        prog.bind(true);
        self.proj = math::calc_ortho_window_f32(window_width, window_height);
        unsafe {
            gl.uniform_matrix_4_f32_slice(self.proj_loc.as_ref(), true, self.proj.as_slice());
        }
    }

    fn set_null_color(&mut self, gl: &GlowGL, prog: &OglProg, null_color: Vec4<f32>) {
        prog.bind(true);
        self.null_color = null_color;
        unsafe {
            gl.uniform_4_f32_slice(self.null_color_loc.as_ref(), self.null_color.as_slice());
        }
    }

    fn set_background_color(&mut self, gl: &GlowGL, prog: &OglProg, bgcolor: Vec4<f32>) {
        prog.bind(true);
        self.background_color = bgcolor;
        unsafe {
            gl.uniform_4_f32_slice(
                self.background_color_loc.as_ref(),
                self.background_color.as_slice(),
            );
        }
    }

    fn set_bounds(&mut self, gl: &GlowGL, prog: &OglProg, w: f32, h: f32) {
        prog.bind(true);
        self.bounds = Vec4::from_array([w, h, 0.0, 0.]);
        unsafe {
            gl.uniform_4_f32_slice(self.bounds_loc.as_ref(), self.bounds.as_slice());
        }
    }

    fn set_position(&mut self, gl: &GlowGL, prog: &OglProg, pos: Vec4<f32>) {
        prog.bind(true);
        self.position = pos;
        let scale = math::scale4(self.bounds);
        let translate = math::translate4(self.position);
        self.modelview = translate * scale;
        unsafe {
            gl.uniform_4_f32_slice(self.position_loc.as_ref(), self.position.as_slice());
            gl.uniform_matrix_4_f32_slice(
                self.modelview_loc.as_ref(),
                true,
                self.modelview.as_slice(),
            );
        }
    }
}

pub struct GUIManager {
    gl: GlowGL,

    gui_shader_program: ogl::OglProg,

    uniforms: ShaderUniforms,

    stack: MatStack<f32>,

    unit_square_vao: ogl::OglArray,

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
        let gui_shader_program = ogl::OglProg::compile_program(&gl, GUI_FRAME_SHADER_SOURCE)
            .expect("GUI SHADER CODE FAILED TO COMPILE");

        //collect uniforms

        //write-unit-square to vector
        let mut vec_data = Vec::<f32>::new();

        //write unit square into buffer
        Self::write_rectangle(
            &mut vec_data,
            Vec4::from_array([0.0, 0.0, 0.0, 1.0]),
            1.0,
            1.0,
        );

        // println!("{:?}", vec_data);

        let buf = ogl::OglBuf::<Vec<f32>>::new(&gl)
            .with_target(glow::ARRAY_BUFFER)
            .with_usage(glow::STATIC_DRAW)
            .with_num_comps(4)
            .with_data(vec_data)
            .with_index(0)
            .build();

        let unit_square_vao =
            ogl::OglArray::new(&gl).init(vec![BufferPair::new("verts", Box::new(buf))]);

        let mut uniforms = ShaderUniforms::new().with_location_hooks(&gl, &gui_shader_program);

        uniforms.set_position(&gl, &gui_shader_program, Vec4::from_array([0., 0., 0., 1.]));

        uniforms.set_bounds(&gl, &gui_shader_program, 300.0, 400.0);

        uniforms.set_background_color(
            &gl,
            &gui_shader_program,
            Vec4::from_rgba_hex_color_u32(0xA66CFF00),
        );

        uniforms.set_null_color(
            &gl,
            &gui_shader_program,
            // Vec4::from_array([1.0, 0.1, 0.1, 1.]),
            Vec4::from_array([0.1, 0.1, 0.1, 1.]),
        );

        uniforms.set_roundness(&gl, &gui_shader_program, 1., 1., 20., 20.);

        uniforms.set_edge_color(
            &gl,
            &gui_shader_program,
            Vec4::from_rgba_hex_color_u32(0xB1E1FF00),
        );

        let manager = Self {
            uniforms,
            unit_square_vao,
            gui_shader_program,
            focused_component: None,
            gui_component_tree: LinearTree::new(),
            key_to_node_table: HashMap::new(),
            component_key_state: 0,
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),
            stack: MatStack::new(),
            gl,
        };

        manager
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }

    pub fn render(&mut self, window_width: f32, window_height: f32) {
        let gl = self.gl.clone();

        self.gui_shader_program.bind(true);

        self.uniforms
            .update_proj(&gl, &self.gui_shader_program, window_width, window_height);

        while let Some(event) = self.window_events.pop_front() {
            match event {
                EventKind::Resize { width, height } => {
                    self.uniforms.update_proj(
                        &gl,
                        &self.gui_shader_program,
                        width as f32,
                        height as f32,
                    );
                }
                EventKind::MouseMove { x, y, .. } => {
                    self.uniforms.set_position(
                        &gl,
                        &self.gui_shader_program,
                        Vec4::from_array([x as f32, y as f32, 0.0, 0.0]),
                    );
                }
                _ => (),
            }
        }

        self.unit_square_vao.bind(true);
        unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }

    fn gen_component_key(&mut self) -> GuiComponentKey {
        let generated_key = GuiComponentKey(self.component_key_state);

        //increment state
        self.component_key_state += 1;

        generated_key
    }

    fn write_rectangle(component_list: &mut Vec<f32>, x0: Vec4<f32>, w: f32, h: f32) {
        let mut writer = ComponentWriter::from(component_list);

        let dx = Vec4::from_array([w, 0.0, 0.0, 0.0]);
        let dy = Vec4::from_array([0.0, h, 0.0, 0.0]);
        let tl = x0;
        let tr = x0 + dx;
        let bl = x0 + dy;
        let br = x0 + dx + dy;

        writer.write(&tl);
        writer.write(&tr);
        writer.write(&bl);

        writer.write(&tr);
        writer.write(&br);
        writer.write(&bl);
    }
}
