use std::ops::{Index, IndexMut};

use super::*;
use crate::{math::Vector, *};

#[derive(Copy, Clone)]
pub struct BufferInfo {
    pub target: u32,
    pub usage: u32,
    pub index: u32,
    pub num_comps: u32,
}

impl Default for BufferInfo {
    fn default() -> Self {
        Self {
            target: glow::ARRAY_BUFFER,
            usage: glow::DYNAMIC_DRAW,
            index: 0,
            num_comps: 1,
        }
    }
}

pub struct OglBuf<T> {
    data: T,
    gl_buf: Option<glow::Buffer>,
    gl: GlowGL,
    info: BufferInfo,
}

pub trait HasBufferObj: HasData + Bindable {
    fn info(&self) -> &BufferInfo;
    fn update(&self);
}

pub trait HasBufferBuilder {
    type InnerStruct;
    type VecItemType;
    fn with_target(self, target: u32) -> OglIncomplete<Self::InnerStruct>;
    fn with_usage(self, usage: u32) -> OglIncomplete<Self::InnerStruct>;
    fn with_index(self, index: u32) -> OglIncomplete<Self::InnerStruct>;
    fn with_num_comps(self, comps: u32) -> OglIncomplete<Self::InnerStruct>;
    fn with_data(self, data: Vec<Self::VecItemType>) -> OglIncomplete<Self::InnerStruct>;
    fn build(self) -> Self::InnerStruct;
}

impl<T> HasBufferBuilder for OglIncomplete<OglBuf<Vec<T>>>
where
    T: Copy + Default,
{
    type InnerStruct = OglBuf<Vec<T>>;
    type VecItemType = T;

    fn with_target(mut self, target: u32) -> Self {
        self.inner.info.target = target;
        self
    }

    fn with_usage(mut self, usage: u32) -> Self {
        self.inner.info.usage = usage;
        self
    }

    fn with_index(mut self, index: u32) -> Self {
        self.inner.info.index = index;
        self
    }
    fn with_num_comps(mut self, comps: u32) -> Self {
        self.inner.info.num_comps = comps;
        self
    }

    fn with_data(mut self, data: Vec<Self::VecItemType>) -> Self {
        self.inner.data = data;
        self
    }

    fn build(self) -> Self::InnerStruct {
        let mut new_self = self.inner;

        let gl = new_self.gl.clone();
        let target = new_self.info.target;
        unsafe {
            new_self.gl_buf = Some(gl.create_buffer().unwrap());
            gl.bind_buffer(target, new_self.gl_buf);
            gl.buffer_data_u8_slice(target, new_self.data.raw_bytes(), new_self.info.usage);
        }
        new_self
    }
}

impl<T> OglBuf<Vec<T>>
where
    T: Default + Copy + Sized,
    Vec<T>: HasData,
{
    pub fn new(gl: &GlowGL) -> OglIncomplete<Self> {
        OglIncomplete::new(Self {
            data: Vec::new(),
            gl_buf: None,
            gl: gl.clone(),
            info: BufferInfo::default(),
        })
    }
}

impl<T> Bindable for OglBuf<T> {
    fn bind(&self, ok: bool) {
        let gl = self.gl.clone();
        unsafe {
            gl.bind_buffer(self.info.target, ok.map(|| self.gl_buf).flatten());
        }
    }
}

impl<T> HasBufferObj for OglBuf<Vec<T>>
where
    Vec<T>: HasData,
{
    fn info(&self) -> &BufferInfo {
        &self.info
    }

    fn update(&self) {
        let gl = &self.gl;
        self.bind(true);
        unsafe {
            gl.buffer_sub_data_u8_slice(self.info.target, 0, self.raw_bytes());
        }
    }
}

impl<T> HasData for OglBuf<Vec<T>>
where
    Vec<T>: HasData,
{
    fn raw_bytes(&self) -> &[u8] {
        self.data.raw_bytes()
    }
    fn raw_bytes_mut(&mut self) -> &mut [u8] {
        self.data.raw_bytes_mut()
    }
}

impl<T> Drop for OglBuf<T> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.gl_buf.unwrap());
        }
    }
}

impl<T> Into<Box<dyn HasBufferObj>> for OglBuf<Vec<T>>
where
    T: 'static,
    Vec<T>: HasData,
{
    fn into(self) -> Box<dyn HasBufferObj> {
        Box::new(self)
    }
}


