#![allow(dead_code)]

use super::super::collections::{nary_forest::*,Ptr};
use std::collections::HashMap;

#[derive(Debug)]
pub enum XMLErrorKind {
    TokenizerErr(&'static str),
    ParserErr(&'static str),
}

#[derive(Copy, Clone)]
pub enum XMLTokenKind {
    //a token is either 'open',close,openclose, or inner;
    Open,
    Close,
    OpenClose,
    Inner,
    //aux token types(these act like states)
    Unknown,
    OpenAttribOpen,
    OpenAttribClose,
    Comment,
}

pub struct XMLToken {
    pub token_kind: XMLTokenKind,
    pub content: String,
    pub attribs: HashMap<String, String>,
}

impl XMLToken {
    pub fn new(token_kind: XMLTokenKind, content: String) -> XMLToken {
        XMLToken {
            token_kind,
            content,
            attribs: HashMap::new(),
        }
    }
}
impl Default for XMLToken {
    fn default() -> XMLToken {
        XMLToken {
            token_kind: XMLTokenKind::Unknown,
            content: String::new(),
            attribs: HashMap::new(),
        }
    }
}

/// Can correctly parse  only  a subset of XML grammar *only*.\
/// I repeat, this code  cannot parse the entire XML grammar. The parser was intented to parse xml that stores raw data.\
/// All the `<!DOCTYPE .. >`, `<!ENTITY ..>` stuff has been cut out of the grammar in this parser \
/// Comments should still work though.
pub struct XMLParser {
    pub tokens: Vec<Option<XMLToken>>,
    pub ast: NaryForest<XMLToken>,
    pub ident_table: HashMap<String, u32>,
}

impl XMLParser {
    pub fn new() -> XMLParser {
        XMLParser {
            tokens: Vec::new(),
            ast: NaryForest::new(),
            ident_table: HashMap::new(),
        }
    }

    ///tokenizes raw  xml text with FSM logic
    fn lexer(mut self, src: &str) -> Result<XMLParser, XMLErrorKind> {
        use XMLTokenKind::*;
        let mut state = Unknown;
        let mut accum = String::new();
        let mut current_key = String::new();

        let mut char_iter = src.chars().peekable();
        while let Some(c) = char_iter.next() {
            match state {
                Unknown => {
                    if c == '<' {
                        state = Open;
                    }
                }
                Open => {
                    if c == '>' {
                        state = Inner;
                        self.push_token(Open, &mut accum);
                    } else if let ('/', Some('>')) = (c, char_iter.peek()) {
                        state = Inner;
                        char_iter.next();
                        self.push_token(OpenClose, &mut accum);
                    } else if let (' ', Some(lookahead)) = (c, char_iter.peek()) {
                        if lookahead.is_alphabetic() {
                            state = XMLTokenKind::OpenAttribOpen;
                            //label token as "open" by default
                            self.push_token(Open, &mut accum);
                        }
                    } else {
                        let adding_first_character = accum.len() == 0;
                        if adding_first_character {
                            if c.is_alphabetic() {
                                accum.push(c);
                            }
                        } else {
                            accum.push(c);
                        }
                    }
                }
                OpenAttribOpen => {
                    if c == '=' {
                        current_key = accum.clone();
                        accum.clear();
                        if let Some('"') = char_iter.peek() {
                            state = OpenAttribClose;
                            char_iter.next();
                        } else {
                            return Err(XMLErrorKind::TokenizerErr(
                                "expected: '\"' right after '='",
                            ));
                        }
                    } else if c == '>' {
                        state = Inner;
                    } else if let ('/', Some('>')) = (c, char_iter.peek()) {
                        state = Inner;
                        char_iter.next();

                        //make sure existing open token is flagged as "openclose"
                        let open_token = &mut self.tokens.last_mut().unwrap().as_mut().unwrap();
                        open_token.token_kind = OpenClose;
                    } else {
                        accum.push(c);
                    }
                }
                OpenAttribClose => {
                    if c == '\"' {
                        let open_token = &mut self.tokens.last_mut().unwrap().as_mut().unwrap();
                        open_token
                            .attribs
                            .insert(current_key.clone(), accum.clone());
                        accum.clear();

                        state = XMLTokenKind::OpenAttribOpen;
                    } else {
                        accum.push(c);
                    }
                }
                Close => {
                    if c == '>' {
                        state = Inner;
                        self.push_token(Close, &mut accum);
                    } else {
                        accum.push(c);
                    }
                }
                Inner => {
                    if c == '<' {
                        let peek = char_iter.peek();
                        if let Some('/') = peek {
                            char_iter.next();
                            state = Close;
                        } else if let Some('!') = peek {
                            char_iter.next();
                            state = Comment;
                        } else {
                            state = Open;
                        }
                        self.push_token(Inner, &mut accum);
                    } else {
                        accum.push(c);
                    }
                }
                Comment => {
                    if c == '-' {
                        let peek = char_iter.peek();
                        if let Some('>') = peek {
                            state = Inner;
                            char_iter.next();
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(self)
    }

    ///Builds AST with an explicit stack
    pub fn parse(mut self, src: &String) -> Result<XMLParser, XMLErrorKind> {
        use XMLTokenKind::*;

        //lex raw text first
        self = self.lexer(src)?;
        // self.print_tokens();

        //init ast_stack with the root_node
        let root_token = self.tokens[0].take().unwrap();
        let root_node_ptr = self.ast.allocate(root_token);
        let mut ast_stack = vec![root_node_ptr];

        for k in 1..self.tokens.len() {
            if let Some(&parent_ptr) = ast_stack.last() {
                let current_token = self.tokens[k].take().unwrap();
                if let Open = current_token.token_kind {
                    let node_ptr = self.ast.allocate(current_token);
                    self.ast.add_child(parent_ptr, node_ptr);
                    ast_stack.push(node_ptr);
                } else if let Close = current_token.token_kind {
                    let open_tag_name = &self.ast[parent_ptr].data.as_ref().unwrap().content;
                    let close_tag_name = &current_token.content;
                    if open_tag_name != close_tag_name {
                        return Err(XMLErrorKind::ParserErr("Tags mismatch"));
                    }
                    if let None = ast_stack.pop() {
                        return Err(XMLErrorKind::ParserErr("Close Tag without Opening Tag"));
                    }
                } else if let Inner = current_token.token_kind {
                    let node_ptr = self.ast.allocate(current_token);
                    self.ast.add_child(parent_ptr, node_ptr);
                } else if let OpenClose = current_token.token_kind {
                    let node_ptr = self.ast.allocate(current_token);
                    self.ast.add_child(parent_ptr, node_ptr);
                }
            } else {
                return Err(XMLErrorKind::ParserErr("Extra Tag"));
            }
        }
        if ast_stack.is_empty() == false {
            return Err(XMLErrorKind::ParserErr(
                "Opening tags do not match close tags",
            ));
        }
        self.tokens.clear();
        //set root of tree
        self.ast.root_list.push(root_node_ptr);
        Ok(self)
    }

    fn push_token(&mut self, token_kind: XMLTokenKind, accum: &mut String) {
        if accum.len() == 0 || accum.trim().len() == 0 {
            accum.clear();
            return;
        }

        let token = XMLToken::new(token_kind, accum.clone());
        self.tokens.push(Some(token));
        accum.clear();
    }
    #[allow(dead_code)]
    pub fn print_tokens(&self) {
        for tok in self.tokens.iter() {
            match &tok {
                Some(XMLToken {
                    token_kind: XMLTokenKind::Open,
                    content: txt,
                    ..
                }) => {
                    println!("kind=Open Content=\'{}\'", txt);
                }
                Some(XMLToken {
                    token_kind: XMLTokenKind::Inner,
                    content: txt,
                    ..
                }) => {
                    println!("kind=Inner Content=\'{}\'", txt.trim());
                }
                Some(XMLToken {
                    token_kind: XMLTokenKind::Close,
                    content: txt,
                    ..
                }) => {
                    println!("kind=Close Content=\'{}\'", txt);
                }
                Some(XMLToken {
                    token_kind: XMLTokenKind::OpenClose,
                    content: txt,
                    ..
                }) => {
                    println!("kind=OpenClose Content=\'{}\'", txt.trim());
                }
                _ => println!("kind=???"),
            }
        }
    }

    pub fn print_tree(&self) {
        let mut char_stack = String::new();
        self.print_tree_helper(self.ast.root_list[0], &mut char_stack, ".");
    }

    fn print_tree_helper(&self, node_ptr: Ptr, char_stack: &mut String, c_kind: &'static str) {
        if node_ptr == Ptr::null() {
            return;
        }

        println!(
            "{}{}",
            char_stack,
            self.ast[node_ptr].data.as_ref().unwrap().content.trim()
        );

        char_stack.push_str(c_kind);

        for child_ptr in self.ast[node_ptr].children.iter() {
            self.print_tree_helper(*child_ptr, char_stack, c_kind);
        }

        (0..c_kind.len()).for_each(|_| {
            char_stack.pop();
            ()
        });
    }

    ///converts the xml AST back to text form
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        if let Some(&root) = self.ast.root_list.get(0) {
            self.to_xml_helper(root, &mut xml);
        }
        xml
    }
    ///  The recursive helper function that renders the tree into a string (spacing is intact)\
    /// `xml_stream` the destination of the converted parse tree in text form\
    /// `note_ptr` the subtre\
    ///  I wrote this a while back but if I recall it does a depth first traversal over the tree\
    ///  converting all tokens into text form.
    fn to_xml_helper(&self, node_ptr: Ptr, xml_stream: &mut String) {
        if node_ptr == Ptr::null() {
            return;
        }
        match self.ast[node_ptr].data.as_ref() {
            Some(token) => match token.token_kind {
                XMLTokenKind::Open => {
                    xml_stream.push_str(format!("<{}", token.content).as_str());
                    for (key, val) in token.attribs.iter() {
                        xml_stream.push_str(format!(" {}=\"{}\"", key, val).as_str());
                    }
                    xml_stream.push('>');
                    for &child in self.ast[node_ptr].children.iter() {
                        self.to_xml_helper(child, xml_stream);
                    }
                    xml_stream.push_str(format!("</{}>", token.content).as_str());
                }
                XMLTokenKind::Inner => {
                    xml_stream.push_str(format!("{}", token.content).as_str());
                }
                XMLTokenKind::OpenClose => {
                    xml_stream.push_str(format!("<{}", token.content).as_str());
                    for (key, val) in token.attribs.iter() {
                        xml_stream.push_str(format!(" {}=\"{}\"", key, val).as_str());
                    }
                    xml_stream.push_str("/>");
                }
                _ => (),
            },
            None => (),
        }
    }

    /// Like to_xml(..) with removes all spacing
    pub fn to_xml_trim(&self) -> String {
        let mut xml = String::new();
        if let Some(&root) = self.ast.root_list.get(0) {
            self.to_xml_helper_trim(root, &mut xml);
        }
        xml
    }

    /// Pretty much a clone of to_xml_helper(...) but with formatting and trimming in the mix/
    /// Maybe write one helper function that does both?
    fn to_xml_helper_trim(&self, node_ptr: Ptr, xml_stream: &mut String) {
        if node_ptr == Ptr::null() {
            return;
        }
        match self.ast[node_ptr].data.as_ref() {
            Some(token) => match token.token_kind {
                XMLTokenKind::Open => {
                    xml_stream.push_str(format!("<{}", token.content).as_str().trim());
                    for (key, val) in token.attribs.iter() {
                        xml_stream.push_str(format!(" {}=\"{}\"", key.trim(), val.trim()).as_str());
                    }
                    xml_stream.push('>');
                    for &child in self.ast[node_ptr].children.iter() {
                        self.to_xml_helper_trim(child, xml_stream);
                    }
                    xml_stream.push_str(format!("</{}>", token.content.trim()).as_str());
                }
                XMLTokenKind::Inner => {
                    xml_stream.push_str(format!("{}", token.content.trim()).as_str());
                }
                XMLTokenKind::OpenClose => {
                    xml_stream.push_str(format!("<{} ", token.content.trim()).as_str());
                    for (key, val) in token.attribs.iter() {
                        xml_stream.push_str(format!(" {}=\"{}\"", key.trim(), val.trim()).as_str());
                    }
                    xml_stream.push_str("/>");
                }
                _ => (),
            },
            None => (),
        }
    }

    /// finds first occurrence of tag matching "content"
    /// returns !0  if search fails
    pub fn search(&self, content: &str, node_ptr: Ptr) -> Option<Ptr> {
        if node_ptr == Ptr::null() {
            return None;
        }
        if self.ast[node_ptr].data.as_ref().unwrap().content == content {
            Some(node_ptr)
        } else {
            for &child_ptr in self.ast[node_ptr].children.iter() {
                let result = self.search(content, child_ptr);
                if result.is_some() {
                    return result;
                }
            }
            None
        }
    }

    /// Returns an iterator that walks through child tokens in xml ast
    pub fn get_child_tokens(
        &self,
        node_ptr: Ptr,
    ) -> impl Iterator<Item = ( usize, Option<&XMLToken> )>
    {
        //without get_child_tokens(...) I would have to do something like this EVERY time i wanted to iterate through
        //the children of a particular node
        self.ast[node_ptr]
            .children
            .iter()
            .map(move |&ptr| self.ast[ptr].data.as_ref())
            .enumerate()
    }
}
