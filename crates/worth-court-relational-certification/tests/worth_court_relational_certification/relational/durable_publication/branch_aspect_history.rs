//! Public branch and aspect history survive actual Native checkpoint recovery.
use super::branch_history_publications::*;
use super::branch_history_recovery::{checkpoint, current_head, recover};
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    history::{AspectResolutionContext, BranchId},
    runtime::RelationalRuntimeApi,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::changed_entities,
    aspect_storage::{aspect_key, AspectSchemaFixture},
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};

fn audit_runtime(
    medium: &std::sync::Arc<TestDurableMedium>,
) -> worth_relational::facade::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(
            AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::RetainDanglingForAudit,
            )
            .build_registry(),
        )
        .build_durable(medium.attachment())
        .unwrap()
}
#[test]
fn durability_contract_recovery_rebuilds_branch_heads_and_latest_commit() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let main = create_entity_outcome(execution, &runtime, "main-a");
    fork(execution, &runtime, "feature");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-a",
        BranchId("feature".to_string()),
    );
    checkpoint(execution, &runtime, &medium);
    let (outcome, recovered) = recover(execution, &medium, durable_runtime(&medium));

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.latest_commit, Some(feature.commit.clone()));
    assert_eq!(
        current_head(&recovered, &BranchId("feature".to_string())),
        Some(feature.commit.clone())
    );
    assert_eq!(
        current_head(&recovered, &BranchId("main".to_string())),
        Some(main.commit.clone())
    );
}

#[test]
fn durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(execution, &runtime, entity, "after");
    let expected_history = runtime
        .history()
        .entity_aspect_history(execution, &BranchId("main".to_string()), entity, None)
        .unwrap();
    let expected_digest = runtime
        .history()
        .entity_aspect_history_with_trace(execution, &BranchId("main".to_string()), entity, None)
        .unwrap()
        .aspect_history_digest();
    let expected_envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, updated.commit.commit_id)
        .unwrap()
        .unwrap();
    checkpoint(execution, &runtime, &medium);
    let (outcome, recovered) = recover(execution, &medium, durable_runtime(&medium));

    let recovered_history = recovered
        .history()
        .entity_aspect_history(execution, &BranchId("main".to_string()), entity, None)
        .unwrap();
    let recovered_digest = recovered
        .history()
        .entity_aspect_history_with_trace(execution, &BranchId("main".to_string()), entity, None)
        .unwrap()
        .aspect_history_digest();
    let recovered_replay = recovered.replay();
    let recovered_envelope = recovered_replay
        .canonical_commit_envelope(execution, updated.commit.commit_id)
        .unwrap()
        .unwrap();

    assert_eq!(outcome.latest_commit, Some(updated.commit.clone()));
    assert_eq!(expected_history, recovered_history);
    assert_eq!(expected_digest, recovered_digest);
    assert_eq!(
        expected_envelope.patch.authoritative_record_patches,
        recovered_envelope.patch.authoritative_record_patches
    );
    assert_eq!(
        recovered_envelope.patch.authoritative_record_patches[0].authoritative_changed_aspects(),
        vec![aspect_key("name")]
    );
    assert!(!recovered_envelope.patch.authoritative_record_patches[0].contains_opaque_aspect);
}

#[test]
fn durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = audit_runtime(&medium);
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome = create_relation_outcome(execution, &runtime, source, target, "r-audit");
    let relation = changed_relations(&relation_outcome)[0];
    let deleted = delete_entity(execution, &runtime, source);
    let expected_history = runtime
        .history()
        .relation_aspect_history(execution, &BranchId("main".to_string()), relation, None)
        .unwrap();
    let expected_digest = runtime
        .history()
        .relation_aspect_history_with_trace(
            execution,
            &BranchId("main".to_string()),
            relation,
            None,
        )
        .unwrap()
        .aspect_history_digest();
    checkpoint(execution, &runtime, &medium);
    let (outcome, recovered) = recover(execution, &medium, audit_runtime(&medium));

    let recovered_history = recovered
        .history()
        .relation_aspect_history(execution, &BranchId("main".to_string()), relation, None)
        .unwrap();
    let recovered_digest = recovered
        .history()
        .relation_aspect_history_with_trace(
            execution,
            &BranchId("main".to_string()),
            relation,
            None,
        )
        .unwrap()
        .aspect_history_digest();

    assert_eq!(outcome.latest_commit, Some(deleted.commit.clone()));
    assert_eq!(expected_history, recovered_history);
    assert_eq!(expected_digest, recovered_digest);
    assert_eq!(recovered_history.len(), 2);
    assert!(!recovered_history.is_empty());
    assert!(recovered_history
        .iter()
        .all(|entry| entry.origin.target == RecordRef::Relation(relation)));
    assert!(recovered_history.iter().all(|entry| matches!(
        entry.resolution,
        AspectResolutionContext::DirectRecordHistory
    )));
    assert_eq!(
        recovered_history[0].origin.changed_aspects,
        vec![
            aspect_key("label"),
            aspect_key("lifecycle"),
            aspect_key("source"),
            aspect_key("target")
        ]
    );
    assert_eq!(
        recovered_history[1].origin.changed_aspects,
        vec![aspect_key("lifecycle")]
    );
}
