#![allow(unused_imports)]
use super::parsers::xml;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
// use serde_json::Result;
use super::collections::linked_list::*;
use super::window_util::*;
use std::cell::*;
use std::convert;
use std::mem;
use std::ops;
use std::rc::*;

use super::ec_util::*;
use super::window_util::event_util::constants::*;
use ec_composer::*;
use glow::*;
use std::mem::*;

// #[test]
// //Heres im just opening a window that outputs events to stdout
// fn keyboard_input_test() -> Result<(), GlueError> {
//     //GlueWindow is configured with XML, the format is self-explanitory
//     let config_text = "
//     <window>
//         <width>800</width>
//         <height>600</height>
//         <title>my_app</title>
//     </window>";

//     let window = GlueWindow::init(config_text)?;

//     unsafe {
//         window.gl().clear_color(1., 0., 0., 0.);
//         window.gl().viewport(0, 0, 800, 600);
//     }

//     window.main_loop(|window, running| {
//         let gl = window.gl();
//         for event in window.get_events().iter_mut() {
//             match event {
//                 EventKind::Quit => *running = false,
//                 EventKind::KeyDown { code } => {
//                     let code: i128 = code.into();
//                     if (code > KeyCode::KEY_A.into()) || (code < KeyCode::KEY_Z.into()) {
//                         println!("char = {}\n", (code as u8 as char).to_lowercase());
//                     }
//                 }
//                 EventKind::MouseMove { x, y, dx, dy } => {
//                     println!("mouse move: [x:{},y:{},dx:{},dy:{}", x, y, dx, dy);
//                 }
//                 EventKind::MouseUp { button_code, x, y } => {
//                     println!("mouse down at: [x:{},y:{}]", x, y);
//                     println!("{}", button_code);
//                 }
//                 EventKind::MouseWheel { button_code } => {
//                     println!("{}", button_code);
//                 }
//                 _ => (),
//             }
//         }
//         unsafe {
//             gl.clear(glow::COLOR_BUFFER_BIT);
//         }
//     });

//     Ok(())
// }

#[test]
fn compositon_interface_test() {
    let gd = GameData::new();
    let mut gs = gd.init_tables();
    for &pos in [(10, 20), (30, 40), (50, 60)].iter() {
        let constructor = gd.borrow().billiardball_factory.entity_constructor();
        let entity = gs.new_entity(constructor);
        gd.mutate_data::<DataBorrow, _, _>(|data| {
            let (_, transform_col) = entity.get_component_chain()[0].get_index_pair_part();
            data.transform[transform_col as usize].x = pos.0 as f32;
            data.transform[transform_col as usize].y = pos.1 as f32;
        });
        gd.borrow_mut().billiardball_factory.push_entity(entity);
        gd.mutate_data::<DataBorrow, _, _>(|data| {
            //update most recently added billiard ball
            gs.update_entity(data.billiardball_factory.billiard_balls.last().unwrap());
        });
    }
}
