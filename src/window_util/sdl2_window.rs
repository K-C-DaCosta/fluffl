use super::event_util::GlueEvent;
use super::*;
use crate::parsers::xml::*;
use event_util::constants::*;
use glow::*;
use std::cell::*;
use std::rc::*;
use std::sync::Arc;

pub struct GlueWindow {
    glue_event: Option<GlueEvent>,
    sdl_context: sdl2::video::GLContext,
    sdl_event_pump: sdl2::EventPump,
    pub gl: Arc<Box<Context>>,
    render_loop: Option<RenderLoop<sdl2::video::Window>>,
}
impl GlueWindow {
    //moves renderloop out of glue_window and calls it
    fn get_render_loop(&mut self) -> impl glow::HasRenderLoop {
        self.render_loop.take().unwrap()
    }
    /// The main loop is pertty basic. Just does user defined task over and over while also \
    /// doing basic matenence for events and the window \
    /// # Arguments
    /// * `closure` is the user defined task. it exposes platform specific internals with `&mut Self`. \
    /// The user should return `true` if they want to continue otherwise `false` if they want to exit the application
    /// # Examples
    /// ```
    /// let mut window = GlueWindow::new();
    /// window.init().map(|error| panic!("something wrong happened"));
    /// window.main_loop(|internals|{
    ///    let events = internals.get_events();
    ///    true  
    /// });
    /// ```
    pub fn main_loop<F>(mut self, mut closure: F)
    where
        F: FnMut(&mut Self, &mut bool) + 'static,
    {
        let render_loop = self.get_render_loop();

        render_loop.run(move |running| {
            self.collect_events();
            closure(&mut self, running);
        });
    }
}
impl WindowManager for GlueWindow {
    //parse sdl's event pump here
    fn collect_events(&mut self) {
        use sdl2::event::Event;
        let mut glue_event = self.glue_event.take();
        self.sdl_event_pump
            .poll_iter()
            .for_each(|event| match event {
                Event::Quit { .. } => {
                    glue_event.as_mut().unwrap().push_event(EventKinds::Quit);
                }
                Event::KeyUp { keycode, .. } => (),
                _ => (),
            });

        //make sure to give the pollevent back
        self.glue_event = glue_event;
    }

    //platform specific clear window stuff here
    fn clear_window(&mut self) {}

    fn get_events(&mut self) -> &mut GlueEvent {
        self.glue_event.as_mut().unwrap()
    }

    fn init(config: &str) -> Result<Self, GlueError> {
        let (width, height, title, context_major, context_minor) = extract_optional_paramaters(config);
        
        // Create a context from a sdl2 window
        let sdl = sdl2::init()?;
        let video = sdl.video()?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(context_major, context_minor);

        let window = video
            .window(title.as_str(), width, height)
            .opengl()
            .resizable()
            .build()?;

        let gl_context = window.gl_create_context()?;
        let context =
            glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);

        let render_loop = Some(glow::RenderLoop::<sdl2::video::Window>::from_sdl_window(
            window,
        ));

        let event_loop = sdl.event_pump()?;

        Ok(Self {
            sdl_context: gl_context,
            sdl_event_pump: event_loop,
            glue_event: Some(GlueEvent::new()),
            gl: Arc::new(Box::new(context)),
            render_loop,
        })
    }
}

fn extract_optional_paramaters(config: &str) -> (u32, u32, String, u8, u8) {
    let mut width = 800;
    let mut height = 600;
    let mut title = String::from("g_lue window");
    let context_major: u8 = 3;
    let context_minor: u8 = 1;

    let parser = XMLParser::new().parse(&String::from(config)).unwrap();

    parser
        .search("width", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    token.content.parse().map_or((), |num| {
                        width = num;
                    });
                });
                false
            });
        });

    parser
        .search("height", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    token.content.parse().map_or((), |num| {
                        height = num;
                    });
                });
                false
            });
        });

    parser
        .search("title", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    title = token.content.clone();
                });
                false
            });
        });

    (width, height, title, context_major, context_minor)
}

impl From<String> for GlueError {
    fn from(string: String) -> Self {
        Self::WindowInitError(string)
    }
}
impl From<sdl2::video::WindowBuildError> for GlueError {
    fn from(build_err: sdl2::video::WindowBuildError) -> Self {
        GlueError::WindowInitError(build_err.to_string())
    }
}
