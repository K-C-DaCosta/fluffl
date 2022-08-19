use super::{
    math_util::*,
    ogl::{array::*, buffer::*, program::*, texture::*, OglIncomplete, *},
};

pub use hiero_pack::{self, *};
use crate::{console::*, *};
use glow::*;

// This module renders Hiero Atlases(signed distance fields only) in OpenGL.
// Right now only 2D orthographic drawing is implemented

/// number of pixels used to represent whitespace (might change this later)
static DEFAULT_WHITE_SPACE_LEN: f32 = 24.0;

///The number of characters that can be drawn in a single draw-call
static CHARACTER_BUFFER_LEN: usize = 256;

///The default shader hardcoded for portability purposes
static DEFAULT_PROGRAM: &'static str = r"
    #ifndef HEADER
        #version 300 es
        precision mediump float;
    #endif

    #ifndef UNIFORMS
        uniform sampler2D page;
        uniform vec4 text_color;
        uniform mat4 projection; 
        uniform mat4 model; 
    #endif

    #ifndef VERTEX_ATTRIBUTES
        layout(location = 1) in vec2 vert_in; 
        layout(location = 2) in vec2 uv_in; 
    #endif

    #ifndef VERTEX_SHADER
        out vec2 tex_coord; 
        void main(){
            tex_coord = uv_in; 
            gl_Position = projection*model*vec4(vert_in.xy,0.0,1.0); 
        }
    #endif
    
    #ifndef FRAGMENT_SHADER
        in vec2 tex_coord; 
        out vec4 color; 
        void main(){
            vec4 page = texture(page,tex_coord);
            float dist = page.w; 
            vec2 grad = vec2( dFdx(dist), dFdy(dist));  
            float grad_mag = length(grad)*1.1;
            color = vec4(1.)*smoothstep(0.5-grad_mag,0.5+grad_mag,dist);
        }
    #endif
";

struct WriterState {
    pen_x: f32,
    pen_y: f32,
    comps_filled: usize,
    chars_filled: usize,
    vert_len: usize,
    text_len: usize,
}

impl WriterState {
    fn advance_pen(&mut self, dx: f32, dy: f32) {
        self.pen_x += dx;
        self.pen_y += dy;
    }

    fn push_glyph(
        &mut self,
        verts: &mut [f32],
        uvs: &mut [f32],
        xoff: f32,
        yoff: f32,
        glyph_bounds: AABB,
    ) {
        //generate glyph quad and its uvs
        TextWriter::set_glyph(
            verts,
            uvs,
            AABB::new(
                self.pen_x + xoff,
                self.pen_y + yoff,
                glyph_bounds.w,
                glyph_bounds.h,
            ),
            glyph_bounds,
            self.comps_filled,
        );
        self.count_glyph();
    }

    fn count_glyph(&mut self) {
        // count writen glyphs
        self.chars_filled += 1;
        // count components written.
        // I increment by  12 because:
        // 6 verts per quad 2 components per vert means 6*2 = 12 components
        self.comps_filled += 12;
    }

    fn buffers_are_full(&self, char_index: usize) -> bool {
        self.comps_filled >= self.vert_len || char_index >= self.text_len - 1
    }

    fn flush(
        &mut self,
        gl: &GlowGL,
        vert_buffer: &mut Box<dyn HasBufferObj>,
        uv_buffer: &mut Box<dyn HasBufferObj>,
    ) {
        // submit changes to opengl
        uv_buffer.update();
        vert_buffer.update();

        // submit draw call here
        unsafe {
            gl.draw_arrays(TRIANGLES, 0, 6 * self.chars_filled as i32);
        }

        //clear counters
        self.comps_filled = 0;
        self.chars_filled = 0;
    }
}
/// # Description
/// A simple text writer that draws signed-distance-fields fonts, using an atlas generated by `hiero`
/// and packed/parsed by my own tool `hiero_pack`
/// # Comments
/// - Needs opengl 3.0 or webgl2 for shader to  rcompile.
/// - Kernings are not implemented, font will render badly for certain inputs.
pub struct TextWriter {
    gl: GlowGL,
    text_geometry: OglArray,
    renderer: OglProg,
    atlas: Option<HieroAtlas>,
    projection_mat_loc: Option<UniformLocation>,
    model_loc: Option<UniformLocation>,
    page_loc: Option<UniformLocation>,
    page_texture: Option<OglTexture>,
    whitespace_len: Option<f32>,
    page_history: [usize; 4],
    page_index: usize,
}

