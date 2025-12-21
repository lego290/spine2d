#[cfg(feature = "upstream-smoke")]
mod tests {
    use crate::{SkeletonData, TransformProperty};
    use std::path::PathBuf;

    fn spineboy_json_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json")
    }

    #[test]
    fn spineboy_transform_constraint_properties_are_parsed() {
        let path = spineboy_json_path();
        let json = std::fs::read_to_string(&path).expect("read spineboy json");
        let data = SkeletonData::from_json_str(&json).expect("parse spineboy json");

        let c = data
            .transform_constraints
            .iter()
            .find(|c| c.name == "shoulder")
            .expect("missing transform constraint 'shoulder'");

        assert_eq!(c.bones.len(), 1);
        assert_eq!(c.mix_x, -1.0);
        assert_eq!(c.mix_y, -1.0);

        let from_props: Vec<_> = c.properties.iter().map(|p| p.property).collect();
        assert!(from_props.contains(&TransformProperty::X));
        assert!(from_props.contains(&TransformProperty::Y));

        let from_x = c
            .properties
            .iter()
            .find(|p| matches!(p.property, TransformProperty::X))
            .expect("missing From X");
        assert!(
            from_x
                .to
                .iter()
                .any(|t| matches!(t.property, TransformProperty::X))
        );

        let from_y = c
            .properties
            .iter()
            .find(|p| matches!(p.property, TransformProperty::Y))
            .expect("missing From Y");
        assert!(
            from_y
                .to
                .iter()
                .any(|t| matches!(t.property, TransformProperty::Y))
        );
    }
}
