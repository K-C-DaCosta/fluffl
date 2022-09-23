use super::*;
use crate::*;

pub struct OglTexture {
    inner: Box<dyn HasTextureObj>,
}

impl std::ops::Deref for OglTexture {
    type Target = Box<dyn HasTextureObj>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for OglTexture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Into<OglTexture> for TextureObj<T>
where
    T: Copy + Default + 'static,
{
    fn into(self) -> OglTexture {
        OglTexture {
            inner: Box::new(self),
        }
    }
}

pub trait HasTextureObj {
    fn bind(&self, texture_unit: u32, sampler_location: Option<&UniformLocation>);
    fn unbind(&self);
    fn set_param_i32(&self, param: u32, val: i32);
    fn get_info(&self) -> &TextureInfo;
    fn copy_image(&self, width: u32, height: u32, src_data: &[u8]);
    fn get_info_mut(&mut self) -> &mut TextureInfo;
}

impl<T> HasTextureObj for TextureObj<T>
where
    T: Copy + Sized + Default,
{
    fn bind(&self, texture_unit: u32, location: Option<&UniformLocation>) {
        unsafe {
            let gl = &self.gl;
            gl.active_texture(glow::TEXTURE0 + texture_unit);
            gl.bind_texture(self.info.target, self.obj_id);
            gl.uniform_1_i32(location, texture_unit as i32);
        }
    }

    fn unbind(&self) {
        unsafe {
            let gl = &self.gl;
            gl.bind_texture(self.info.target, None);
        }
    }

    fn set_param_i32(&self, param: u32, val: i32) {
        unsafe {
            let gl = &self.gl;
            let target = self.info.target;
            gl.bind_texture(target, self.obj_id);
            gl.tex_parameter_i32(target, param, val);
        }
    }

    fn copy_image(&self, width: u32, height: u32, src_data: &[u8]) {
        unsafe {
            let gl = &self.gl;
            let target = self.info.target;
            let format = self.info.format;
            let comp_type = self.info.comp_type;
            let internal_format = self.info.internal_format as i32;
            // I dont assume the texture is bound
            gl.bind_texture(target, self.obj_id);
            gl.tex_image_2d(
                target,
                0,
                internal_format,
                width as i32,
                height as i32,
                0,
                format,
                comp_type,
                Some(src_data),
            );
        }
    }

    fn get_info(&self) -> &TextureInfo {
        &self.info
    }
    fn get_info_mut(&mut self) -> &mut TextureInfo {
        &mut self.info
    }
}

pub trait HasTextureBuilder<'a> {
    type Inner;
    type PixelType;
    fn with_target(self, target: u32) -> Self;
    fn with_width(self, w: u32) -> Self;
    fn with_height(self, h: u32) -> Self;
    fn with_pixels(self, pixels: Self::PixelType) -> Self;
    fn with_component_type(self, comp_type: u32) -> Self;
    fn with_internal_format(self, fmt: u32) -> Self;
    fn with_pixels_slice(self, cb: &'a [u8]) -> Self;
    fn with_format(self, fmt: u32) -> Self;
    fn build(self) -> Self::Inner;
}

pub struct TextureBuilder<'a, T> {
    pixel_slice: Option<&'a [u8]>,
    tex_obj: TextureObj<T>,
}

impl<'a, T> std::ops::Deref for TextureBuilder<'a, T> {
    type Target = TextureObj<T>;
    fn deref(&self) -> &Self::Target {
        &self.tex_obj
    }
}
impl<'a, T> std::ops::DerefMut for TextureBuilder<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tex_obj
    }
}

#[derive(Copy, Clone)]
pub struct TextureInfo {
    target: u32,
    width: u32,
    height: u32,
    internal_format: u32,
    format: u32,
    level: u32,
    comp_type: u32,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            target: glow::TEXTURE_2D,
            width: 0,
            height: 0,
            level: 0,
            format: glow::RGB,
            internal_format: glow::RGBA,
            comp_type: glow::UNSIGNED_BYTE,
        }
    }
}

