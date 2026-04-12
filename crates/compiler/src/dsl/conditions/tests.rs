#[test]
fn test() {
    assert_eq!(
        super::parse("M123 X=9:10 Y=11 S13=ON"),
        crate::format::output::Condition {
            map: Some(123),
            map_x1: Some(9),
            map_x2: Some(10),
            map_y1: Some(11),
            switch_id: Some(13),
            switch_value: true,
            ..Default::default()
        }
    );
}
