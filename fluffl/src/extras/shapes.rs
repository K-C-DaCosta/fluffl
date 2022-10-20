use super::math_util::*;
use crate::{GlowGL,ogl::{array::*, buffer::*, program::*, *}};
use glow::*;

static BOX_PROGRAM_SOURCE: &str = "
    #ifndef HEADER
        #version 300 es
        precision mediump float;
    #endif

    #ifndef UNIFORMS
        //common uniforms
        uniform vec4 shape_color;
        uniform mat4 projection;
        uniform float glow_str;
        uniform float roundness;
        uniform float shape_morph; 

        //circle uniforms 
        uniform float circle_radius;
        uniform vec2 circle_center; 
        
        //rectangle uniforms 
        uniform mat3 world_to_local;
        uniform vec2 rectangle_dims;
    #endif

    #ifndef VERTEX_ATTRIBUTES
        layout (location = 1) in vec2 verts_in; 
    #endif
    
    #ifndef VERTEX_SHADER
        out vec2 aabb_pos; 
        void main(){
            aabb_pos = verts_in; 
            gl_Position = projection*vec4(verts_in,0.,1.); 
        }
    #endif
    
    #ifndef FRAGMENT_SHADER
        in vec2 aabb_pos;
        out vec4 color;  
        

         float sdCircle( in vec2 p , in vec2 center, in float radius )
         {
            return length(p-center)-radius;
         }

        //source: https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
        float sdBox( in vec2 p, in vec2 b )
        {
            vec2 d = abs(p)-b;
            return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
        }

        void main(){
            vec2 pos = (world_to_local*vec3(aabb_pos,1.)).xy;
            
            float dCircle = sdCircle(aabb_pos,circle_center,circle_radius) - roundness;
            float dBox = sdBox(pos,rectangle_dims) - roundness;
            
            
            float d = mix(dBox,dCircle,shape_morph);
            
            // for anti-aliasing compute screenspace derivates (another trick i picked up from shadertoy)
            vec2 grad =  vec2( dFdx(d), dFdy(d) )  ;
            float grad_len = length(grad);

            color = vec4(0);
            
            color += shape_color* smoothstep(grad_len,-grad_len,d);
            color += shape_color*(2.0/(  ( sqrt(abs(d))  )+0.5) )*glow_str;
            
        }
    #endif
";

/// # Description
/// An opengl utility the lets you draw smooth anti-aliased shapes with interesting effects
/// # Comments
/// - You're only supposed to create ONE of these, then just call the draw functions
/// - Shaders are hard-coded into the source
/// - The painter doesn't know the screen bounds unless you give it that that info! call update_bounds(..) when window
///  changes dimensions
/// - This draws shapes by using implicit equations, so it doesn't draw many triangles just one quad that may or may not be fullscreen
/// - shaders this uses will need opengl 3.0 / webgl 2 in order to work

