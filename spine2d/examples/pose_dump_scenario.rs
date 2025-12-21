use serde_json::json;
use spine2d::{
    AnimationState, AnimationStateData, MixBlend, Skeleton, SkeletonData, TrackEntryHandle,
};
use std::path::PathBuf;
use std::sync::Arc;

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage:\n  pose_dump_scenario <skeleton.(json|skel)> <commands...>\n\nCommands:\n  --set-skin <name|none>\n  --dump-slot-vertices <slotName>\n  --dump-update-cache\n  --mix <from> <to> <duration>\n  --set <track> <animation> <loop 0|1>\n  --add <track> <animation> <loop 0|1> <delay>\n  --set-empty <track> <mixDuration>\n  --add-empty <track> <mixDuration> <delay>\n  --entry-alpha <alpha>\n  --entry-hold-previous <0|1>\n  --entry-mix-blend <setup|first|replace|add>\n  --entry-reverse <0|1>\n  --entry-shortest-rotation <0|1>\n  --entry-reset-rotation-directions\n  --step <dt>\n"
    );
    std::process::exit(2);
}

fn parse_mix_blend(s: &str) -> Option<MixBlend> {
    match s {
        "setup" => Some(MixBlend::Setup),
        "first" => Some(MixBlend::First),
        "replace" => Some(MixBlend::Replace),
        "add" => Some(MixBlend::Add),
        _ => None,
    }
}

