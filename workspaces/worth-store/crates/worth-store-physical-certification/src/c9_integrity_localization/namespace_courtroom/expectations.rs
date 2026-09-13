use super::corruption::Operator;
use serde_json::{json, Value};

pub(super) fn require(operator: Operator, store: &str, runtime: &Value, offline: &Value) {
    let row = offline["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == "namespace/identity")
        .unwrap();
    assert_eq!(row["family"], "namespace_identity");
    assert_eq!(row["range"], json!({"offset": 0, "length": 72}));
    let outcome = &row["outcome"];
    if matches!(operator, Operator::Clean) {
        assert_eq!(runtime["admission"], "admitted");
        assert_eq!(runtime["store"], store);
        assert_eq!(runtime["recovery_effects"], 0);
        assert_eq!(offline["store"], store);
        assert_eq!(outcome["posture"], "intact");
        assert_eq!(offline["consumed"]["namespace_identity_decoders"], 1);
        return;
    }
    assert_eq!(runtime["admission"], "denied");
    assert_eq!(runtime["stage"], "qualify_existing");
    assert_eq!(runtime["cause"], "ExistingStoreRequired");
    assert!(runtime["store"].is_null());
    assert!(offline["store"].is_null());
    assert_eq!(offline["consumed"]["namespace_identity_decoders"], 0);
    if matches!(operator, Operator::Encoding | Operator::Schema) {
        assert_eq!(outcome["posture"], "unsupported");
        assert_eq!(outcome["observed"], 2);
        let (axis, offset) = if matches!(operator, Operator::Encoding) {
            ("namespace_encoding", 8)
        } else {
            ("namespace_schema", 10)
        };
        assert_eq!(outcome["axis"], axis);
        assert_eq!(outcome["range"], json!({"offset": offset, "length": 2}));
        return;
    }
    assert_eq!(outcome["posture"], "damaged");
    let (cause, range, field, blast) = match operator {
        Operator::CoveredByte | Operator::ChecksumByte => (
            "checksum_mismatch",
            json!({"offset": 0, "length": 72}),
            json!("checksum"),
            "artifact",
        ),
        Operator::Truncate => (
            "truncation",
            json!({"offset": 71, "length": 1}),
            json!("record_length"),
            "field",
        ),
        Operator::Remove => ("missing_artifact", Value::Null, Value::Null, "artifact"),
        _ => unreachable!(),
    };
    assert_eq!(outcome["cause"], cause);
    assert_eq!(outcome["damaged_range"], range);
    assert_eq!(outcome["field"], field);
    assert_eq!(outcome["blast_radius"], blast);
}
