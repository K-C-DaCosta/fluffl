use super::*;

pub fn mousemove<ProgramState>() -> ListenerCallBack<ProgramState> {
    Box::new(|info| {
        let root_key = info.key;
        let gui_component_tree = info.gui_comp_tree;
        let mouse_pos = info.event.mouse_pos();
        let frame = get_frame(gui_component_tree, root_key);
        frame.last_known_mouse_pos = mouse_pos;
        None
    })
}

pub fn wheel<ProgramState>() -> ListenerCallBack<ProgramState> {
    Box::new(|info| {
        let root_key = info.key;
        let gui_component_tree = info.gui_comp_tree;

        resize_component_bounds_if_needed(gui_component_tree, root_key);

        let frame = get_frame(gui_component_tree, root_key);
        let wheel = info.event.wheel() * 0.125;
        let horizontal_scroll_area = frame.horizontal_scroll_area;

        if horizontal_scroll_area.is_point_inside(frame.last_known_mouse_pos) {
            frame.percentages[0] += wheel;
            frame.percentages[0] = frame.percentages[0].clamp(0.0, 1.0);
            let uv = frame.percentages;
            scroll_elements(gui_component_tree, root_key, uv);
        } else {
            frame.percentages[1] += -wheel;
            frame.percentages[1] = frame.percentages[1].clamp(0.0, 1.0);
            let uv = frame.percentages;
            scroll_elements(gui_component_tree, root_key, uv);
        }

        None
    })
}
pub fn drag<ProgramState>() -> ListenerCallBack<ProgramState> {
    Box::new(|info| {
        let root_key = info.key;
        let gui_component_tree = info.gui_comp_tree;
        resize_component_bounds_if_needed(gui_component_tree, root_key);

        let mouse_pos = info.event.mouse_pos();

        // found the percentages have to have a low-resolution step size otherwise
        // positioning gets fucked up for some reason
        // currently percentages are in { 0.01*k | k > 0 && k < 99 }
        let mouse_uv = {
            let frame = get_frame(gui_component_tree, root_key);
            let frame_pos = info.key_to_aabb_table.get(&root_key).unwrap().min_pos;
            let frame_bounds = frame.bounds;
            let mut uv = (mouse_pos - frame_pos) / (frame_bounds * 0.99);
            uv.iter_mut()
                .for_each(|comp| *comp = (comp.clamp(0.0, 1.0) * 100.0).floor() / 100.0);
            uv
        };

        let (horizontal_scroll_area, vertical_scroll_area, uv) = {
            let frame = get_frame(gui_component_tree, root_key);
            let hsa = frame.horizontal_scroll_area;
            let vsa = frame.vertical_scroll_area;

            (hsa, vsa, frame.percentages)
        };

        if horizontal_scroll_area.is_point_inside(mouse_pos) {
            scroll_elements(
                gui_component_tree,
                root_key,
                Vec2::from([mouse_uv.x(), uv.y()]),
            );
        }
        if vertical_scroll_area.is_point_inside(mouse_pos) {
            scroll_elements(
                gui_component_tree,
                root_key,
                Vec2::from([uv.x(), mouse_uv.y()]),
            );
        }
        None
    })
}

fn get_frame<'a>(
    tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    key: GuiComponentKey,
) -> &'a mut FrameState {
    tree.get_mut(key)
        .expect("root key invalid")
        .as_any_mut()
        .downcast_mut::<FrameState>()
        .expect("node expected to be a frame")
}

fn translate_children<'a>(
    tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
    disp: Vec2<f32>,
) {
    for NodeInfoMut { val, .. } in tree
        .iter_children_mut(root_key)
        .filter(|node| node.val.flags().is_set(component_flags::TITLEBAR) == false)
    {
        val.translate(disp);
    }
    let frame = get_frame(tree, root_key);
    frame.camera += disp;
    frame.components_aabb.translate(disp);
}

fn scroll_elements(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
    uv: Vec2<f32>,
) {
    let frame = get_frame(gui_component_tree, root_key);
    let new_min = frame.rails.unwrap().eval(uv.x(), uv.y());
    let disp = new_min - frame.components_aabb.min_pos;
    frame.percentages = uv;
    translate_children(gui_component_tree, root_key, disp)
}

fn compute_component_bounds(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
) -> AABB2<f32> {
    let mut aabb = AABB2::flipped_infinity();
    let mut executed = false;
    for NodeInfoMut { val, .. } in gui_component_tree.iter_children_mut(root_key).skip(1) {
        let &pos = val.rel_position();
        let bounds = val.get_bounds();
        let rel_aabb = AABB2::from_point_and_lengths(pos, bounds);
        aabb.merge(rel_aabb);
        executed = true;
    }

    if executed {
        aabb
    } else {
        AABB2::zero()
    }
}

fn resize_component_bounds_if_needed(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
) {
    let old_component_bounding_box = get_frame(gui_component_tree, root_key).components_aabb;
    let new_component_bounding_box = compute_component_bounds(gui_component_tree, root_key);
    const EPSILON: f32 = 0.001;

    let component_bounding_box_changed_dramatically = new_component_bounding_box
        .dims()
        .iter()
        .zip(old_component_bounding_box.dims().iter())
        .all(|(&cur, &prev)| (cur - prev).abs() < EPSILON)
        == false;

    if component_bounding_box_changed_dramatically {
        let old_uv = get_frame(gui_component_tree, root_key).percentages;
        scroll_elements(gui_component_tree, root_key, Vec2::zero());

        get_frame(gui_component_tree, root_key).rails = None;
        //compute NEW AABB in the coordinate original space
        get_frame(gui_component_tree, root_key).components_aabb =
            compute_component_bounds(gui_component_tree, root_key);
        get_frame(gui_component_tree, root_key)
            .update_component_bounds_assuming_new_bounds_already_set();

        scroll_elements(gui_component_tree, root_key, old_uv);
    }
}
