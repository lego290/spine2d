use crate::{
    BoneData, BoneTimeline, Curve, Inherit, MixBlend, RotateFrame, RotateTimeline, ScaleTimeline,
    ShearTimeline, Skeleton, SkeletonData, TranslateTimeline, Vec2Frame, apply_animation,
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

fn skeleton_data_with_root(setup_x: f32, setup_y: f32, setup_rotation: f32) -> Arc<SkeletonData> {
    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            x: setup_x,
            y: setup_y,
            rotation: setup_rotation,
            scale_x: 2.0,
            scale_y: 4.0,
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
    })
}

#[test]
fn translate_timeline_interpolates() {
    let data = skeleton_data_with_root(2.0, 3.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 7.0);
    assert_approx(skeleton.bones[0].y, 3.0);
}

#[test]
fn rotate_timeline_interpolates_linearly() {
    let data = skeleton_data_with_root(0.0, 0.0, 20.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![
                RotateFrame {
                    time: 0.0,
                    angle: 170.0,
                    curve: Curve::Linear,
                },
                RotateFrame {
                    time: 1.0,
                    angle: -170.0,
                    curve: Curve::Linear,
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].rotation, 20.0);
}

#[test]
fn rotate_timeline_mixes_shortest_path() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].rotation = 170.0;

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![RotateFrame {
                time: 0.0,
                angle: -170.0,
                curve: Curve::Linear,
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.0,
        false,
        0.5,
        MixBlend::Replace,
    );

    // Upstream RotateTimeline does not normalize the final rotation, it uses CurveTimeline1
    // relative blending (see spine-cpp `RotateTimeline::apply`).
    assert_approx(skeleton.bones[0].rotation, 0.0);
}

#[test]
fn looping_wraps_time_by_duration() {
    let data = skeleton_data_with_root(1.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        1.25,
        true,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 3.5);
}

#[test]
fn setup_blend_uses_setup_as_base() {
    let data = skeleton_data_with_root(2.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].x = 100.0;

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
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

    apply_animation(&animation, &mut skeleton, 0.0, false, 0.5, MixBlend::Setup);
    assert_approx(skeleton.bones[0].x, 7.0);
}

#[test]
fn scale_timeline_applies() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Scale(ScaleTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 1.0,
                    y: 1.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 3.0,
                    y: 5.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.25,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].scale_x, 3.0);
    assert_approx(skeleton.bones[0].scale_y, 8.0);
}

#[test]
fn stepped_curve_holds_previous_value() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Stepped; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 0.0);
}

#[test]
fn shear_timeline_applies_to_bone() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Shear(ShearTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 20.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].shear_x, 5.0);
    assert_approx(skeleton.bones[0].shear_y, 10.0);
}

#[test]
fn bezier_curve_interpolates_in_value_space() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    // When x control points are at 1/3 and 2/3 of the segment, x(t) becomes linear.
    // With value1=0, value2=10 and cy1=cy2=0, then y(t)=10*t^3, so y(0.5)=1.25.
    let animation = crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [
                        Curve::Bezier {
                            cx1: 1.0 / 3.0,
                            cy1: 0.0,
                            cx2: 2.0 / 3.0,
                            cy2: 0.0,
                        },
                        Curve::Linear,
                    ],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
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

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 1.25);
}
