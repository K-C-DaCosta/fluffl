#![allow(unused_imports)]
use g_lue::parsers::xml;
use serde::{Deserialize, Serialize};
// use serde_json::Result;

use g_lue::collections::linked_list::*;
use g_lue::window_util::*;
use std::cell::*;
use std::convert;
use std::mem;
use std::ops;
use std::rc::*;

use ec_composer::*;
use g_lue::ec_util::*;
use g_lue::window_util::event_util::constants::*;
use glow::*;
use std::mem::*; 

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
use std::num::ParseIntError;

fn main() -> Result<(), GlueError> {
    // let gd = GameData::new();
    // let mut gs = gd.init_tables();

    // for &pos in [(10, 20), (30, 40), (50, 60)].iter() {
    //     let constructor = gd.borrow().billiardball_factory.entity_constructor();
    //     let entity = gs.new_entity(constructor);
    //     gd.mutate_data::<DataBorrow, _, _>(|data| {
    //         let (_, transform_col) = entity.get_component_chain()[0].get_index_pair_part();
    //         data.transform[transform_col as usize].x = pos.0 as f32;
    //         data.transform[transform_col as usize].y = pos.1 as f32;
    //     });
    //     gd.borrow_mut().billiardball_factory.push_entity(entity);
    //     gd.mutate_data::<DataBorrow, _, _>(|data| {
    //         //update most recently added billiard ball
    //         gs.update_entity(data.billiardball_factory.billiard_balls.last().unwrap());
    //     });
    // }

    let config_text = "
    <window>
        <width>800</width>
        <height>600</height>
        <title>title</title>
    </window>";

    let window = GlueWindow::init(config_text)?;
    unsafe {
        window.gl.clear_color(1., 0., 0., 0.);
        window.gl.viewport(0, 0, 800, 600);
    }

    window.main_loop(|window, running| {
        let gl = window.gl.clone();
        for event in window.get_events().iter_mut() {
            match event {
                EventKind::Quit => *running = false,
                EventKind::KeyDown{ code} => {
                    let code :i128 = code.into(); 

                    if (code > KeyCode::KEY_A.into()) || (code < KeyCode::KEY_Z.into()) {
                        println!("char = {}\n",(code as u8 as char).to_lowercase());
                    }
                    
                },
                _ => (),
            }
        }
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    });
    Ok(())
}