pub struct TextureObj<T> {
    gl: GlowGL,
    obj_id: Option<glow::Texture>,
    info: TextureInfo,
    pixel_vec: Vec<T>,
}

impl<T> Drop for TextureObj<T> {
    fn drop(&mut self) {
        self.obj_id.map(|id| unsafe {
            self.gl.delete_texture(id);
        });
    }
}

impl<'a, T> TextureObj<T> {
    pub fn new(gl: &GlowGL) -> OglIncomplete<TextureBuilder<'a, T>> {
        let tex_obj = Self {
            gl: gl.clone(),
            obj_id: None,
            info: TextureInfo::default(),
            pixel_vec: Vec::new(),
        };
        OglIncomplete::new(TextureBuilder {
            tex_obj,
            pixel_slice: None,
        })
    }
}

impl<'a, T> HasTextureBuilder<'a> for OglIncomplete<TextureBuilder<'a, T>>
where
    T: Copy + Default + Sized,
{
    type Inner = TextureObj<T>;
    type PixelType = Vec<T>;

    fn with_height(mut self, h: u32) -> Self {
        self.inner.info.height = h;
        self
    }

    fn with_width(mut self, w: u32) -> Self {
        self.inner.info.width = w;
        self
    }

    fn with_pixels(mut self, pixels: Self::PixelType) -> Self {
        self.inner.pixel_vec = pixels;
        self
    }

    fn with_target(mut self, target: u32) -> Self {
        self.inner.info.target = target;
        self
    }

    fn with_component_type(mut self, comp_type: u32) -> Self {
        self.inner.info.comp_type = comp_type;
        self
    }

    fn with_format(mut self, fmt: u32) -> Self {
        self.inner.info.format = fmt;
        self
    }

    fn with_internal_format(mut self, fmt: u32) -> Self {
        self.inner.info.internal_format = fmt;
        self
    }

    fn with_pixels_slice(mut self, slice: &'a [u8]) -> Self {
        self.inner.pixel_slice = Some(slice);
        self
    }

    fn build(self) -> Self::Inner {
        unsafe {
            let mut tex = self.inner;
            let gl = tex.gl.clone();

            tex.obj_id = Some(gl.create_texture().unwrap());

            gl.bind_texture(tex.info.target, tex.obj_id);

            let pixels = &tex.pixel_vec;

            match tex.info.target {
                glow::TEXTURE_2D => {
                    if tex.pixel_vec.len() > 0 {
                        //if vector (Vec<T>) was set i'll use that
                        gl.tex_image_2d(
                            tex.info.target,
                            tex.info.level as i32,
                            tex.info.internal_format as i32,
                            tex.info.width as i32,
                            tex.info.height as i32,
                            0,
                            tex.info.format,
                            tex.info.comp_type,
                            Some(pixels.raw_bytes()),
                        );
                    } else if tex.pixel_slice.is_some() {
                        //if slice was provided instead i'll use that
                        gl.tex_image_2d(
                            tex.info.target,
                            tex.info.level as i32,
                            tex.info.internal_format as i32,
                            tex.info.width as i32,
                            tex.info.height as i32,
                            0,
                            tex.info.format,
                            tex.info.comp_type,
                            tex.pixel_slice,
                        );
                    } else {
                        //if vector is empty and slice was not provided i'll create an empty texture of some size
                        gl.tex_image_2d(
                            tex.info.target,
                            tex.info.level as i32,
                            tex.info.internal_format as i32,
                            tex.info.width as i32,
                            tex.info.height as i32,
                            0,
                            tex.info.format,
                            tex.info.comp_type,
                            None,
                        );
                    }

                    //default texture parameters are here (can be changed later on obv)
                    gl.tex_parameter_i32(
                        TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    gl.tex_parameter_i32(
                        TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                    gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                }
                _ => (),
            }

            tex.tex_obj
        }
    }
}