impl TextWriter {
    pub fn new(gl: &GlowGL) -> OglIncomplete<Self> {
        let renderer = match OglProg::compile_program(gl, DEFAULT_PROGRAM) {
            Ok(prog) => prog,
            Err(comp_err) => {
                match comp_err {
                    CompilationError::ShaderError {
                        ogl_error,
                        faulty_source,
                    } => {
                        console_log!("ogl_error:\n{}\nsource:{}\n", ogl_error, faulty_source);
                    }
                    _ => (),
                };

                panic!("shader compiler error");
            }
        };

        let vert_data: Vec<f32> = vec![0.; CHARACTER_BUFFER_LEN * 6 * 2];
        let uv_data = vert_data.clone();

        let array = OglArray::new(gl).init(vec![
            BufferPair::new(
                "verts",
                OglBuf::new(gl)
                    .with_usage(DYNAMIC_DRAW)
                    .with_data(vert_data)
                    .with_index(1)
                    .with_num_comps(2)
                    .build()
                    .into(),
            ),
            BufferPair::new(
                "uvs",
                OglBuf::new(&gl)
                    .with_usage(DYNAMIC_DRAW)
                    .with_data(uv_data)
                    .with_index(2)
                    .with_num_comps(2)
                    .build()
                    .into(),
            ),
        ]);

        unsafe {
            renderer.bind(true);
            let proj_loc = gl.get_uniform_location(renderer.prog(), "projection");
            let page_loc = gl.get_uniform_location(renderer.prog(), "page");
            let model_loc = gl.get_uniform_location(renderer.prog(), "model");

            OglIncomplete::new(Self {
                gl: gl.clone(),
                renderer: renderer,
                atlas: None,
                model_loc,
                projection_mat_loc: proj_loc,
                text_geometry: array,
                page_loc,
                page_texture: None,
                whitespace_len: None,
                page_history: [99999; 4],
                page_index: 0,
            })
        }
    }
    /// # Description
    /// Calculates a tight bounding box of the text, but doesn't actually draw anything
    /// # Parameters
    /// - `text` - the text you wish compute bonding box of
    /// - `x0`,`y0` - the top left corner of the bounding box
    /// - `size` - the vertical height of the text
    /// # Comments
    /// In order to avoid 'squished' looking text, I try to maintain aspect ratio of unscaled glyphs
    pub fn calc_text_aabb(&self, text: &str, x0: f32, y0: f32, size: f32) -> AABB {
        let src_bb = self.calculate_bounding_box(x0, y0, text);
        let aspect_ratio = src_bb.w / src_bb.h;
        let width = aspect_ratio * size;
        AABB {
            x: src_bb.x,
            y: src_bb.y,
            w: width,
            h: size,
        }
    }

