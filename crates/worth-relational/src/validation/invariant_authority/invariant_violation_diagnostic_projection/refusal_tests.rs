//! Retention projection preserves the original canonical carrier as bytes.
use super::{InvariantViolationDiagnosticProjection, RelationalDiagnosticValue};
use crate::validation::data::{
    CustomInvariantRefusal, CustomInvariantRuleId, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, InvariantViolationFields,
};

#[test]
fn retained_violation_diagnostics_keep_canonical_refusal_bytes() {
    let bytes = br#"{"rule":1,"observed":"closed"}"#.to_vec();
    let fields = InvariantViolationFields::CustomInvariantViolation {
        identity: CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new("tests.retained-refusal"),
            semantic_version: CustomInvariantSemanticVersion::new(1, 0),
        },
        refusal: CustomInvariantRefusal::DeclaredValue {
            value_identity: "tests.refusal.v1".into(),
            bytes: bytes.clone(),
        },
    };
    let encoded = serde_json::to_vec(&fields).unwrap();
    let retained: InvariantViolationFields = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(retained, fields);
    assert_eq!(serde_json::to_vec(&retained).unwrap(), encoded);
    let RelationalDiagnosticValue::Object(projected) =
        InvariantViolationDiagnosticProjection::from_fields(&retained).to_diagnostic_value()
    else {
        panic!("violation diagnostic object");
    };
    assert_eq!(
        projected.get("refusal"),
        Some(&RelationalDiagnosticValue::object([
            (
                "value_identity",
                RelationalDiagnosticValue::string("tests.refusal.v1")
            ),
            ("bytes", RelationalDiagnosticValue::CanonicalBytes(bytes)),
        ]))
    );
}
