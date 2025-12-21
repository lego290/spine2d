use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData};
use std::path::PathBuf;

fn upstream_examples_root() -> PathBuf {
    if let Ok(dir) = std::env::var("SPINE2D_UPSTREAM_EXAMPLES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../assets/spine-runtimes/examples"),
        manifest_dir.join("../third_party/spine-runtimes/examples"),
        manifest_dir.join("../.cache/spine-runtimes/examples"),
    ];
    for p in candidates {
        if p.is_dir() {
            return p;
        }
    }

    panic!(
        "Upstream Spine examples not found. Run `./scripts/import_spine_runtimes_examples.zsh --mode json` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_json_path(relative: &str) -> PathBuf {
    upstream_examples_root().join(relative)
}

fn bone_index(data: &SkeletonData, name: &str) -> usize {
    data.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("missing bone: {name}"))
}

fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot: {name}"))
}

fn transform_constraint_index(data: &SkeletonData, name: &str) -> usize {
    data.transform_constraints
        .iter()
        .position(|c| c.name == name)
        .unwrap_or_else(|| panic!("missing transform constraint: {name}"))
}

fn assert_approx(actual: f32, expected: f32) {
    let eps = 1.0e-6;
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "expected {expected}, got {actual} (diff {diff}, eps {eps})"
    );
}

#[test]
fn skin_required_active_and_gating_match_spine_cpp_semantics() {
    let path = example_json_path("mix-and-match/export/mix-and-match-pro.json");
    let json = std::fs::read_to_string(&path).expect("read mix-and-match-pro.json");
    let data = SkeletonData::from_json_str(&json).expect("parse mix-and-match-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_to_setup_pose();

    // Start from no skin, then set a skin. Upstream applies setup attachments from the new skin.
    skeleton
        .set_skin(Some("accessories/backpack"))
        .expect("set skin");

    let backpack_bone = bone_index(&data, "backpack");
    assert!(skeleton.bones[backpack_bone].active);
    let hat_control_bone = bone_index(&data, "hat-control");
    assert!(!skeleton.bones[hat_control_bone].active);

    let hat_control_constraint = transform_constraint_index(&data, "hat-control");
    assert!(!skeleton.transform_constraints[hat_control_constraint].active);

    let backpack_slot = slot_index(&data, "backpack");
    let key = skeleton.slots[backpack_slot]
        .attachment
        .as_deref()
        .expect("backpack setup attachment should be applied from skin");
    assert_eq!(key, "backpack");
    let resolved = skeleton
        .slot_attachment_data(backpack_slot)
        .expect("resolve backpack attachment");
    assert_eq!(resolved.name(), "boy/backpack");

    // Bone timeline gating: `aware` anim drives `hat-control.translate` but the bone is inactive
    // under this skin, so its local transform must remain at setup values.
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    state.set_animation(0, "aware", true).expect("set aware");
    state.update(0.1667);
    state.apply(&mut skeleton);

    let setup = &data.bones[hat_control_bone];
    let bone = &skeleton.bones[hat_control_bone];
    assert_approx(bone.x, setup.x);
    assert_approx(bone.y, setup.y);
}
