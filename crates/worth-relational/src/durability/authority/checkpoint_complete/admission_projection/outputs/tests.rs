//! Pure untrusted wire values exercise the projection, without minting authority.
use super::{CanonicalRequiredOutputCodecRetirement, Retirement};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use serde_json::Value;
use std::sync::Arc;
use worth_foundational::facade::*;

#[test]
fn codec_retirement_projection_keeps_meaning_and_omits_runtime_identity_slots() {
    // Deliberately runtime-affine DATA. This value is not a retirement proof or
    // an admitted publication; only its codec projection is under test.
    let source = serde_json::from_value(serde_json::json!({
        "descriptor_version": 2, "runtime_instance_id": 99, "branch_id": "main",
        "reference": { "branch_id": "relational/99/main", "target": "Empty", "generation": 4 },
        "truth_version": 7, "root_identity": 123,
        "schema_commitment": CanonicalDigestId::new([3; 32]), "visibility_commitment": CanonicalDigestId::new([4; 32]),
        "posture": "Live"
    }))
    .unwrap();
    let value = CanonicalRequiredOutputCodecRetirement {
        publication: CommitId(11),
        meaning: crate::mvcc::RequiredOutputPublicationMeaning {
            implementation: crate::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
                [1; 16], [2; 32],
            )
            .unwrap(),
            family: "codec-family".into(),
            source_scope: EntityId::new(PartitionId::main(), 0, 1),
            source_partition: worth_foundational::CanonicalDigestId::new([5; 32]),
            source_epoch: worth_foundational::CanonicalDigestId::new([6; 32]),
        },
        source,
        retiring: RelationalCommitReceipt {
            commit_id: CommitId(12),
            version_id: VersionId(13),
            branch_id: BranchId("main".into()),
            parents: vec![CommitId(11)],
        },
        kinds: Arc::new(Vec::new()),
        activation: Arc::new(crate::runtime::RecoveredCheckpointProgramActivationUpdate {
            entity_id: EntityId::new(PartitionId::main(), 1, 1),
            field_locator: AspectFieldLocator::new(
                LocatorAuthority::Planned,
                AspectKey::new("activation").unwrap(),
                CanonicalFieldPath::single(FieldKey::new("value").unwrap()),
            ),
            expected: AspectValue::String(InternedString::from("before")),
            replacement: AspectValue::String(InternedString::from("after")),
        }),
        _funding: None,
    };
    let projected = serde_json::to_value(Retirement(&value)).unwrap();
    assert_no_runtime_slots(&projected);
    assert_eq!(
        projected["source"]["reference"]["observation"]["branch"],
        "main"
    );
    assert_eq!(
        projected["source"]["reference"]["observation"]["generation"],
        4
    );
    assert_eq!(projected["source"]["reference"]["truth_version"], 7);
    let original = serde_json::to_value(&value).unwrap();
    for field in ["publication", "meaning", "retiring", "kinds", "activation"] {
        assert_eq!(projected[field], original[field]);
    }
}
fn assert_no_runtime_slots(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (name, value) in fields {
                assert_ne!(name, "runtime_instance_id");
                assert_ne!(name, "root_identity");
                assert_no_runtime_slots(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                assert_no_runtime_slots(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
