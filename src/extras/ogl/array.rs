use super::{buffer::*, *};
use crate::{window::*, *};

/// `buffer_name` should be a unique string to name your buffer.
/// `buffer_object` is just the buffer object
pub struct BufferPair {
    buffer_name: &'static str,
    buffer_object: Box<dyn HasBufferObj>,
}

impl BufferPair {
    pub fn new(name: &'static str, object: Box<dyn HasBufferObj>) -> Self {
        Self {
            buffer_name: name,
            buffer_object: object,
        }
    }
}

pub trait ArrayBuilder {
    type InnerStruct;
    fn init(self, buffer_list: Vec<BufferPair>) -> Self::InnerStruct;
}

impl ArrayBuilder for OglIncomplete<OglArray> {
    type InnerStruct = OglArray;
    /// initalize the vertex array object bv first specifying the buffers
    /// every buffer is given a name
    fn init(mut self, buffer_list: Vec<BufferPair>) -> Self::InnerStruct {
        let gl = self.inner.gl.clone();
        //make sure vao is bound
        unsafe {
            self.inner.gl_array = Some(gl.create_vertex_array().unwrap());
            self.inner.bind(true);
        }
        for BufferPair {
            buffer_name,
            buffer_object,
        } in buffer_list
        {
            if let Ok(name) = FixedString::from(buffer_name) {
                //bind buffer object
                buffer_object.bind(true);
                let index = buffer_object.info().index;
                unsafe {
                    //define attrib pointers
                    gl.vertex_attrib_pointer_f32(
                        index,
                        buffer_object.info().num_comps as i32,
                        glow::FLOAT,
                        false,
                        0,
                        0,
                    );

                    //enable attrib pointer
                    gl.enable_vertex_attrib_array(index);
                }

                self.inner.buf_table.insert(name, buffer_object);
            }
        }
        self.inner
    }
}

pub struct OglArray {
    gl: GlowGL,
    gl_array: Option<glow::VertexArray>,
    buf_table: HashMap<FixedString, Box<dyn HasBufferObj>>,
}

impl OglArray {
    pub fn new(gl: &GlowGL) -> OglIncomplete<Self> {
        OglIncomplete::new(Self {
            gl: gl.clone(),
            gl_array: None,
            buf_table: HashMap::new(),
        })
    }
    pub fn get(&self, buffer_name: &'static str) -> Option<&Box<dyn HasBufferObj>> {
        self.buf_table.get(buffer_name)
    }
    pub fn get_mut(&mut self, buffer_name: &'static str) -> Option<&mut Box<dyn HasBufferObj>> {
        self.buf_table.get_mut(buffer_name)
    }
}

impl Bindable for OglArray {
    fn bind(&self, ok: bool) {
        unsafe {
            self.gl
                .bind_vertex_array(ok.map(|| self.gl_array).flatten());
        }
    }
}

impl Drop for OglArray {
    fn drop(&mut self) {
        let gl = self.gl.clone();
        unsafe {
            gl.delete_vertex_array(self.gl_array.unwrap());
        }
    }
}
