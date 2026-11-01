//! New authoring must preserve the published externally tagged legacy inputs.
use super::*;
use worth_query_consumer_values::{PlanarCurrentOutputExpectation, PlanarDerivedOutput};

#[test]
fn scene_authoring_preserves_literal_legacy_planar_operation_wire_shapes() {
    let cases = [
        (
            PlanarOperation::CreateCycle(vec![PlanarVertex {
                body_key: "v".into(),
                x: length(1),
                y: length(2),
            }]),
            r#"{"CreateCycle":[{"body_key":"v","x":1,"y":2}]}"#,
        ),
        (
            PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "v".into(),
                replacement_y: length(3),
            }]),
            r#"{"Adjust":[{"body_key":"v","replacement_y":3}]}"#,
        ),
        (
            PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "v".into(),
                value: length(4),
            }),
            r#"{"PublishDerivedOutput":{"body_key":"v","value":4}}"#,
        ),
        (
            PlanarOperation::RetargetSuccessor {
                source_key: "a".into(),
                previous_target_key: "b".into(),
                replacement_target_key: "c".into(),
            },
            r#"{"RetargetSuccessor":{"source_key":"a","previous_target_key":"b","replacement_target_key":"c"}}"#,
        ),
        (
            PlanarOperation::VerifyCurrentOutputs(vec![PlanarCurrentOutputExpectation {
                producer_key: "a".into(),
                output_key: "b".into(),
            }]),
            r#"{"VerifyCurrentOutputs":[{"producer_key":"a","output_key":"b"}]}"#,
        ),
        (
            PlanarOperation::VerifyFinalCurrentOutputs(vec![]),
            r#"{"VerifyFinalCurrentOutputs":[]}"#,
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(serde_json::to_string(&input).unwrap(), expected);
    }
}
