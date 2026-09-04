#[test]
fn test() {
    assert_eq!(
        super::parse("<test>", "M123 X=9:10 Y=11 S13=ON"),
        Some(crate::format::output::Condition {
            map: Some(123),
            map_x1: 9,
            map_x2: 10,
            map_y1: 11,
            switch_id: Some(13),
            switch_value: true,
            trigger: None,
            ..Default::default()
        })
    );
}
