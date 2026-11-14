//! The exact comparison consumes descriptive data, never issues custody.
use super::*;
use crate::mvcc::{RequiredOutputProducingImplementation, RequiredOutputPublicationMeaning};
use crate::tests::support::{
    create_entity, runtime_with_test_schema, test_owner_transaction_validation_input_for_main,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};

#[test]
fn corrective_owner_union_keeps_missing_extra_and_wrong_predecessor_refusals() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let source = create_entity(&execution, &runtime, "descriptive-owner");
    let original = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .last()
        .unwrap()
        .canonical_arc()
        .clone();
    let admission = test_owner_transaction_validation_input_for_main(&runtime);
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint([1; 16], [2; 32])
            .unwrap(),
        family: "owner-selection-data".into(),
        source_scope: source,
        source_partition: worth_foundational::CanonicalDigestId::new([3; 32]),
        source_epoch: worth_foundational::CanonicalDigestId::new([4; 32]),
    };
    // No installed issuer is fabricated. These are comparison inputs only;
    // complete publication/header/readmission still have their other gates.
    let value = serde_json::json!({
        "commit": original.commit, "basis": admission.basis().descriptor(),
        "producer": "owner-selection-data", "source_scope_owned": true,
        "dependency_identity": worth_foundational::CanonicalDigestId::new([5;32]),
        "outputs": [], "relations": [], "meaning": meaning, "publication": "Birth",
        "birth": original.commit.commit_id, "birth_producer": "owner-selection-data",
        "birth_dependency": worth_foundational::CanonicalDigestId::new([6;32]), "members": []
    });
    let expected = SelectedOutputPublication {
        evidence: crate::publication::ownership::PublicationEvidence::Canonical(original.clone()),
        origin: Arc::new(serde_json::from_value(value.clone()).unwrap()),
        ordinal: 0,
    };
    let mut successor = value.clone();
    successor["publication"] =
        serde_json::json!({"Preserve": {"predecessor": original.commit.commit_id}});
    let correct: CanonicalRequiredOutputOrigin = serde_json::from_value(successor.clone()).unwrap();
    let mut wrong = successor.clone();
    wrong["publication"] = serde_json::json!({"Preserve": {"predecessor": crate::history::data::CommitId(original.commit.commit_id.0 + 1)}});
    let wrong: CanonicalRequiredOutputOrigin = serde_json::from_value(wrong).unwrap();
    let mut extra = successor;
    extra["meaning"]["family"] = serde_json::json!("another-owner");
    let extra: CanonicalRequiredOutputOrigin = serde_json::from_value(extra).unwrap();
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    exact(
                        std::slice::from_ref(&expected),
                        std::iter::once(&correct),
                        true,
                        context,
                    )
                    .unwrap();
                    assert!(exact(
                        std::slice::from_ref(&expected),
                        std::iter::empty(),
                        true,
                        context
                    )
                    .is_err());
                    assert!(exact(
                        std::slice::from_ref(&expected),
                        [&correct, &extra].into_iter(),
                        true,
                        context
                    )
                    .is_err());
                    assert!(exact(
                        std::slice::from_ref(&expected),
                        std::iter::once(&wrong),
                        true,
                        context
                    )
                    .is_err());
                },
            )
        })
        .unwrap()
        .unwrap();
}
