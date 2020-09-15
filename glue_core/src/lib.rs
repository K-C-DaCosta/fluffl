#![allow(warnings)]
pub mod audio;
pub mod collections;
pub mod ec_util;
pub mod io;
pub mod parsers;
pub mod window_util;
pub mod console; 
#[derive(Debug)]
pub enum GlueError {
    GenericError(String),
    FromUtf8ParseError(String),
    WindowInitError(String),
    IOError(String),
    WavParseError(String),
}

impl From<std::io::Error> for GlueError {
    fn from(err: std::io::Error) -> Self {
        GlueError::IOError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for GlueError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8ParseError(err.to_string())
    }
}

#[cfg(test)]
mod glue_tests;

// macro_rules! tuple_as {
//     (  $tuple_ident:ident , ( $($y:expr),*)  ) => {
//         {
//             let tuple = $tuple_ident;
//             let converted_tuple = (
//                 tuple_as_helper!(tuple, 1 ,  $($y:expr), *  )
//             );
//         }
//     };
// }

// macro_rules! tuple_as_helper{
//     ( $tuple:ident , $type_count:expr ) =>{

//     };
//     ( $tuple:ident , $type_count:expr , $e:expr ) =>{
//         // let y = $type_count;
//         $type_count
//     };
//     ( $tuple:ident , $type_count:expr , $e:expr, $($es:expr), +   ) =>{
//         // let y = $type_count;
//         ($type_count , tuple_as_helper!($tuple, (1+$type_count) , $($es:expr),* ))
//     };

// }



#[test]
fn serde_test_thing() -> serde_json::Result<()> {
    use window_util::event_util::EventKind;
    use window_util::event_util::KeyCode;
    use window_util::GlueWindow;

    let event_obj = serde_json::to_string(&EventKind::KeyUp {
        code: KeyCode::KEY_A,
    })?;
    println!("event obj :\n{}", event_obj);
    Ok(())
}
