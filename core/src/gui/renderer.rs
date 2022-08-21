use super::*;
use glow::HasContext;


mod shader_sources;
use shader_sources::*;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum RendererShaderKind {
    Frame = 0,
}

struct ShaderUniforms {
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

    fn set_edge_color(&self, gl: &GlowGL, prog: &OglProg, col: Vec4<f32>) {
        prog.bind(true);
        unsafe {
            gl.uniform_4_f32_slice(self.edge_color_loc.as_ref(), col.as_slice());
        }
    }

    fn set_roundness(&self, gl: &GlowGL, prog: &OglProg, tl: f32, tr: f32, bl: f32, br: f32) {
        prog.bind(true);
        let roundness = Vec4::from_array([br, tr, bl, tl]);
        unsafe {
            gl.uniform_4_f32_slice(self.roundness_loc.as_ref(), roundness.as_slice());
        }
    }

    fn recompute_proj(&self, gl: &GlowGL, prog: &OglProg, window_width: f32, window_height: f32) {
        prog.bind(true);
        let proj = math::calc_ortho_window_f32(window_width, window_height);
        unsafe {
            gl.uniform_matrix_4_f32_slice(self.proj_loc.as_ref(), true, proj.as_slice());
        }
    }

    fn set_null_color(&self, gl: &GlowGL, prog: &OglProg, null_color: Vec4<f32>) {
        prog.bind(true);
        let null_color = null_color;
        unsafe {
            gl.uniform_4_f32_slice(self.null_color_loc.as_ref(), null_color.as_slice());
        }
    }

    fn set_background_color(&self, gl: &GlowGL, prog: &OglProg, bgcolor: Vec4<f32>) {
        prog.bind(true);
        let background_color = bgcolor;
        unsafe {
            gl.uniform_4_f32_slice(
                self.background_color_loc.as_ref(),
                background_color.as_slice(),
            );
        }
    }

    fn set_bounds(&self, gl: &GlowGL, prog: &OglProg, w: f32, h: f32) {
        prog.bind(true);
        let bounds = Vec4::from_array([w, h, 0.0, 0.]);
        unsafe {
            gl.uniform_4_f32_slice(self.bounds_loc.as_ref(), bounds.as_slice());
        }
    }

    fn set_position(&self, gl: &GlowGL, prog: &OglProg, pos: Vec4<f32>, bounds: Vec4<f32>) {
        prog.bind(true);
        let position = pos;
        let scale = math::scale4(bounds);
        let translate = math::translate4(position);
        let modelview = translate * scale;
        unsafe {
            gl.uniform_4_f32_slice(self.position_loc.as_ref(), position.as_slice());
            gl.uniform_matrix_4_f32_slice(self.modelview_loc.as_ref(), true, modelview.as_slice());
        }
    }
}

pub struct RenderBuilder<'a> {
    gl: &'a GlowGL,
    prog: &'a OglProg,
    unit_square_vao: &'a ogl::OglArray,
    uniforms: &'a ShaderUniforms,
}
impl<'a> RenderBuilder<'a> {
    pub fn set_edge_color<T>(self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        let color = Vec4::from(color);
        self.uniforms.set_edge_color(self.gl, self.prog, color);
        self
    }
    pub fn set_roundness(self, tl: f32, tr: f32, bl: f32, br: f32) -> Self {
        self.uniforms
            .set_roundness(self.gl, self.prog, tl, tr, bl, br);
        self
    }

    pub fn set_roundness_vec<T>(self, roundness: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        let r = Vec4::from(roundness);
        self.set_roundness(r[0], r[1], r[2], r[3])
    }

    pub fn set_null_color<T>(self, null_color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        let null_color = Vec4::from(null_color);
        self.uniforms.set_null_color(self.gl, self.prog, null_color);
        self
    }

    pub fn set_background_color<T>(self, bgcolor: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        let color = Vec4::from(bgcolor);

        self.uniforms
            .set_background_color(self.gl, self.prog, color);
        self
    }

    pub fn set_bounds<T>(self, bounds: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        let bounds = Vec2::from(bounds);
        self.uniforms
            .set_bounds(self.gl, self.prog, bounds[0], bounds[1]);
        self
    }

    pub fn set_position<A, B>(self, pos: A, bounds: B) -> Self
    where
        Vec4<f32>: From<A> + From<B>,
    {
        let pos = Vec4::from(pos);
        let bounds = Vec4::from(bounds);
        self.uniforms.set_position(self.gl, self.prog, pos, bounds);
        self
    }

    pub fn set_window(self, window_width: f32, window_height: f32) -> Self {
        self.uniforms
            .recompute_proj(self.gl, self.prog, window_width, window_height);
        self
    }

    /// draw call happens here
    pub fn render(self) {
        self.unit_square_vao.bind(true);
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

pub struct GuiRenderer {
    shader_program_table: HashMap<RendererShaderKind, OglProg>,
    unit_square_vao: ogl::OglArray,
    uniforms: ShaderUniforms,
}

impl GuiRenderer {
    pub fn new(gl: &GlowGL) -> Self {
        //compile the shader
        let frame_program = ogl::OglProg::compile_program(&gl, FRAME_SHADER_SOURCE)
            .expect("GUI SHADER CODE FAILED TO COMPILE");

        //write-unit-square to vector
        let mut vec_data = Vec::<f32>::new();

        //write unit square into buffer
        write_rectangle(
            &mut vec_data,
            Vec4::from_array([0.0, 0.0, 0.0, 1.0]),
            1.0,
            1.0,
        );

        let buf = ogl::OglBuf::<Vec<f32>>::new(&gl)
            .with_target(glow::ARRAY_BUFFER)
            .with_usage(glow::STATIC_DRAW)
            .with_num_comps(4)
            .with_data(vec_data)
            .with_index(0)
            .build();

        let unit_square_vao =
            ogl::OglArray::new(&gl).init(vec![BufferPair::new("verts", Box::new(buf))]);

        let uniforms = ShaderUniforms::new().with_location_hooks(&gl, &frame_program);

        let bounds = Vec4::from_array([300.0, 400.0, 0., 0.]);

        uniforms.set_bounds(&gl, &frame_program, bounds[0], bounds[1]);

        uniforms.set_position(
            &gl,
            &frame_program,
            Vec4::from_array([0., 0., 0., 1.]),
            bounds,
        );

        uniforms.set_background_color(&gl, &frame_program, Vec4::rgba_u32(0xA66CFF00));

        uniforms.set_null_color(
            &gl,
            &frame_program,
            // Vec4::from_array([1.0, 0.1, 0.1, 1.]),
            Vec4::from_array([0.1, 0.1, 0.1, 1.]),
        );

        uniforms.set_roundness(&gl, &frame_program, 1., 1., 20., 20.);

        uniforms.set_edge_color(&gl, &frame_program, Vec4::rgb_u32(0xB1E1FF));

        Self {
            unit_square_vao,
            uniforms,
            shader_program_table: vec![(RendererShaderKind::Frame, frame_program)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        }
    }

    pub fn builder<'a, 'b>(&'a self, gl: &'b GlowGL, kind: RendererShaderKind) -> RenderBuilder<'a>
    where
        'b: 'a,
    {
        let prog = self
            .shader_program_table
            .get(&kind)
            .expect("shader kind not valid");
        RenderBuilder {
            gl,
            prog,
            unit_square_vao: &self.unit_square_vao,
            uniforms: &self.uniforms,
        }
    }
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