    /// # Description
    /// Draws a line of `text`
    /// # Parameters
    /// - `x0` and `y0` are the position of text in the top-left corner\
    /// - `size` - specifys the height of the text ( aspect ratio is preserved ) width depends on the length(number of chars) of text\
    /// - `screen_bounds` - the routine *needs* the dimensions of the screen in order to draw correctly\
    /// # Notes
    /// ---
    /// - Characters not present in the atlas are considered whitespace
    /// - Best performance is when an atlas consists of a SINGLE page(only one decode for entire lifetime of writer).
    /// - The routine only decodes one page at a time, so rendering can be very, very slow for certain strings.
    /// ## For exmaple(of worst case scenario):
    /// suppose character 'a' is in page 0 and character 'b' is in page 1, then the string 'ababab' will
    /// decode page 0 and 1 SIX times collectively. Decodes are really,relly, really slow.  
    pub fn draw_text_line(
        &mut self,
        text: &str,
        x0: f32,
        y0: f32,
        size: f32,
        screen_bounds: Option<(u32, u32)>,
    ) {
        //just covering base cases
        if text.len() == 0 {
            return;
        }

        let gl = self.gl.clone();
        let (screen_w, screen_h) = screen_bounds.unwrap_or((800, 600));
        let proj_mat = calc_proj(screen_w as f32, screen_h as f32);
        let src_bb = self.calculate_bounding_box(x0, y0, text);
        let aspect_ratio = src_bb.w / src_bb.h;
        let resize_matrix = resize_region(src_bb, AABB::new(x0, y0, aspect_ratio * size, size));
        let whitespace = self.whitespace();

        //bind program
        self.renderer.bind(true);

        //bind text vao
        self.text_geometry.bind(true);

        //if it exists, bind texture to texture unit 0
        self.page_texture
            .as_ref()
            .map(|texture| texture.bind(0, self.page_loc.as_ref()));

        //force split borrow here
        let uv_buffer: &mut Box<dyn HasBufferObj> = unsafe {
            let ptr = self.text_geometry.get_mut("uvs").unwrap() as *mut Box<dyn HasBufferObj>;
            &mut *ptr
        };

        //force split borrow here too
        let vert_buffer: &mut Box<dyn HasBufferObj> = unsafe {
            let ptr = self.text_geometry.get_mut("verts").unwrap() as *mut Box<dyn HasBufferObj>;
            &mut *ptr
        };

        //intitalize render state
        let mut writer_state = WriterState {
            pen_x: x0,
            pen_y: y0,
            comps_filled: 0,
            chars_filled: 0,
            vert_len: Self::to_float_slice(vert_buffer.raw_bytes_mut()).len(),
            text_len: text.len(),
        };

        let first_char = text.chars().next().unwrap();

        let first_page = self
            .atlas
            .as_ref()
            .map(|atlas| {
                atlas
                    .bitmap_table
                    .get(&first_char)
                    .map(|bitmap| bitmap.page)
            })
            .flatten();

        first_page.map(|index| {
            if self.cur_page() != index as usize {
                self.decode_page(index as usize);
                self.new_page(index as usize);
            }
        });

        unsafe {
            //enable blending here
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            //make uniforms are up-to-date
            gl.uniform_matrix_4_f32_slice(self.projection_mat_loc.as_ref(), false, &proj_mat[..]);
            gl.uniform_matrix_4_f32_slice(self.model_loc.as_ref(), true, &resize_matrix[..]);
        }

        for (k, character) in text.char_indices() {
            let bitmap = self
                .atlas
                .as_ref()
                .map(|atlas| atlas.bitmap_table.get(&character))
                .flatten()
                .map(|&a| a);

            let adv = bitmap.map_or((whitespace, 0.), |bitmap| {
                let new_page = bitmap.page as usize;
                let cur_page = self.cur_page();

                if new_page != cur_page {
                    //flush buffer old buffer
                    writer_state.flush(&gl, vert_buffer, uv_buffer);
                    //load new_page
                    self.decode_page(new_page);
                }

                let hiero_bounds = AABB::new(
                    bitmap.x as f32,
                    bitmap.y as f32,
                    bitmap.width as f32,
                    bitmap.height as f32,
                );

                //type-pun both buffers to float-32
                let uvs = Self::to_float_slice(uv_buffer.raw_bytes_mut());
                let verts = Self::to_float_slice(vert_buffer.raw_bytes_mut());

                writer_state.push_glyph(
                    verts,
                    uvs,
                    bitmap.xoffset as f32,
                    bitmap.yoffset as f32,
                    hiero_bounds,
                );

                self.new_page(new_page);

                (bitmap.xadvance as f32, 0.0)
            });

            writer_state.advance_pen(adv.0, adv.1);

            if writer_state.buffers_are_full(k) {
                //opengl draw call happens here
                writer_state.flush(&gl, vert_buffer, uv_buffer);
            }
        }

        unsafe {
            //disable blending before exiting
            gl.disable(glow::BLEND);
        }
    }
    fn decode_page(&mut self, new_page: usize) {
        //decode new page and update OglTexture
        self.atlas.as_ref().map(|atlas| {
            atlas
                .try_unpack_page(new_page)
                .map(|page| {
                    let pixel_slice = &page.pixels()[..];
                    self.page_texture.as_ref().map(|page_texture| {
                        page_texture.copy_image(512, 512, pixel_slice);
                    })
                })
                .map_err(|err| {
                    panic!("Error: {}", err);
                })
                .unwrap();
        });
    }
    /// # Description
    /// Computes bounding box of unscaled `text` at pen position: `(x0,y0)`
    /// # Comments
    /// - All coordinates are in standard screen-space
    fn calculate_bounding_box(&self, x0: f32, y0: f32, text: &str) -> AABB {
        let mut minx = std::f32::INFINITY;
        let mut miny = std::f32::INFINITY;
        let mut maxx = std::f32::NEG_INFINITY;
        let mut maxy = std::f32::NEG_INFINITY;
        let mut pen_x = x0;
        let pen_y = y0;
        let whitespace = self.whitespace();
        self.atlas.as_ref().map(|atlas| {
            text.chars().for_each(|c| {
                let x_adv = atlas.bitmap_table.get(&c).map_or_else(
                    || whitespace,
                    |bitmap| {
                        let xoff = bitmap.xoffset as f32;
                        let yoff = bitmap.yoffset as f32;
                        let x = xoff + pen_x;
                        let y = yoff + pen_y;
                        let w = bitmap.width as f32;
                        let h = bitmap.height as f32;

                        [(x, y), (x + w, y), (x, y + h), (x + w, y + h)]
                            .iter()
                            .for_each(|&(x, y)| {
                                minx = minx.min(x);
                                maxx = maxx.max(x);
                                miny = miny.min(y);
                                maxy = maxy.max(y);
                            });

                        bitmap.xadvance as f32
                    },
                );

                pen_x += x_adv;
            });
        });

        AABB {
            x: minx,
            y: miny,
            w: maxx - minx,
            h: maxy - miny,
        }
    }

