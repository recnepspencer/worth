use serde_json::{json, Value};

use super::artifact_manifest::PendingArtifact;
use super::corruption::Operator;

pub(super) fn require(
    artifact: &PendingArtifact,
    operator: Operator,
    runtime: &Value,
    offline: &Value,
) {
    let target = if matches!(operator, Operator::Duplicate) {
        artifact.duplicate_path()
    } else {
        artifact.path.clone()
    };
    let path = target.to_str().unwrap().replace('\\', "/");
    let runtime_rows = runtime["observations"].as_array().unwrap();
    let live = runtime_rows
        .iter()
        .find(|row| row["path"] == path)
        .expect("runtime expected canonical PW row");
    let independent = offline["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == path && row["family"] == "physical_work_obligation")
        .expect("offline expected canonical PW row");
    assert_eq!(
        live["scope"]["byte_range"],
        json!({"offset": 0, "length": 160})
    );
    assert_eq!(live["scope"]["store_identity"], json!(artifact.store));
    let expected_operation =
        artifact.operation + u64::from(matches!(operator, Operator::Duplicate));
    assert_eq!(
        live["scope"]["identity"]["PhysicalWork"],
        json!({
            "runtime": artifact.runtime, "generation": artifact.generation, "operation": expected_operation,
        })
    );
    assert_eq!(live["scope"]["family"], "PhysicalWorkObligation");
    assert_eq!(independent["generation"], artifact.generation);
    assert_eq!(
        independent["identity"],
        format!(
            "operation:{:016x}:{:016x}:{:016x}",
            artifact.runtime, artifact.generation, expected_operation
        )
    );
    assert_eq!(independent["range"], json!({"offset": 0, "length": 160}));
    let intact = matches!(operator, Operator::Clean | Operator::Duplicate);
    let count = if matches!(operator, Operator::Duplicate) {
        2
    } else {
        1
    };
    assert_eq!(runtime["counters"]["attempted"], count);
    assert_eq!(runtime["counters"]["admitted"], u64::from(intact));
    assert_eq!(runtime["counters"]["owner_entries"], u64::from(intact));
    assert_eq!(runtime["counters"]["rejected"], count - u64::from(intact));
    if intact {
        assert_eq!(
            runtime["obligation_dispositions"],
            json!(["InspectionRequired"])
        );
    }
    if matches!(operator, Operator::Clean) {
        assert_eq!(runtime["evidence_damaged"], false);
        assert_eq!(live["outcome"], "Admitted");
        assert_eq!(independent["outcome"]["posture"], "intact");
        return;
    }
    assert_eq!(runtime["evidence_damaged"], true);
    if matches!(operator, Operator::Unsupported) {
        assert_eq!(
            live["outcome"]["Unsupported"]["axis"],
            "PhysicalWorkObligation"
        );
        assert_eq!(live["outcome"]["Unsupported"]["observed"], 7);
        assert_eq!(independent["outcome"]["posture"], "unsupported");
        assert_eq!(independent["outcome"]["observed"], 7);
        assert_eq!(independent["outcome"]["axis"], "physical_work");
        assert_eq!(
            independent["outcome"]["range"],
            json!({"offset": 8, "length": 1})
        );
        return;
    }
    // These expectations come from the producer bytes and the declared edit,
    // never a validator, decoded runtime artifact, or the peer's outcome.
    let (cause, runtime_range, offline_cause, offline_range) = match operator {
        Operator::CoveredByte | Operator::ChecksumByte => {
            ("ChecksumMismatch", (0, 160), "checksum_mismatch", (0, 128))
        }
        Operator::WrongStore => (
            "StoreIdentityMismatch",
            (16, 16),
            "scope_mismatch",
            (16, 16),
        ),
        Operator::Truncate => ("Truncated", (159, 1), "truncation", (159, 1)),
        Operator::Duplicate => (
            "ArtifactIdentityMismatch",
            (48, 8),
            "scope_mismatch",
            (32, 24),
        ),
        _ => unreachable!(),
    };
    let damage = &live["outcome"]["Damaged"];
    assert_eq!(damage["cause"], cause);
    assert_eq!(damage["scope"], live["scope"]);
    let identity_edit = matches!(operator, Operator::WrongStore | Operator::Duplicate);
    assert_eq!(
        damage["blast_radius"],
        if identity_edit {
            "CompleteArtifact"
        } else {
            "CanonicalFrame"
        }
    );
    let (runtime_field, offline_field) = match operator {
        Operator::WrongStore => (json!("StoreIdentity"), json!("store_identity")),
        Operator::Duplicate => (json!("OperationIdentity"), json!("identity_field")),
        _ => (Value::Null, Value::Null),
    };
    assert_eq!(damage["field"], runtime_field);
    assert_eq!(independent["outcome"]["field"], offline_field);
    assert_eq!(
        independent["outcome"]["blast_radius"],
        if identity_edit { "field" } else { "artifact" }
    );
    assert_eq!(
        damage["damaged_range"],
        json!({"offset": runtime_range.0, "length": runtime_range.1})
    );
    assert_eq!(independent["outcome"]["posture"], "damaged");
    assert_eq!(independent["outcome"]["cause"], offline_cause);
    assert_eq!(
        independent["outcome"]["damaged_range"],
        json!({"offset": offline_range.0, "length": offline_range.1})
    );
}
