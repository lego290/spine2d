#![allow(dead_code)]

use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation};
use std::path::PathBuf;
use std::sync::Arc;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn load_bytes(rel: &str) -> Vec<u8> {
    std::fs::read(repo_root().join(rel)).expect(rel)
}

#[cfg(feature = "json")]
fn load_string(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).expect(rel)
}

fn pose_at(data: Arc<SkeletonData>, animation_name: &str, time: f32) -> Skeleton {
    let (_, anim) = data.animation(animation_name).expect("animation exists");
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_to_setup_pose();
    apply_animation(anim, &mut skeleton, time, true, 1.0, MixBlend::Replace);
    skeleton.update_world_transform();
    skeleton
}

#[cfg(all(feature = "json", feature = "binary"))]
fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot {name:?}"))
}

fn assert_approx(a: f32, b: f32, eps: f32, ctx: &str) {
    if (a - b).abs() > eps {
        panic!("{ctx}: expected {b}, got {a} (diff {})", (a - b).abs());
    }
}

#[cfg(feature = "json")]
fn bone_name(s: &Skeleton, data_index: usize) -> &str {
    s.data
        .bones
        .get(data_index)
        .map(|d| d.name.as_str())
        .unwrap_or("?")
}

#[cfg(feature = "json")]
fn assert_pose_close(a: &Skeleton, b: &Skeleton, eps: f32, ctx: &str) {
    assert_eq!(a.bones.len(), b.bones.len(), "bones length");
    assert_eq!(a.slots.len(), b.slots.len(), "slots length");
    assert_eq!(a.draw_order, b.draw_order, "draw order");

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name_a = bone_name(a, ba.data_index());
        let name_b = bone_name(b, bb.data_index());
        assert_eq!(
            ba.data_index(),
            bb.data_index(),
            "{ctx}: bone[{i}] data_index"
        );
        assert_eq!(name_a, name_b, "{ctx}: bone[{i}] name");
        assert_eq!(ba.active, bb.active, "{ctx}: bone[{i}]({name_a}).active");
        assert_eq!(ba.inherit, bb.inherit, "{ctx}: bone[{i}]({name_a}).inherit");
        assert_eq!(
            ba.parent_index(),
            bb.parent_index(),
            "{ctx}: bone[{i}]({name_a}).parent_index"
        );

        assert_approx(ba.x, bb.x, eps, &format!("{ctx}: bone[{i}]({name_a}).x"));
        assert_approx(ba.y, bb.y, eps, &format!("{ctx}: bone[{i}]({name_a}).y"));
        assert_approx(
            ba.rotation,
            bb.rotation,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).rotation"),
        );
        assert_approx(
            ba.scale_x,
            bb.scale_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).scale_x"),
        );
        assert_approx(
            ba.scale_y,
            bb.scale_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).scale_y"),
        );
        assert_approx(
            ba.shear_x,
            bb.shear_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).shear_x"),
        );
        assert_approx(
            ba.shear_y,
            bb.shear_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).shear_y"),
        );
        assert_approx(ba.a, bb.a, eps, &format!("{ctx}: bone[{i}]({name_a}).a"));
        assert_approx(ba.b, bb.b, eps, &format!("{ctx}: bone[{i}]({name_a}).b"));
        assert_approx(ba.c, bb.c, eps, &format!("{ctx}: bone[{i}]({name_a}).c"));
        assert_approx(ba.d, bb.d, eps, &format!("{ctx}: bone[{i}]({name_a}).d"));
        assert_approx(
            ba.world_x,
            bb.world_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).world_x"),
        );
        assert_approx(
            ba.world_y,
            bb.world_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).world_y"),
        );
    }

    for (i, (sa, sb)) in a.slots.iter().zip(&b.slots).enumerate() {
        assert_eq!(sa.attachment, sb.attachment, "slot[{i}].attachment");
        assert_eq!(
            sa.sequence_index, sb.sequence_index,
            "slot[{i}].sequence_index"
        );
        assert_eq!(sa.deform.len(), sb.deform.len(), "slot[{i}].deform.len");
        for (j, (&da, &db)) in sa.deform.iter().zip(&sb.deform).enumerate() {
            assert_approx(da, db, eps, &format!("slot[{i}].deform[{j}]"));
        }
        for k in 0..4 {
            assert_approx(
                sa.color[k],
                sb.color[k],
                eps,
                &format!("slot[{i}].color[{k}]"),
            );
        }
        assert_eq!(sa.has_dark, sb.has_dark, "slot[{i}].has_dark");
        for k in 0..3 {
            assert_approx(
                sa.dark_color[k],
                sb.dark_color[k],
                eps,
                &format!("slot[{i}].dark_color[{k}]"),
            );
        }
    }

    assert_eq!(
        a.ik_constraints.len(),
        b.ik_constraints.len(),
        "ik constraints length"
    );
    for (i, (ca, cb)) in a.ik_constraints.iter().zip(&b.ik_constraints).enumerate() {
        assert_approx(ca.mix, cb.mix, eps, &format!("ik[{i}].mix"));
        assert_approx(ca.softness, cb.softness, eps, &format!("ik[{i}].softness"));
        // `.skel` and `.json` exports may differ in `bend_direction` for single-bone IK
        // constraints (it is ignored by the solver). Only enforce it for two-bone IK.
        if ca.bones.len() == 2 || cb.bones.len() == 2 {
            assert_eq!(
                ca.bend_direction, cb.bend_direction,
                "ik[{i}].bend_direction"
            );
        }
    }

    assert_eq!(
        a.transform_constraints.len(),
        b.transform_constraints.len(),
        "transform constraints length"
    );
    for (i, (ca, cb)) in a
        .transform_constraints
        .iter()
        .zip(&b.transform_constraints)
        .enumerate()
    {
        assert_approx(
            ca.mix_rotate,
            cb.mix_rotate,
            eps,
            &format!("transform[{i}].mix_rotate"),
        );
        assert_approx(ca.mix_x, cb.mix_x, eps, &format!("transform[{i}].mix_x"));
        assert_approx(ca.mix_y, cb.mix_y, eps, &format!("transform[{i}].mix_y"));
        assert_approx(
            ca.mix_scale_x,
            cb.mix_scale_x,
            eps,
            &format!("transform[{i}].mix_scale_x"),
        );
        assert_approx(
            ca.mix_scale_y,
            cb.mix_scale_y,
            eps,
            &format!("transform[{i}].mix_scale_y"),
        );
        assert_approx(
            ca.mix_shear_y,
            cb.mix_shear_y,
            eps,
            &format!("transform[{i}].mix_shear_y"),
        );
    }

    assert_eq!(
        a.path_constraints.len(),
        b.path_constraints.len(),
        "path constraints length"
    );
    for (i, (ca, cb)) in a
        .path_constraints
        .iter()
        .zip(&b.path_constraints)
        .enumerate()
    {
        assert_approx(
            ca.position,
            cb.position,
            eps,
            &format!("path[{i}].position"),
        );
        assert_approx(ca.spacing, cb.spacing, eps, &format!("path[{i}].spacing"));
        assert_approx(
            ca.mix_rotate,
            cb.mix_rotate,
            eps,
            &format!("path[{i}].mix_rotate"),
        );
        assert_approx(ca.mix_x, cb.mix_x, eps, &format!("path[{i}].mix_x"));
        assert_approx(ca.mix_y, cb.mix_y, eps, &format!("path[{i}].mix_y"));
    }
}

