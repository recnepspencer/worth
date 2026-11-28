use std::sync::Arc;

use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};

use crate::facade::{
    RelationalBridgeRecordIdentityParts, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadRequest, TruthSnapshotReader,
};
use crate::relational_source::relational_test_support::{
    changed_relations, create_entity, create_entity_outcome, create_relation_outcome,
    runtime_with_declared_aspect_schema,
};
use worth_relational::facade::config::CascadeDeletePolicy;

use super::bridge_snapshot_identity_for_handle;
use crate::relational_source::snapshot_reading::RuntimePublicationSnapshotReader;

#[test]
fn snapshot_reader_reads_published_entity_values_without_projection_surface() {
    let host_request = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let host_request = crate::snapshot::test_serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let published_entity = runtime
        .read_truth()
        .read_snapshot(&published_snapshot, request_execution)
        .expect("fixture read request remains live")
        .expect("published snapshot read")
        .entities()
        .first()
        .expect("published entity record")
        .entity_id;

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            published_entity.partition_id.0,
            published_entity.local_slot.0,
            published_entity.generation.0,
        ),
        scalar_string_contract("name"),
    )]);

    let result = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect("published packet read");

    assert_eq!(result.snapshot_identity(), &snapshot_identity);
    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("visible".into()))
    );
}

#[test]
fn snapshot_reader_reads_published_relation_field_aspects_from_authoritative_state() {
    let host_request = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let created_relation =
        create_relation_outcome(execution, &runtime, source, target, "visible-edge");
    let relation = changed_relations(&created_relation)[0];
    let published_snapshot = created_relation.snapshot.clone();

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::relation(
            relation.partition_id.0,
            relation.local_slot.0,
            relation.generation.0,
        ),
        scalar_string_contract("label"),
    )]);

    let result = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect("published relation aspect packet read");

    assert_eq!(result.snapshot_identity(), &snapshot_identity);
    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("visible-edge".into()))
    );
}

#[test]
fn snapshot_reader_rejects_undeclared_dotted_document_paths() {
    let host_request = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let published_entity =
        crate::relational_source::relational_test_support::changed_entities(&created)[0];

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            published_entity.partition_id.0,
            published_entity.local_slot.0,
            published_entity.generation.0,
        ),
        scalar_string_contract("profile.name"),
    )]);

    let error = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect_err("flat aspect path should not satisfy dotted aspect path");

    assert!(
        error
            .to_string()
            .contains("could not resolve aspect `profile.name`"),
        "unexpected bridge snapshot error: {error}"
    );
}

#[test]
fn snapshot_reader_rejects_untyped_bridge_record_identity() {
    let host_request = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "visible");
    let published_snapshot = created.snapshot.clone();

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(runtime, snapshot_identity, published_snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        // allowed-untyped-negative-test
        "legacy-row",
        scalar_string_contract("name"),
    )]);

    let error = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect_err("relational snapshot reads require typed record identity parts");

    assert!(
        error
            .to_string()
            .contains("requires typed record identity parts"),
        "unexpected bridge snapshot error: {error}"
    );
}

#[test]
fn snapshot_reader_reports_missing_record_as_authoritative_absence() {
    let host_request = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(runtime, snapshot_identity, published_snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(1, 999, 1),
        scalar_string_contract("name"),
    )]);

    let result = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect("absence is authoritative");

    assert_eq!(result.records().len(), 1);
    assert!(result.records()[0].is_absent());
}

fn runtime_with_test_schema() -> worth_relational::facade::runtime::RelationalRuntime {
    runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn reader_for_current(
    runtime: worth_relational::facade::runtime::RelationalRuntime,
    snapshot_identity: crate::facade::TruthSnapshotIdentity,
    expected_version: worth_relational::facade::identity::VersionId,
) -> RuntimePublicationSnapshotReader {
    let branch = worth_relational::facade::history::BranchId("main".to_owned());
    let identity = runtime
        .branch_identity(&branch)
        .expect("main branch identity");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("main branch basis");
    let observation = basis.observation();
    assert_eq!(observation.version_id(), expected_version);
    RuntimePublicationSnapshotReader::for_observation_authority(
        worth_relational::facade::change_source::RelationalRuntimeHandle::immutable(Arc::new(
            runtime,
        )),
        snapshot_identity,
        observation,
        None,
    )
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid test aspect key")
}

fn scalar_string_contract(aspect: &str) -> SnapshotReadContract {
    SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String)
}

