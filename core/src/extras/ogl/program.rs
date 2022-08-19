use super::*;
use crate::*;

pub struct OglProg {
    pub prog: glow::Program,
    pub gl: GlowGL,
}

impl OglProg {
    pub fn prog(&self) -> glow::Program {
        self.prog.clone()
    }
    /// # Description
    /// This function does some preprocessing to seperate concatenated shaders into individual shaders before compilation.\
    /// Shaders are seperated with preprocessor if statements. Example of `raw_source` format:\
    ///  \
    ///  ```c
    ///  #ifndef HEADER
    ///  --code here is concatenated to both vertex shader and fragment shader--
    ///  #endif
    ///  #ifndef VERTEX_SHADER
    ///   --shader source--
    ///  #endif
    ///  #ifndef FRAGMENT_SHADER
    ///   --shader source--
    ///  #endif
    ///  ```
    /// so again `raw_source` is just ONE block of text with multiple shaders jammed in one file.
    pub fn compile_program(gl: &GlowGL, raw_source: &str) -> Result<OglProg, CompilationError> {
        let tokens = tokensize_source(raw_source);
        let header = get_source_block(&tokens, "HEADER");
        let uniforms = get_source_block(&tokens, "UNIFORMS");
        let vertex_attributes = get_source_block(&tokens, "VERTEX_ATTRIBUTES");
        let vertex_shader = get_source_block(&tokens, "VERTEX_SHADER");
        let fragment_shader = get_source_block(&tokens, "FRAGMENT_SHADER");

        // println!("START_DUMP");
        // for (k, tok) in tokens.iter().enumerate() {
        //     print!("{}:{}", k, tok.to_string());
        // }
        // println!("END_DUMP");

        // println!("\n**START_BLOCK**\n");
        // uniforms.map(|(lbound, ubound)| {
        //     println!("({},{})", lbound, ubound);
        //     for k in lbound..=ubound {
        //         print!("{}", tokens[k].to_string());
        //     }
        // });
        // println!("\n**END_BLOCK**");

        let vertex_shader_module = gen_shader_module(
            &tokens,
            vec![header, uniforms, vertex_attributes, vertex_shader],
        );

        let fragment_shader_module =
            gen_shader_module(&tokens, vec![header, uniforms, fragment_shader]);

        let mut module_list = vec![
            (glow::VERTEX_SHADER, vertex_shader_module),
            (glow::FRAGMENT_SHADER, fragment_shader_module),
        ];

        let shader_iterator = module_list.iter_mut().filter(|(_, opt)| opt.is_some()).map(
            |(shader_type, opt)| unsafe {
                let source = opt.take().unwrap();
                let shader: glow::Shader = gl.create_shader(*shader_type).unwrap();
                gl.shader_source(shader, source.as_str());
                gl.compile_shader(shader);
                if gl.get_shader_compile_status(shader) == false {
                    let compile_error = gl.get_shader_info_log(shader);
                    Err(CompilationError::ShaderError {
                        ogl_error: compile_error,
                        faulty_source: source,
                    })
                } else {
                    Ok(shader)
                }
            },
        );

        let program = unsafe {
            let mut shaders = Vec::new();
            let program = gl.create_program().unwrap();

            for shader_res in shader_iterator {
                shaders.push(shader_res?);
                let cur_shader: glow::Shader = shaders.last().unwrap().clone();
                gl.attach_shader(program, cur_shader);
            }
            gl.link_program(program);
            if gl.get_program_link_status(program) == false {
                let ogl_error = gl.get_program_info_log(program);
                return Err(CompilationError::ShaderError {
                    ogl_error,
                    faulty_source: String::new(),
                });
            }
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            OglProg {
                gl: gl.clone(),
                prog: program,
            }
        };

        Ok(program)
    }
}

impl Drop for OglProg {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.prog);
        }
    }
}

impl Bindable for OglProg {
    fn bind(&self, opt: bool) {
        let gl = &self.gl;
        unsafe { gl.use_program(opt.map(|| self.prog())) }
    }
}
#[derive(Debug)]
pub enum CompilationError {
    ShaderError {
        ogl_error: String,
        faulty_source: String,
    },
    LinkError {
        ogl_error: String,
        faulty_source: String,
    },
}

enum SlToken {
    If(String),
    Ifndef(String),
    Define(String),
    Source(String),
    Endif,
}

