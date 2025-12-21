use crate::SkeletonData;

#[test]
fn json_physics_constraint_defaults_match_spine_cpp() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [],
  "constraints": [
    { "name": "p0", "type": "physics", "bone": "root" }
  ]
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let c = data.physics_constraints.get(0).expect("physics constraint");

    assert_eq!(c.inertia, 0.5);
    assert_eq!(c.strength, 100.0);
    assert_eq!(c.damping, 0.85);
    assert_eq!(c.mass_inverse, 1.0);
    assert_eq!(c.wind, 0.0);
    assert_eq!(c.gravity, 0.0);
    assert_eq!(c.mix, 1.0);
    assert_eq!(c.limit, 5000.0);
    assert!((c.step - (1.0 / 60.0)).abs() <= 1.0e-6);
}