    ///computes quads verticies and uv coordinates and writes then to the appropriate slices
    fn set_glyph(vert: &mut [f32], uvs: &mut [f32], vb: AABB, hb: AABB, offset: usize) {
        //it seems hiero pages are always 512x512
        const INV_PAGE_DIM: f32 = 1.0 / 512.;

        let get_pos = |rel_index: usize| -> usize { (2 * rel_index) + offset };

        let mut uv_helper = |rel_index: usize, x, y| {
            uvs[get_pos(rel_index) + 0] = x * INV_PAGE_DIM;
            uvs[get_pos(rel_index) + 1] = y * INV_PAGE_DIM;
        };

        let mut vert_helper = |rel_index: usize, x, y| {
            vert[get_pos(rel_index) + 0] = x;
            vert[get_pos(rel_index) + 1] = y;
        };

        //verts and uvs are generated on stack here
        let uvs = [
            (hb.x, hb.y),
            (hb.x + hb.w, hb.y),
            (hb.x + hb.w, hb.y + hb.h),
            (hb.x, hb.y),
            (hb.x + hb.w, hb.y + hb.h),
            (hb.x, hb.y + hb.h),
        ];

        let verts = [
            (vb.x, vb.y),
            (vb.x + vb.w, vb.y),
            (vb.x + vb.w, vb.y + vb.h),
            (vb.x, vb.y),
            (vb.x + vb.w, vb.y + vb.h),
            (vb.x, vb.y + vb.h),
        ];

        //write attribute data from stack to heap
        for (k, (&(vx, vy), (uvx, uvy))) in verts.iter().zip(uvs.iter()).enumerate() {
            uv_helper(k, uvx, uvy);
            vert_helper(k, vx, vy);
        }
    }

    /// This operation is pretty much just a cast
    fn to_float_slice(slice: &mut [u8]) -> &mut [f32] {
        let bytes = slice.len();
        unsafe {
            std::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut f32,
                bytes / std::mem::size_of::<f32>(),
            )
        }
    }

    /// # Description
    /// Returns the number of pixels the writer should skip when encounting a whitespace character
    /// # Comments
    /// - the text writer maintains an internal`whitespace_len` that the user should be able to change if
    /// the current whitespace settings are not desireable
    /// - there should be a public "`set_whitespace(..)` or `with_whitespace(..)`" function, but currently one doesn't exist
    fn whitespace(&self) -> f32 {
        self.whitespace_len
            .unwrap_or_else(|| DEFAULT_WHITE_SPACE_LEN)
    }

    #[allow(dead_code)]
    /// # Description
    /// Moves page index to previous page
    fn prev_page(&self) -> usize {
        let len = self.page_history.len();
        self.page_history[(self.page_index + len - 1) % len]
    }
    /// # Description
    /// fetches currently loaded page number
    fn cur_page(&self) -> usize {
        self.page_history[self.page_index]
    }

    /// # Description
    /// records accessed page
    fn new_page(&mut self, page: usize) {
        let new_index = (self.page_index + 1) % self.page_history.len();
        self.page_history[new_index] = page;
        self.page_index = new_index;
    }
}
pub trait HasTextWriterBuilder {
    type WriterType;
    fn with_atlas(self, atlus: HieroAtlas) -> Self;
    fn build(self) -> Self::WriterType;
}

impl HasTextWriterBuilder for OglIncomplete<TextWriter> {
    type WriterType = TextWriter;
    fn with_atlas(mut self, atlus: HieroAtlas) -> Self {
        let gl = &self.inner.gl;

        self.inner.page_texture = atlus.try_unpack_page(0).ok().map(|page| {
            let info = page.info();
            TextureObj::<u8>::new(gl)
                .with_width(info.width)
                .with_height(info.height)
                .with_format(glow::RGBA)
                .build()
                .into()
        });

        self.inner.atlas = Some(atlus);
        self
    }
    fn build(self) -> Self::WriterType {
        self.inner
    }
}