pub struct ShapePainter2D {
    box_program: OglProg,
    bounding_box: OglArray,
    projection_loc: Option<glow::UniformLocation>,
    color_loc: Option<glow::UniformLocation>,
    circle_center_loc: Option<glow::UniformLocation>,
    circle_radius_loc: Option<glow::UniformLocation>,
    world_to_local_loc: Option<glow::UniformLocation>,
    rectangle_dims_loc: Option<glow::UniformLocation>,
    rectangle_roundness_loc: Option<glow::UniformLocation>,
    rectangle_glow_str_loc: Option<glow::UniformLocation>,
    shape_morph_loc: Option<glow::UniformLocation>,
    gl: GlowGL,
    window_width: f32,
    window_height: f32,
}
impl ShapePainter2D {
    pub fn new(gl: &GlowGL) -> Self {
        let box_program = match OglProg::compile_program(gl, BOX_PROGRAM_SOURCE) {
            Ok(a) => a,
            Err(CompilationError::ShaderError {
                ogl_error,
                faulty_source,
            }) => panic!("{},{}", ogl_error, faulty_source),
            Err(_) => panic!("link error"),
        };

        let projection_loc = unsafe { gl.get_uniform_location(box_program.prog(), "projection") };
        let color_loc = unsafe { gl.get_uniform_location(box_program.prog(), "shape_color") };
        let world_to_local_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "world_to_local") };
        let rectangle_dims_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "rectangle_dims") };
        let rectangle_roundness_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "roundness") };

        let rectangle_glow_str_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "glow_str") };

        let circle_center_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "circle_center") };
        let circle_radius_loc =
            unsafe { gl.get_uniform_location(box_program.prog(), "circle_radius") };

        let shape_morph_loc = unsafe { gl.get_uniform_location(box_program.prog(), "shape_morph") };

        let vao = OglArray::new(gl).init(vec![BufferPair::new(
            "quad_verts",
            OglBuf::new(gl)
                .with_num_comps(2)
                .with_target(glow::ARRAY_BUFFER)
                .with_usage(glow::DYNAMIC_DRAW)
                .with_index(1)
                .with_data(vec![0.0; 12])
                .build()
                .into(),
        )]);

        Self {
            box_program,
            bounding_box: vao,
            projection_loc,
            color_loc,
            world_to_local_loc,
            rectangle_dims_loc,
            rectangle_roundness_loc,
            rectangle_glow_str_loc,
            circle_center_loc,
            circle_radius_loc,
            shape_morph_loc,
            gl: gl.clone(),
            window_width: 800.0,
            window_height: 600.0,
        }
    }
    /// # Description
    /// internal
    pub fn update_bounds(&mut self, bounds: (u32, u32)) {
        self.window_width = bounds.0 as f32;
        self.window_height = bounds.1 as f32;
    }

    /// # Description
    /// Draws an anti-aliased rectangle
    /// # Parameters  
    /// - `a` - 2D coordinate of the start of a line segment
    /// - `b` - 2D coordinate of the end of a line segment
    /// - `color` - 4d rgba color coordinate
    /// - `half-height` - the thickness of the rectangle
    /// - `roundness`- the roundness of the rectangle valid from: 0.0 <= roundness <= +inf
    /// - `glow_strength` - determines the strength of the glow
    /// - 'circle_morph' - determines the transition from rectangle(morph=0) to circle(morph=1)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rectangle(
        &mut self,
        a: &[f32],
        b: &[f32],
        color: &[f32],
        half_height: f32,
        roundness: f32,
        glow_strength: f32,
        morph: f32,
    ) {
        let segment_points = [[a[0], a[1]], [b[0], b[1]]];

        let (world_to_local, points, half_width) =
            compute_world_to_local_from_segment(segment_points[0], segment_points[1], half_height);

        let aabb = compute_bounding_box_from_points_2d(&points[..]);

        self.box_program.bind(true);
        self.bounding_box.bind(true);

        let mut top_left = aabb.get_top_left();
        let mut aabb_dims = aabb.get_dims();

        //basically im just expanding the bounding box so that the rectangle doesnt get clipped
        top_left[0] += -8.0 * (roundness + 1.5);
        top_left[1] += -8.0 * (roundness + 1.5);
        aabb_dims[0] += 16.0 * (roundness + 1.5);
        aabb_dims[1] += 16.0 * (roundness + 1.5);

        //update uniforms
        unsafe {
            let proj_mat = calc_proj(self.window_width, self.window_height);

            self.gl
                .uniform_matrix_4_f32_slice(self.projection_loc.as_ref(), false, &proj_mat[..]);

            self.gl
                .uniform_2_f32(self.rectangle_dims_loc.as_ref(), half_width, half_height);
            self.gl
                .uniform_1_f32(self.rectangle_roundness_loc.as_ref(), roundness);

            self.gl.uniform_4_f32_slice(self.color_loc.as_ref(), color);

            self.gl
                .uniform_1_f32(self.rectangle_glow_str_loc.as_ref(), glow_strength.max(0.));

            //update world_to_local mat
            self.gl.uniform_matrix_3_f32_slice(
                self.world_to_local_loc.as_ref(),
                false,
                &world_to_local[..],
            );

            self.gl.uniform_2_f32(
                self.circle_center_loc.as_ref(),
                (a[0] + b[0]) * 0.5,
                (a[1] + b[1]) * 0.5,
            );

            self.gl
                .uniform_1_f32(self.circle_radius_loc.as_ref(), half_height * 2.0);

            self.gl.uniform_1_f32(self.shape_morph_loc.as_ref(), morph);
        }

        let window_dims = [self.window_width, self.window_height];

        //compute bounding box by modifing content of vertex buffer
        if let Some(buffer_ref) = self.bounding_box.get_mut("quad_verts") {
            let vect_list = cast_slice_to_vec2(buffer_ref.raw_bytes_mut());

            if glow_strength.max(0.) > 0.1 {
                // if glowing is needed, I generate a fullscreen quad, because I couldn't figure out
                // how to use a tighter aabb without leaving behind noticable cut-off of the gradient
                set_bounding_box(vect_list, [0., 0.], window_dims);
            } else {
                // the rectangle doesn't glow so a tighter qui
                set_bounding_box(vect_list, top_left, aabb_dims);
            }

            // submit changes to vertex buffer
            buffer_ref.update();
        }

        //draw bounding box
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }

        self.box_program.bind(false);
    }

    /// # Description
    /// Draws an anti-aliased circle
    /// # Parameters  
    /// - `center` - 2D coordinate
    /// - `radius` - radius of circle
    /// - `color` - 4d rgba color coordinate
    /// - `half-height` - the thickness of the rectangle
    /// - `roundness`- the roundness of the rectangle valid from: 0.0 <= roundness <= +inf
    /// - `glow_strength` - determines the strength of the glow
    /// - 'circle_morph' - determines the transition from rectangle(morph=0) to circle(morph=1)
    pub fn draw_circle(
        &mut self,
        center: &[f32],
        radius: f32,
        color: &[f32],
        roundness: f32,
        glow_strength: f32,
    ) {
        self.box_program.bind(true);
        self.bounding_box.bind(true);

        let top_left = [center[0] - radius, center[1] - radius];
        let aabb_dims = [2. * radius, 2. * radius];

        //update uniforms
        unsafe {
            let proj_mat = calc_proj(self.window_width, self.window_height);

            self.gl
                .uniform_matrix_4_f32_slice(self.projection_loc.as_ref(), false, &proj_mat[..]);

            self.gl
                .uniform_1_f32(self.rectangle_roundness_loc.as_ref(), roundness);

            self.gl.uniform_4_f32_slice(self.color_loc.as_ref(), color);

            self.gl
                .uniform_1_f32(self.rectangle_glow_str_loc.as_ref(), glow_strength.max(0.));

            self.gl
                .uniform_1_f32(self.circle_radius_loc.as_ref(), radius);

            self.gl
                .uniform_2_f32(self.circle_center_loc.as_ref(), center[0], center[1]);

            self.gl.uniform_1_f32(self.shape_morph_loc.as_ref(), 1.0);
        }

        let window_dims = [self.window_width, self.window_height];

        //compute bounding box by modifing content of vertex buffer
        if let Some(buffer_ref) = self.bounding_box.get_mut("quad_verts") {
            let vect_list = cast_slice_to_vec2(buffer_ref.raw_bytes_mut());

            if glow_strength.max(0.) > 0.1 {
                // if glowing is needed, I generate a fullscreen quad, because I couldn't figure out
                // how to use a tighter aabb without leaving behind noticable cut-off of the gradient
                set_bounding_box(vect_list, [0., 0.], window_dims);
            } else {
                // the rectangle doesn't glow so a tighter qui
                set_bounding_box(vect_list, top_left, aabb_dims);
            }

            // submit changes to vertex buffer
            buffer_ref.update();
        }

        //draw bounding box
        unsafe {
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }

        self.box_program.bind(false);
    }
}

/// # Description
/// Generates a quad that gets written to `box_points`
fn set_bounding_box(box_points: &mut [Vec2], top_left: Vec2, bounds: Vec2) {
    let (tl, tr, br, bl) = (0, 1, 2, 3);
    let points = [
        [top_left[0], top_left[1]],                         //tl
        [top_left[0] + bounds[0], top_left[1]],             //tr
        [top_left[0] + bounds[0], top_left[1] + bounds[1]], //br
        [top_left[0], top_left[1] + bounds[1]],             //bl
    ];

    //first triangle of quad
    box_points[0] = points[tl];
    box_points[1] = points[bl];
    box_points[2] = points[br];

    //second triangle of quiad
    box_points[3] = points[br];
    box_points[4] = points[tr];
    box_points[5] = points[tl];
}

fn cast_slice_to_vec2(slice: &mut [u8]) -> &mut [Vec2] {
    const VEC2_SIZE_IN_BYES: usize = std::mem::size_of::<Vec2>();
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut [f32; 2],
            slice.len() / VEC2_SIZE_IN_BYES,
        )
    }
}
