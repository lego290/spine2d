use crate::runtime::{AnimationState, AnimationStateData};
use crate::{
    Animation, AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, BoneData,
    BoneTimeline, Curve, DrawOrderFrame, DrawOrderTimeline, Inherit, MixBlend,
    RegionAttachmentData, Skeleton, SkeletonData, SkinData, SlotData, TranslateTimeline, Vec2Frame,
};
use std::collections::HashMap;
use std::sync::Arc;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn base_skeleton_data() -> SkeletonData {
    SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
            skin_required: false,
        }],
        slots: Vec::new(),
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    }
}

#[test]
fn hold_previous_keeps_unkeyed_properties_from_fading_out() {
    let anim_a = Animation {
        name: "a".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 10.0,
                y: 0.0,
                curve: [Curve::Linear; 2],
            }],
        })],
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: Vec::new(),
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
    };
    let anim_b = Animation {
        name: "b".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: Vec::new(),
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
    };

    let mut data = base_skeleton_data();
    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("a", "b", 1.0).unwrap();

    // Without holdPrevious, A fades out for properties not keyed by B.
    {
        let mut state = AnimationState::new(state_data.clone());
        let mut skeleton = Skeleton::new(data.clone());

        state.set_animation(0, "a", false).unwrap();
        skeleton.set_to_setup_pose();
        state.apply(&mut skeleton);
        assert_approx(skeleton.bones[0].x, 10.0);

        state.set_animation(0, "b", false).unwrap();
        state.update(0.8);
        skeleton.set_to_setup_pose();
        state.apply(&mut skeleton);
        assert_approx(skeleton.bones[0].x, 2.0);
    }

    // With holdPrevious, A is held (alphaHold) instead of fading out (alphaMix).
    {
        let mut state = AnimationState::new(state_data);
        let mut skeleton = Skeleton::new(data);

        let a = state.set_animation(0, "a", false).unwrap();
        a.set_mix_blend(&mut state, MixBlend::Replace);
        skeleton.set_to_setup_pose();
        state.apply(&mut skeleton);
        assert_approx(skeleton.bones[0].x, 10.0);

        let b = state.set_animation(0, "b", false).unwrap();
        b.set_hold_previous(&mut state, true);
        state.update(0.8);
        skeleton.set_to_setup_pose();
        state.apply(&mut skeleton);
        assert_approx(skeleton.bones[0].x, 10.0);
    }
}

#[test]
fn mixing_thresholds_gate_attachment_and_draw_order_from_mixing_from() {
    let anim_a = Animation {
        name: "a".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![AttachmentFrame {
                time: 0.0,
                name: Some("A".to_string()),
            }],
        }],
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: Some(DrawOrderTimeline {
            frames: vec![DrawOrderFrame {
                time: 0.0,
                draw_order_to_setup_index: Some(vec![1, 0]),
            }],
        }),
    };

    // B does not key attachments/draw order, so A can be held via mix thresholds.
    let anim_b = Animation {
        name: "b".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: Vec::new(),
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
    };

    let mut data = base_skeleton_data();
    data.slots = vec![
        SlotData {
            name: "s0".to_string(),
            bone: 0,
            attachment: Some("setup0".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
        },
        SlotData {
            name: "s1".to_string(),
            bone: 0,
            attachment: Some("setup1".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
        },
    ];

    // Attachments must exist in a skin to be applied by attachment timelines or setup poses.
    let mut attachments = vec![HashMap::new(), HashMap::new()];
    attachments[0].insert(
        "setup0".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup0".to_string(),
            path: "setup0.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 1.0,
            height: 1.0,
        }),
    );
    attachments[0].insert(
        "A".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "A".to_string(),
            path: "A.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 1.0,
            height: 1.0,
        }),
    );
    attachments[1].insert(
        "setup1".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup1".to_string(),
            path: "setup1.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 1.0,
            height: 1.0,
        }),
    );
    data.skins.insert(
        "default".to_string(),
        SkinData {
            name: "default".to_string(),
            attachments,
            bones: Vec::new(),
            ik_constraints: Vec::new(),
            transform_constraints: Vec::new(),
            path_constraints: Vec::new(),
            physics_constraints: Vec::new(),
            slider_constraints: Vec::new(),
        },
    );

    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("a", "b", 1.0).unwrap();

    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    let a = state.set_animation(0, "a", false).unwrap();
    a.set_mix_attachment_threshold(&mut state, 0.5);
    a.set_mix_draw_order_threshold(&mut state, 0.5);

    skeleton.set_to_setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("A"));
    assert_eq!(skeleton.draw_order, vec![1, 0]);

    state.set_animation(0, "b", false).unwrap();

    // mix=0.4: mixingFrom(A) still applies attachment/draw order.
    state.update(0.4);
    skeleton.set_to_setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("A"));
    assert_eq!(skeleton.draw_order, vec![1, 0]);

    // mix=0.6: mixingFrom(A) no longer applies attachment/draw order.
    state.update(0.2);
    skeleton.set_to_setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("setup0"));
    assert_eq!(skeleton.draw_order, vec![0, 1]);
}
