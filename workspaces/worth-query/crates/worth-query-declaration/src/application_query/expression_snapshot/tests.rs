//! Data selection follows nominal schema meaning, not native generated record names or positions.
use super::*;
use worth_foundational::expression_api::ExpressionTypeName;
fn schema(reverse: bool) -> ExpressionSchema {
    let fields = if reverse {
        vec![
            ("noise", ExpressionType::String),
            ("name", ExpressionType::String),
        ]
    } else {
        vec![
            ("name", ExpressionType::String),
            ("noise", ExpressionType::String),
        ]
    };
    let person =
        ExpressionType::Record(ExpressionTypeName::new("DifferentNominalName", 1).unwrap());
    ExpressionSchema::builder()
        .record("DifferentNominalName", 1, fields)
        .unwrap()
        .record(
            "QueryContext::root",
            1,
            [
                ("owner", ExpressionType::option(person.clone())),
                ("people", ExpressionType::list(person)),
                ("comment", ExpressionType::option(ExpressionType::String)),
            ],
        )
        .unwrap()
        .build()
}
#[test]
fn nested_selection_tracks_nominal_types_and_changed_field_order_not_first_list_members() {
    for reverse in [false, true] {
        let mut values = vec![
            ExpressionValue::string("actual name"),
            ExpressionValue::string("unrelated"),
        ];
        if reverse {
            values.reverse();
        }
        let person = ExpressionValue::record(values);
        let snapshot = ApplicationQueryExpressionSnapshot::from_untrusted_parts(
            schema(reverse),
            ExpressionValue::record(vec![
                ExpressionValue::some(person.clone()),
                ExpressionValue::list(vec![person]),
                ExpressionValue::none(),
            ]),
        );
        assert_eq!(
            snapshot
                .field_path(&["owner", "name"])
                .and_then(ExpressionValue::as_str),
            Some("actual name")
        );
        assert!(snapshot.field_path(&["people", "name"]).is_none());
        assert!(snapshot.field_path(&["owner", "unknown"]).is_none());
        assert!(snapshot.field_path(&[]).is_none());
        assert_eq!(
            snapshot
                .field("comment")
                .and_then(ExpressionValue::as_option),
            Some(None)
        );
    }
}
#[test]
fn absent_optional_intermediate_and_incomplete_untrusted_values_are_not_defaulted() {
    for value in [
        ExpressionValue::record(vec![ExpressionValue::none()]),
        ExpressionValue::record(vec![]),
        ExpressionValue::string("not a record"),
    ] {
        let snapshot =
            ApplicationQueryExpressionSnapshot::from_untrusted_parts(schema(false), value);
        assert!(snapshot.field_path(&["owner", "name"]).is_none());
    }
    let person_list = ExpressionValue::list(vec![
        ExpressionValue::string("counterfeit"),
        ExpressionValue::string("noise"),
    ]);
    let nested = ApplicationQueryExpressionSnapshot::from_untrusted_parts(
        schema(false),
        ExpressionValue::record(vec![ExpressionValue::some(person_list.clone())]),
    );
    assert!(nested.field_path(&["owner", "name"]).is_none());
    let root = ApplicationQueryExpressionSnapshot::from_untrusted_parts(
        schema(false),
        ExpressionValue::list(vec![ExpressionValue::some(person_list)]),
    );
    assert!(root.field("owner").is_none());
    assert!(root.field_path(&["owner", "name"]).is_none());
}