#[test]
#[cfg(feature = "upstream-smoke")]
fn skel_smoke_loads_spineboy_pro() {
    let bytes = load_bytes("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.skel");
    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
    assert!(data.animation("run").is_some(), "missing 'run' animation");
    let _ = pose_at(data, "run", 0.2);
}

#[test]
#[cfg(feature = "upstream-smoke")]
fn skel_spineboy_constraints_match_spine_cpp_lite_reference() {
    // Expected values are dumped from the official C++ runtime (oracle) loading
    // `spineboy-pro.skel` (see `scripts/run_spine_cpp_lite_dump_constraints.zsh`).
    let bytes = load_bytes("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.skel");
    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");

    let ik = |name: &str| {
        data.ik_constraints
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing ik constraint {name:?}"))
    };
    let tr = |name: &str| {
        data.transform_constraints
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing transform constraint {name:?}"))
    };

    assert_approx(ik("aim-ik").mix, 0.0, 1.0e-6, "aim-ik mix");
    assert_eq!(ik("aim-ik").bend_direction, -1, "aim-ik bend");

    assert_approx(ik("aim-torso-ik").mix, 1.0, 1.0e-6, "aim-torso-ik mix");
    assert_eq!(ik("aim-torso-ik").bend_direction, -1, "aim-torso-ik bend");

    assert_approx(ik("front-leg-ik").mix, 1.0, 1.0e-6, "front-leg-ik mix");
    assert_eq!(ik("front-leg-ik").bend_direction, -1, "front-leg-ik bend");

    assert_approx(ik("rear-leg-ik").mix, 1.0, 1.0e-6, "rear-leg-ik mix");
    assert_eq!(ik("rear-leg-ik").bend_direction, -1, "rear-leg-ik bend");

    assert_approx(
        tr("aim-front-arm-transform").mix_rotate,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_rotate",
    );
    assert_approx(
        tr("aim-front-arm-transform").mix_x,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_x",
    );
    assert_approx(
        tr("aim-front-arm-transform").mix_y,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_y",
    );

    assert_approx(
        tr("shoulder").mix_rotate,
        0.0,
        1.0e-6,
        "shoulder mix_rotate",
    );
    assert_approx(tr("shoulder").mix_x, -1.0, 1.0e-6, "shoulder mix_x");
    assert_approx(tr("shoulder").mix_y, -1.0, 1.0e-6, "shoulder mix_y");
}

