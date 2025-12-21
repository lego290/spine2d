use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData, build_draw_list};

#[test]
fn sequence_timeline_drives_slot_sequence_index_and_render_path() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "wing" } ],
  "skins": {
    "default": {
      "slot0": {
        "wing": {
          "type": "region",
          "path": "wing",
          "sequence": { "count": 3, "start": 1, "digits": 2, "setupIndex": 1 },
          "width": 2,
          "height": 2
        }
      }
    }
  },
  "animations": {
    "fly": {
      "attachments": {
        "default": {
          "slot0": {
            "wing": {
              "sequence": [
                { "time": 0, "mode": "loop", "index": 0, "delay": 0.1 },
                { "time": 1, "mode": "loop", "index": 0, "delay": 0.1 }
              ]
            }
          }
        }
      }
    }
  }
}
"#,
    )
    .unwrap();

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_to_setup_pose();

    assert_eq!(skeleton.slots[0].sequence_index, -1);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "wing02");

    let state_data = AnimationStateData::new(data);
    let mut state = AnimationState::new(state_data);
    state.set_animation(0, "fly", true).unwrap();

    state.update(0.0);
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].sequence_index, 0);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws[0].texture_path, "wing01");

    state.update(0.15);
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].sequence_index, 1);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws[0].texture_path, "wing02");
}