impl ToString for SlToken {
    fn to_string(&self) -> String {
        match self {
            SlToken::If(text) => format!("#if {}\n", text),
            SlToken::Ifndef(text) => format!("#ifndef {}\n", text),
            SlToken::Define(text) => format!("#define {}\n", text),
            SlToken::Source(text) => format!("{}", text),
            SlToken::Endif => format!("#endif\n"),
        }
    }
}
#[derive(PartialEq)]
enum LexState {
    Start,
    If,
    Ifndef,
    Endif,
    Define,
}

fn tokensize_source(raw_source: &str) -> Vec<SlToken> {
    let mut tokens = Vec::new();
    let mut state = LexState::Start;
    let mut source_iter = raw_source.chars().peekable();
    let mut accum = String::new();

    let push_source = |accum: &mut String, tokens: &mut Vec<_>| {
        let all_whitespece = accum.chars().all(|a| a.is_whitespace());
        if accum.len() > 0 && all_whitespece == false {
            tokens.push(SlToken::Source(accum.clone()));
        }
        accum.clear();
    };

    while source_iter.peek().is_some() || accum.is_empty() == false || state != LexState::Start {
        let c = source_iter.next().unwrap_or_else(|| '\0');
        match state {
            LexState::Start => {
                if let ('#', true, _iter) = lookahead(c, "ifndef", source_iter.clone()) {
                    source_iter = _iter;
                    state = LexState::Ifndef;
                    push_source(&mut accum, &mut tokens);
                } else if let ('#', true, _iter) = lookahead(c, "define", source_iter.clone()) {
                    source_iter = _iter;
                    state = LexState::Define;
                    push_source(&mut accum, &mut tokens);
                } else if let ('#', true, _iter) = lookahead(c, "endif", source_iter.clone()) {
                    source_iter = _iter;
                    state = LexState::Endif;
                    push_source(&mut accum, &mut tokens);
                } else if let ('#', true, _iter) = lookahead(c, "if", source_iter.clone()) {
                    source_iter = _iter;
                    state = LexState::If;
                    push_source(&mut accum, &mut tokens);
                } else {
                    accum.push(c);
                }
            }
            LexState::Ifndef => {
                if c == '\n' || c == '\0' {
                    state = LexState::Start;
                    tokens.push(SlToken::Ifndef(accum.clone()));
                    accum.clear();
                } else if c.is_alphanumeric() || c == '_' {
                    accum.push(c);
                }
            }
            LexState::If => {
                if c == '\n' || c == '\0' {
                    state = LexState::Start;
                    tokens.push(SlToken::If(accum.clone()));
                    accum.clear();
                } else if c.is_alphanumeric() || c == '_' {
                    accum.push(c);
                }
            }
            LexState::Define => {
                if c == '\n' || c == '\0' {
                    state = LexState::Start;
                    tokens.push(SlToken::Define(accum.clone()));
                    accum.clear();
                } else if c.is_alphanumeric() || c == '_' {
                    accum.push(c);
                }
            }
            LexState::Endif => {
                if c == '\n' || c == '\0' {
                    state = LexState::Start;
                    tokens.push(SlToken::Endif);
                    accum.clear();
                }
            }
        }
    }
    tokens
}

fn lookahead<T>(c: char, ident: &'static str, mut iter: T) -> (char, bool, T)
where
    T: std::iter::Iterator<Item = char> + Sized,
{
    let eq_test = (&mut iter).take(ident.len()).collect::<String>() == ident;
    (c, eq_test, iter)
}

fn get_source_block(tokens: &Vec<SlToken>, block_ident: &'static str) -> Option<(usize, usize)> {
    let mut found_ifndef = false;
    let query: ArrayVec<[usize; 4]> = tokens
        .iter()
        .enumerate()
        .filter(|&(_, tok)| {
            if let SlToken::Ifndef(text) = tok {
                if text == block_ident {
                    found_ifndef = true;
                    true
                } else {
                    false
                }
            } else if let SlToken::Endif = tok {
                found_ifndef
            } else {
                false
            }
        })
        .take(2)
        .map(|(index, _)| index)
        .collect();

    if query.len() == 2 {
        Some((query[0], query[1]))
    } else {
        None
    }
}

fn gen_shader_module(
    tokens: &Vec<SlToken>,
    code_blocks: Vec<Option<(usize, usize)>>,
) -> Option<String> {
    let mut source = String::new();

    //the last element in the tokens list is always the core shader source ( void main(){ ... }  )
    if tokens.last().is_none() {
        return None;
    }

    code_blocks.iter().for_each(|opt| {
        opt.map(|(lbound, ubound)| {
            for k in lbound + 1..ubound {
                // print!("{}",tokens[k].to_string());
                source.push_str(tokens[k].to_string().as_str());
            }
        });
    });

    Some(source)
}