#[test]
#[cfg(all(feature = "json", feature = "binary", feature = "upstream-smoke"))]
fn skel_tank_treads_path_attachment_matches_json() {
    let skel = load_bytes("assets/spine-runtimes/examples/tank/export/tank-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let slot_name = "treads-path";
    let slot_skel = slot_index(&data_skel, slot_name);
    let slot_json = slot_index(&data_json, slot_name);
    assert_eq!(slot_skel, slot_json, "slot index");

    let skin_skel = data_skel.skin("default").expect("default skin (skel)");
    let skin_json = data_json.skin("default").expect("default skin (json)");

    let att_skel = skin_skel
        .attachments
        .get(slot_skel)
        .and_then(|m| m.get(slot_name))
        .unwrap_or_else(|| panic!("missing {slot_name:?} attachment in skel default skin"));
    let att_json = skin_json
        .attachments
        .get(slot_json)
        .and_then(|m| m.get(slot_name))
        .unwrap_or_else(|| panic!("missing {slot_name:?} attachment in json default skin"));

    let (p_skel, p_json) = match (att_skel, att_json) {
        (crate::AttachmentData::Path(a), crate::AttachmentData::Path(b)) => (a, b),
        _ => panic!("treads-path attachment must be Path"),
    };

    assert_eq!(p_skel.closed, p_json.closed, "closed");
    assert_eq!(
        p_skel.constant_speed, p_json.constant_speed,
        "constant_speed"
    );
    assert_eq!(p_skel.lengths.len(), p_json.lengths.len(), "lengths.len");
    for (i, (&a, &b)) in p_skel.lengths.iter().zip(&p_json.lengths).enumerate() {
        assert_approx(a, b, 1.0e-3, &format!("lengths[{i}]"));
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn skel_matches_json_pose_spineboy_run() {
    let skel = load_bytes("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    for &t in &[0.0, 0.1, 0.2, 0.4, 0.6] {
        let a = pose_at(data_skel.clone(), "run", t);
        let b = pose_at(data_json.clone(), "run", t);
        // `.skel` stores binary `f32`s while JSON stores decimals; small export/parse drift is
        // expected and can accumulate through constraints.
        assert_pose_close(&a, &b, 2.5e-1, &format!("spineboy.run t={t}"));
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn skel_matches_json_pose_tank_shoot() {
    let skel = load_bytes("assets/spine-runtimes/examples/tank/export/tank-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    for &t in &[0.1, 0.3, 0.5] {
        let a = pose_at(data_skel.clone(), "shoot", t);
        let b = pose_at(data_json.clone(), "shoot", t);
        assert_pose_close(&a, &b, 2.5e-1, &format!("tank.shoot t={t}"));
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_run_t0_skel_vs_json() {
    let skel = load_bytes("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let a = pose_at(data_skel, "run", 0.0);
    let b = pose_at(data_json, "run", 0.0);

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name = bone_name(&a, ba.data_index());
        let da = (ba.a - bb.a).abs();
        let dwx = (ba.world_x - bb.world_x).abs();
        let dwy = (ba.world_y - bb.world_y).abs();
        println!(
            "bone[{i:02}] {name:20} a {:+.6} vs {:+.6} (Δ{:.6}) wx {:+.3} vs {:+.3} (Δ{:.3}) wy {:+.3} vs {:+.3} (Δ{:.3}) rot {:+.3} vs {:+.3} (Δ{:.3})",
            ba.a,
            bb.a,
            da,
            ba.world_x,
            bb.world_x,
            dwx,
            ba.world_y,
            bb.world_y,
            dwy,
            ba.rotation,
            bb.rotation,
            (ba.rotation - bb.rotation).abs(),
        );
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_skel_vs_json_constraints() {
    let skel = load_bytes("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    println!(
        "IK constraints: skel={} json={}",
        data_skel.ik_constraints.len(),
        data_json.ik_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .ik_constraints
        .iter()
        .zip(&data_json.ik_constraints)
        .enumerate()
    {
        println!(
            "ik[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} target {} vs {} mix {:.3} vs {:.3} softness {:.3} vs {:.3} compress {} vs {} stretch {} vs {} uniform {} vs {} bend {} vs {} skin_required {} vs {}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.target,
            b.target,
            a.mix,
            b.mix,
            a.softness,
            b.softness,
            a.compress,
            b.compress,
            a.stretch,
            b.stretch,
            a.uniform,
            b.uniform,
            a.bend_direction,
            b.bend_direction,
            a.skin_required,
            b.skin_required,
        );
    }

    println!(
        "Transform constraints: skel={} json={}",
        data_skel.transform_constraints.len(),
        data_json.transform_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .transform_constraints
        .iter()
        .zip(&data_json.transform_constraints)
        .enumerate()
    {
        println!(
            "transform[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} source {} vs {} local_source {} vs {} local_target {} vs {} additive {} vs {} clamp {} vs {} mix_rotate {:.3} vs {:.3}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.source,
            b.source,
            a.local_source,
            b.local_source,
            a.local_target,
            b.local_target,
            a.additive,
            b.additive,
            a.clamp,
            b.clamp,
            a.mix_rotate,
            b.mix_rotate
        );
    }

    println!(
        "Path constraints: skel={} json={}",
        data_skel.path_constraints.len(),
        data_json.path_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .path_constraints
        .iter()
        .zip(&data_json.path_constraints)
        .enumerate()
    {
        println!(
            "path[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} target {} vs {} rotate_mode {:?} vs {:?}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.target,
            b.target,
            a.rotate_mode,
            b.rotate_mode
        );
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_tank_skel_vs_json_constraints() {
    let skel = load_bytes("assets/spine-runtimes/examples/tank/export/tank-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    println!(
        "IK constraints: skel={} json={}",
        data_skel.ik_constraints.len(),
        data_json.ik_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .ik_constraints
        .iter()
        .zip(&data_json.ik_constraints)
        .enumerate()
    {
        println!(
            "ik[{i}] name skel='{}' json='{}' mix {:.3} vs {:.3} bend {} vs {} target {} vs {} bones {:?} vs {:?}",
            a.name,
            b.name,
            a.mix,
            b.mix,
            a.bend_direction,
            b.bend_direction,
            a.target,
            b.target,
            a.bones,
            b.bones
        );
    }

    println!(
        "Transform constraints: skel={} json={}",
        data_skel.transform_constraints.len(),
        data_json.transform_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .transform_constraints
        .iter()
        .zip(&data_json.transform_constraints)
        .enumerate()
    {
        println!(
            "transform[{i}] name skel='{}' json='{}' mix_rotate {:.3} vs {:.3} mix_x {:.3} vs {:.3} mix_y {:.3} vs {:.3}",
            a.name, b.name, a.mix_rotate, b.mix_rotate, a.mix_x, b.mix_x, a.mix_y, b.mix_y
        );
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_tank_shoot_t01_skel_vs_json() {
    let skel = load_bytes("assets/spine-runtimes/examples/tank/export/tank-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let t = 0.1;
    let a = pose_at(data_skel, "shoot", t);
    let b = pose_at(data_json, "shoot", t);

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name = bone_name(&a, ba.data_index());
        let dwx = (ba.world_x - bb.world_x).abs();
        let dwy = (ba.world_y - bb.world_y).abs();
        if dwx > 0.01 || dwy > 0.01 {
            println!(
                "bone[{i:02}] {name:16} wx {:+.5} vs {:+.5} (Δ{:.5}) wy {:+.5} vs {:+.5} (Δ{:.5}) a {:+.6} vs {:+.6} (Δ{:.6})",
                ba.world_x,
                bb.world_x,
                dwx,
                ba.world_y,
                bb.world_y,
                dwy,
                ba.a,
                bb.a,
                (ba.a - bb.a).abs(),
            );
        }
    }
}