fn load_skeleton_data(path: &PathBuf) -> Arc<SkeletonData> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("skel") {
        #[cfg(feature = "binary")]
        {
            let bytes = std::fs::read(path).expect("read skel");
            return SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
        }
        #[cfg(not(feature = "binary"))]
        {
            panic!("Input is .skel but spine2d was built without feature `binary`.");
        }
    }

    let json = std::fs::read_to_string(path).expect("read json");
    SkeletonData::from_json_str(&json).expect("parse json")
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_usage_and_exit();
    }

    let json_path = PathBuf::from(args.remove(0));
    let mut dump_slot_vertices: Option<String> = None;
    let mut dump_update_cache: bool = false;
    let data: Arc<SkeletonData> = load_skeleton_data(&json_path);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut last_entry: Option<TrackEntryHandle> = None;
    let mut total_time = 0.0f32;

    // Setup pose once; scenario steps do not reset the skeleton each frame.
    skeleton.set_to_setup_pose();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dump-slot-vertices" if i + 1 < args.len() => {
                dump_slot_vertices = args.get(i + 1).cloned();
                i += 2;
            }
            "--dump-update-cache" => {
                dump_update_cache = true;
                i += 1;
            }
            "--set-skin" if i + 1 < args.len() => {
                let name = args[i + 1].as_str();
                if name == "none" {
                    skeleton.set_skin(None).expect("set skin");
                } else {
                    skeleton.set_skin(Some(name)).expect("set skin");
                }
                i += 2;
            }
            "--mix" if i + 3 < args.len() => {
                let from = args[i + 1].as_str();
                let to = args[i + 2].as_str();
                let duration: f32 = args[i + 3].parse().unwrap();
                state
                    .data_mut()
                    .set_mix(from, to, duration)
                    .expect("set mix");
                i += 4;
            }
            "--set" if i + 3 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let anim = args[i + 2].as_str();
                let looped: bool = args[i + 3].parse::<i32>().unwrap_or(0) != 0;
                last_entry = Some(
                    state
                        .set_animation(track, anim, looped)
                        .expect("set animation"),
                );
                i += 4;
            }
            "--add" if i + 4 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let anim = args[i + 2].as_str();
                let looped: bool = args[i + 3].parse::<i32>().unwrap_or(0) != 0;
                let delay: f32 = args[i + 4].parse().unwrap();
                last_entry = Some(
                    state
                        .add_animation(track, anim, looped, delay)
                        .expect("add animation"),
                );
                i += 5;
            }
            "--set-empty" if i + 2 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let mix_duration: f32 = args[i + 2].parse().unwrap();
                last_entry = Some(
                    state
                        .set_empty_animation(track, mix_duration)
                        .expect("set empty animation"),
                );
                i += 3;
            }
            "--add-empty" if i + 3 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let mix_duration: f32 = args[i + 2].parse().unwrap();
                let delay: f32 = args[i + 3].parse().unwrap();
                last_entry = Some(
                    state
                        .add_empty_animation(track, mix_duration, delay)
                        .expect("add empty animation"),
                );
                i += 4;
            }
            "--entry-alpha" if i + 1 < args.len() => {
                let alpha: f32 = args[i + 1].parse().unwrap();
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-alpha requires a preceding --set/--add"))
                    .set_alpha(&mut state, alpha);
                i += 2;
            }
            "--entry-hold-previous" if i + 1 < args.len() => {
                let hold_previous: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-hold-previous requires a preceding --set/--add")
                    })
                    .set_hold_previous(&mut state, hold_previous);
                i += 2;
            }
            "--entry-mix-blend" if i + 1 < args.len() => {
                let mix_blend = parse_mix_blend(args[i + 1].as_str())
                    .unwrap_or_else(|| panic!("invalid mix blend: {}", args[i + 1]));
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-mix-blend requires a preceding --set/--add"))
                    .set_mix_blend(&mut state, mix_blend);
                i += 2;
            }
            "--entry-reverse" if i + 1 < args.len() => {
                let reverse: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-reverse requires a preceding --set/--add"))
                    .set_reverse(&mut state, reverse);
                i += 2;
            }
            "--entry-shortest-rotation" if i + 1 < args.len() => {
                let shortest_rotation: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-shortest-rotation requires a preceding --set/--add")
                    })
                    .set_shortest_rotation(&mut state, shortest_rotation);
                i += 2;
            }
            "--entry-reset-rotation-directions" => {
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-reset-rotation-directions requires a preceding --set/--add")
                    })
                    .reset_rotation_directions(&mut state);
                i += 1;
            }
            "--step" if i + 1 < args.len() => {
                let dt: f32 = args[i + 1].parse().unwrap();
                state.update(dt);
                state.apply(&mut skeleton);
                skeleton.update_world_transform();
                total_time += dt;
                i += 2;
            }
            _ => {
                print_usage_and_exit();
            }
        }
    }

    let bones: Vec<_> = skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(i, bone)| {
            let name = skeleton
                .data
                .bones
                .get(i)
                .map(|b| b.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "active": if bone.active { 1 } else { 0 },
                "world": {"a": bone.a, "b": bone.b, "c": bone.c, "d": bone.d, "x": bone.world_x, "y": bone.world_y},
                "applied": {"x": bone.ax, "y": bone.ay, "rotation": bone.arotation, "scaleX": bone.ascale_x, "scaleY": bone.ascale_y, "shearX": bone.ashear_x, "shearY": bone.ashear_y},
            })
        })
        .collect();

    let slots: Vec<_> = skeleton
        .slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let name = skeleton
                .data
                .slots
                .get(i)
                .map(|s| s.name.as_str())
                .unwrap_or("<unknown>");
            let attachment = skeleton
                .slot_attachment_data(i)
                .map(|a| json!({"name": a.name()}));
            let has_dark = if slot.has_dark { 1 } else { 0 };
            let dark_color = if slot.has_dark {
                [
                    slot.dark_color[0],
                    slot.dark_color[1],
                    slot.dark_color[2],
                    1.0,
                ]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            json!({
                "i": i,
                "name": name,
                "color": slot.color,
                "hasDark": has_dark,
                "darkColor": dark_color,
                "attachment": attachment,
            })
        })
        .collect();

    let draw_order: Vec<_> = skeleton
        .draw_order
        .iter()
        .copied()
        .map(|slot_index| slot_index as i32)
        .collect();

    let ik_constraints: Vec<_> = skeleton
        .ik_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .ik_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "mix": c.mix,
                "softness": c.softness,
                "bendDirection": c.bend_direction,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let transform_constraints: Vec<_> = skeleton
        .transform_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .transform_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "mixRotate": c.mix_rotate,
                "mixX": c.mix_x,
                "mixY": c.mix_y,
                "mixScaleX": c.mix_scale_x,
                "mixScaleY": c.mix_scale_y,
                "mixShearY": c.mix_shear_y,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let path_constraints: Vec<_> = skeleton
        .path_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .path_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "position": c.position,
                "spacing": c.spacing,
                "mixRotate": c.mix_rotate,
                "mixX": c.mix_x,
                "mixY": c.mix_y,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let mut debug_map = serde_json::Map::new();
    if dump_update_cache {
        debug_map.insert(
            "updateCache".to_string(),
            json!(skeleton.debug_update_cache()),
        );
        let transform_constraint_data: Vec<_> = skeleton
            .data
            .transform_constraints
            .iter()
            .map(|c| {
                let bone_names: Vec<_> = c
                    .bones
                    .iter()
                    .filter_map(|&i| skeleton.data.bones.get(i).map(|b| b.name.as_str()))
                    .collect();
                let source_name = skeleton
                    .data
                    .bones
                    .get(c.source)
                    .map(|b| b.name.as_str())
                    .unwrap_or("<unknown>");
                json!({
                    "name": c.name,
                    "bones": c.bones.len(),
                    "boneNames": bone_names,
                    "source": source_name,
                    "properties": c.properties.len(),
                    "mixX": c.mix_x,
                    "mixY": c.mix_y,
                    "localSource": c.local_source,
                    "localTarget": c.local_target,
                    "additive": c.additive,
                    "clamp": c.clamp,
                })
            })
            .collect();
        debug_map.insert(
            "transformConstraintData".to_string(),
            json!(transform_constraint_data),
        );

        debug_map.insert(
            "invalidAppliedBones".to_string(),
            json!(skeleton.debug_invalid_applied_bones()),
        );
    }
    if let Some(slot_name) = dump_slot_vertices.as_deref() {
        if let Some(slot_index) = skeleton.data.slots.iter().position(|s| s.name == slot_name) {
            if let Some(world_vertices) = skeleton.slot_vertex_attachment_world_vertices(slot_index)
            {
                debug_map.insert("slot".to_string(), json!(slot_name));
                debug_map.insert("slotIndex".to_string(), json!(slot_index as i32));
                debug_map.insert("worldVertices".to_string(), json!(world_vertices));
            }
        }
    }
    let debug = if debug_map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(debug_map))
    };

    let out = json!({
        "mode": "scenario",
        "time": total_time,
        "bones": bones,
        "slots": slots,
        "drawOrder": draw_order,
        "ikConstraints": ik_constraints,
        "transformConstraints": transform_constraints,
        "pathConstraints": path_constraints,
        "debug": debug,
    });

    println!("{}", serde_json::to_string(&out).expect("json"));
}