#[path = "snapshot_serial_memory_tests.rs"]
mod serial_memory;

#[path = "snapshot_request_work_tests.rs"]
mod request_work;

#[test]
fn guarded_authorization_lowers_only_actual_relational_matches() {
    use crate::facade::{
        BridgeAuthorizationBindingIdentity, BridgeAuthorizationClauseContract,
        BridgeAuthorizationClauseObservation, BridgeAuthorizationInstallationRequest,
        BridgeAuthorizationObservation, BridgeAuthorizationRequirementContract,
        BridgeAuthorizationRequirementObservation, BridgeAuthorizationRuleContract,
        BridgeAuthorizationRuleEffect, BridgeAuthorizationRuleObservation,
        BridgeAuthorizationRuntime,
    };
    use worth_foundational::facade::CanonicalDigestId;
    use worth_relational::facade::{
        authorization::{RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan},
        identity::KindId,
    };

    let host = crate::snapshot::test_serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let principal = create_entity(execution, &runtime, "principal");
    let scope = create_entity_outcome(execution, &runtime, "other scope");
    let other = crate::relational_source::relational_test_support::changed_entities(&scope)[0];
    let mut bridge = BridgeAuthorizationRuntime::new();
    let digest = CanonicalDigestId::new([4; 32]);
    let identity = [5; 32];
    let correspondence = bridge
        .install(BridgeAuthorizationInstallationRequest::new(
            &digest,
            BridgeAuthorizationBindingIdentity::new(1, 1, digest.clone(), digest.clone()),
            "read",
            "subject",
            "self",
            [BridgeAuthorizationRuleContract::all(
                BridgeAuthorizationRuleEffect::Required,
                [BridgeAuthorizationRequirementContract::any([
                    BridgeAuthorizationClauseContract::new(identity),
                ])],
            )],
        ))
        .unwrap();

    let observe = |target| {
        let plan = RelationalAuthorizationObservationPlan::try_new(
            scope.snapshot.clone(),
            principal,
            target,
            KindId(1),
            KindId(1),
            [RelationalAuthorizationPathPlan::new([], [])],
            [],
        )
        .unwrap();
        runtime.observe_authorization(plan, execution).unwrap()
    };
    let positive = observe(principal);
    let negative = observe(other);
    let rules = |clause| {
        [BridgeAuthorizationRuleObservation::all(
            BridgeAuthorizationRuleEffect::Required,
            [BridgeAuthorizationRequirementObservation::any([clause])],
        )]
    };
    for (evidence, guard, expected) in [
        (&positive, true, true),
        (&positive, false, false),
        (&negative, true, false),
    ] {
        let clause = BridgeAuthorizationClauseObservation::from_guarded_relational_path(
            identity,
            &evidence.paths()[0],
            guard,
        );
        let observation =
            BridgeAuthorizationObservation::new(correspondence, evidence, rules(clause));
        let decision = bridge.evaluate(observation).unwrap();
        assert_eq!(decision.is_allowed(), expected);
        assert_eq!(
            decision.dependency_identity(),
            evidence.observation_identity().bytes()
        );
    }

    let forged =
        BridgeAuthorizationClauseObservation::from_relational_path(identity, &positive.paths()[0]);
    let observation = BridgeAuthorizationObservation::new(correspondence, &negative, rules(forged));
    assert_eq!(
        bridge
            .evaluate(observation)
            .err()
            .expect("mixed evidence must be rejected")
            .kind(),
        crate::facade::BridgeAuthorizationDenialKind::ObservationShapeMismatch
    );
}
