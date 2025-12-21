use crate::{
    AttachmentFrame, AttachmentTimeline, MixBlend, Skeleton, SkeletonData, apply_animation,
    apply_attachment, build_draw_list,
};

const SKELETON_UNWEIGHTED: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {
    "d": {
      "attachments": {
        "default": {
          "slot0": {
            "mesh0": {
              "deform": [
                { "time": 0, "offset": 0, "vertices": [1,0, 0,0, 0,0, 0,0] }
              ]
            }
          }
        }
      }
    }
  }
}
"#;

const SKELETON_WEIGHTED: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [
            1, 0, -1, -1, 1,
            1, 0,  1, -1, 1,
            1, 0,  1,  1, 1,
            1, 0, -1,  1, 1
          ],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {
    "d": {
      "attachments": {
        "default": {
          "slot0": {
            "mesh0": {
              "deform": [
                { "time": 0, "offset": 0, "vertices": [1,0, 0,0, 0,0, 0,0] }
              ]
            }
          }
        }
      }
    }
  }
}
"#;

const SKELETON_LINKEDMESH_PARENT_DEFAULT_SKIN_DEFORM: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "child" } ],
  "skins": {
    "default": {
      "slot0": {
        "parent": {
          "type": "mesh",
          "path": "parent",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    },
    "skinA": {
      "slot0": {
        "child": {
          "type": "linkedmesh",
          "parent": "parent",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {
    "d": {
      "attachments": {
        "default": {
          "slot0": {
            "parent": {
              "deform": [
                { "time": 0, "offset": 0, "vertices": [1,0, 0,0, 0,0, 0,0] }
              ]
            }
          }
        }
      }
    }
  }
}
"#;

fn assert_approx2(actual: [f32; 2], expected: [f32; 2]) {
    let dx = (actual[0] - expected[0]).abs();
    let dy = (actual[1] - expected[1]).abs();
    assert!(
        dx <= 1.0e-6 && dy <= 1.0e-6,
        "expected {expected:?}, got {actual:?} (dx {dx}, dy {dy})"
    );
}

#[test]
fn deform_timeline_unweighted_is_applied_to_slot_and_rendered() {
    let data = SkeletonData::from_json_str(SKELETON_UNWEIGHTED).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("d").unwrap();
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    assert_eq!(skeleton.slots[0].deform.len(), 8);
    assert_approx2(
        [skeleton.slots[0].deform[0], skeleton.slots[0].deform[1]],
        [0.0, -1.0],
    );

    let draw_list = build_draw_list(&skeleton);
    assert!(!draw_list.vertices.is_empty());
    assert_approx2(draw_list.vertices[0].position, [0.0, -1.0]);
}

#[test]
fn deform_timeline_weighted_is_applied_as_offsets() {
    let data = SkeletonData::from_json_str(SKELETON_WEIGHTED).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("d").unwrap();
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    assert_eq!(skeleton.slots[0].deform.len(), 8);
    assert_approx2(
        [skeleton.slots[0].deform[0], skeleton.slots[0].deform[1]],
        [1.0, 0.0],
    );

    let draw_list = build_draw_list(&skeleton);
    assert!(!draw_list.vertices.is_empty());
    assert_approx2(draw_list.vertices[0].position, [0.0, -1.0]);
}

#[test]
fn deform_timeline_applies_to_linked_mesh_inheriting_parent_deform_from_default_skin() {
    let data = SkeletonData::from_json_str(SKELETON_LINKEDMESH_PARENT_DEFAULT_SKIN_DEFORM).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("d").unwrap();

    skeleton.set_skin(Some("skinA")).unwrap();
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    assert_eq!(skeleton.slots[0].deform.len(), 8);
    assert_approx2(
        [skeleton.slots[0].deform[0], skeleton.slots[0].deform[1]],
        [0.0, -1.0],
    );

    let draw_list = build_draw_list(&skeleton);
    assert!(!draw_list.vertices.is_empty());
    assert_approx2(draw_list.vertices[0].position, [0.0, -1.0]);
}

#[test]
fn attachment_switch_between_linked_mesh_and_parent_preserves_deform_when_timeline_attachment_matches()
 {
    let data = SkeletonData::from_json_str(SKELETON_LINKEDMESH_PARENT_DEFAULT_SKIN_DEFORM).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("d").unwrap();

    skeleton.set_skin(Some("skinA")).unwrap();
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    assert_eq!(skeleton.slots[0].deform.len(), 8);

    let switch = AttachmentTimeline {
        slot_index: 0,
        frames: vec![AttachmentFrame {
            time: 0.0,
            name: Some("parent".to_string()),
        }],
    };
    apply_attachment(&switch, &mut skeleton, 0.0, MixBlend::Replace, true, 0);

    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("parent"));
    assert_eq!(skeleton.slots[0].deform.len(), 8);
}
