use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData};
use std::path::PathBuf;
use std::sync::Arc;

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

fn run_all_animations_smoke(relative: &str) {
    let path = example_json_path(relative);
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.default_mix = 0.2; // matches upstream spine-c-unit-tests
    let mut state = AnimationState::new(state_data);

    let animations = data
        .animations
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    if animations.is_empty() {
        return;
    }

    state
        .set_animation(0, &animations[0], false)
        .unwrap_or_else(|e| panic!("set animation {}: {e}", animations[0]));
    for name in animations.iter().skip(1) {
        state
            .add_animation(0, name, false, 0.0)
            .unwrap_or_else(|e| panic!("add animation {name}: {e}"));
    }

    const MAX_RUN_TIME: usize = 6000; // matches upstream (about 100s at 60fps)
    for _ in 0..MAX_RUN_TIME {
        if state.with_track_entry(0, |_| ()).is_none() {
            break;
        }
        state.update(1.0 / 60.0);
        state.apply(&mut skeleton);
        skeleton.update_world_transform();
    }
}

#[test]
fn upstream_spine_c_interface_smoke_spineboy() {
    run_all_animations_smoke("spineboy/export/spineboy-ess.json");
}

#[test]
fn upstream_spine_c_interface_smoke_raptor() {
    run_all_animations_smoke("raptor/export/raptor-pro.json");
}

#[test]
fn upstream_spine_c_interface_smoke_goblins() {
    run_all_animations_smoke("goblins/export/goblins-pro.json");
}
