use worth_foundational::facade::{
    AspectFieldLocator, AspectLocator, AspectMask, AspectValue, LocatorAuthority, ProjectionMask,
    ScalarAspectType,
};

use crate::diagnostics::BridgeHistoricalMaterializationPath;
use crate::policy::BridgeDiagnosticsTier;
use crate::snapshot::{
    AdmittedSnapshotContext, BridgeDeliveryIntent, BridgeReplayMode, BridgeSnapshotContext,
    BridgeSnapshotReadErrorKind, BridgeSnapshotToken, BridgeTruthViewAuthorityBasis,
    BridgeTruthViewSelector, HistoricalEvaluationDeclaration, PlannedTruthViewPacket,
    ResolvedTruthViewPolicy, SnapshotReadContract, SnapshotReadPacket, SnapshotReadRequest,
    TruthSnapshotReader, TruthViewReplayContinuity, TruthViewRetentionAdmission,
    TruthViewSourceCapability,
};

use super::{materialize_bridge_row_set, BridgeRowSetMaterializationError};

#[path = "row_set_tests/snapshot_readers.rs"]
mod snapshot_readers;
use snapshot_readers::{
    AuthoritativeAbsenceReader, ChangedStatusReader, FixtureReader, MissingRecordReader,
};

fn observation() -> crate::snapshot::MaterializedTruthViewObservation {
    observation_with_reader(Box::new(FixtureReader) as Box<dyn TruthSnapshotReader>)
}

fn observation_with_reader(
    snapshot_reader: Box<dyn TruthSnapshotReader>,
) -> crate::snapshot::MaterializedTruthViewObservation {
    observation_with_reader_and_packet(snapshot_reader, default_row_set_packet())
}

fn default_row_set_packet() -> SnapshotReadPacket {
    SnapshotReadPacket::new(vec![
        SnapshotReadRequest::for_coarse(
            "entity-1",
            SnapshotReadContract::scalar(aspect_key("identity.id"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_coarse(
            "entity-1",
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_coarse(
            "entity-2",
            SnapshotReadContract::scalar(aspect_key("identity.id"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_coarse(
            "entity-2",
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
        ),
    ])
}

fn observation_with_reader_and_packet(
    snapshot_reader: Box<dyn TruthSnapshotReader>,
    packet: SnapshotReadPacket,
) -> crate::snapshot::MaterializedTruthViewObservation {
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        ),
        BridgeReplayMode::Disabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let packet = PlannedTruthViewPacket::new(
        declaration.clone(),
        ResolvedTruthViewPolicy::admitted(
            &declaration,
            TruthViewRetentionAdmission::HistoricalLookupRequired,
            TruthViewSourceCapability::HistoricalLookupAndSnapshotRead,
            TruthViewReplayContinuity::ReplayPermitted,
        ),
        BridgeTruthViewAuthorityBasis::from_resolved_envelope(
            declaration.selector(),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        packet,
    );
    let snapshot = BridgeSnapshotContext::bind(snapshot_reader);
    let admitted = AdmittedSnapshotContext::admit_for(
        snapshot,
        &crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    )
    .expect("snapshot should admit");
    crate::snapshot::MaterializedTruthViewObservation::new(
        packet,
        BridgeSnapshotToken::issued(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            "row-set-test",
        ),
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot,
        admitted,
    )
}

#[test]
fn bridge_row_set_preserves_multi_row_truth() {
    let policy = crate::policy::BridgeExecutionPolicyBaseline::operational().request_policy();
    let lease = crate::snapshot::test_execution_lease_for_policy(
        policy,
        worth_execution::CancellationToken::new(),
    );
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let row_set = execution
        .run(
            worth_execution::ExecutionWorkCeiling::new(8_000_000),
            |_| materialize_bridge_row_set(&observation(), execution),
        )
        .expect("active leased scope")
        .0
        .expect("row set");

    assert_eq!(row_set.rows().len(), 2);
    assert_eq!(row_set.rows()[0].row_identity().as_str(), "entity-1");
    assert_eq!(
        row_set.rows()[0]
            .whole_aspect_fields_for_key(&aspect_key("identity.id"))
            .next()
            .and_then(|field| field.scalar_value()),
        Some(&AspectValue::String("task-1".into()))
    );
    assert_eq!(
        row_set.rows()[0]
            .whole_aspect_fields_for_key(&aspect_key("status"))
            .next()
            .and_then(|field| field.scalar_value()),
        Some(&AspectValue::String("todo".into()))
    );
    let status_aspect_key = aspect_key("status");
    let status_projection = row_set.rows()[0]
        .whole_aspect_fields_for_key(&status_aspect_key)
        .next()
        .expect("status field")
        .projection();
    assert_eq!(status_projection.aspect_key().as_str(), "status");
    assert!(status_projection
        .field_identity()
        .as_str()
        .starts_with("bridge-row-field:sha256:"));
    assert!(status_projection.projection_mask().is_whole_aspect());
    assert!(status_projection
        .canonical_basis()
        .contains("locator=version=bridge.row-field-projection.v1;domain=locator;"));
    assert!(status_projection
        .canonical_basis()
        .contains("projection-mask=version=bridge.row-field-projection.v1;domain=aspect-mask;"));
    assert!(status_projection
        .canonical_basis()
        .contains(status_projection.field_identity().as_str()));
    let status_field = row_set.rows()[0]
        .whole_aspect_fields_for_key(&status_aspect_key)
        .next()
        .expect("status field");
    assert_eq!(
        crate::snapshot::contract_validated_scalar_aspect_value(status_field.validated_value()),
        Some(&AspectValue::String("todo".into()))
    );
    assert_eq!(
        status_field.validated_value_canonical_basis(),
        crate::snapshot::validated_value_basis::validated_snapshot_read_value_canonical_basis(
            status_field.validated_value()
        )
    );
    assert_eq!(row_set.rows()[1].row_identity().as_str(), "entity-2");
    assert_eq!(
        row_set.rows()[1]
            .whole_aspect_fields_for_key(&aspect_key("identity.id"))
            .next()
            .and_then(|field| field.scalar_value()),
        Some(&AspectValue::String("task-2".into()))
    );
    assert_eq!(
        row_set.rows()[1]
            .whole_aspect_fields_for_key(&aspect_key("status"))
            .next()
            .and_then(|field| field.scalar_value()),
        Some(&AspectValue::String("doing".into()))
    );
}

#[test]
fn bridge_row_set_digest_is_derived_from_validated_aspect_values() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let baseline = materialize_bridge_row_set(&observation(), execution).expect("baseline row set");
    let changed = materialize_bridge_row_set(
        &observation_with_reader(Box::new(ChangedStatusReader) as Box<dyn TruthSnapshotReader>),
        execution,
    )
    .expect("changed row set");

    assert_ne!(baseline.digest(), changed.digest());
}

#[test]
fn bridge_row_set_omits_authoritatively_absent_fields_without_panicking() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let row_set = materialize_bridge_row_set(
        &observation_with_reader(
            Box::new(AuthoritativeAbsenceReader) as Box<dyn TruthSnapshotReader>
        ),
        execution,
    )
    .expect("authoritative absence is a lawful row-set posture");

    assert_eq!(row_set.rows().len(), 2);
    assert!(row_set.rows()[1]
        .whole_aspect_fields_for_key(&aspect_key("status"))
        .next()
        .is_none());
}

#[test]
fn bridge_row_set_preserves_typed_snapshot_read_contract_failure() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let error = materialize_bridge_row_set(
        &observation_with_reader(Box::new(MissingRecordReader) as Box<dyn TruthSnapshotReader>),
        execution,
    )
    .expect_err("row-set materialization must fail before assembling partial rows");

    match error {
        BridgeRowSetMaterializationError::SnapshotReadContractFailure { error } => {
            assert_eq!(
                error.kind(),
                BridgeSnapshotReadErrorKind::RecordCountMismatch
            );
        }
        BridgeRowSetMaterializationError::DuplicateMaterializedField { .. } => {
            panic!("missing snapshot records must fail before row assembly")
        }
    }
}

#[test]
fn bridge_row_set_rejects_duplicate_materialized_field_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let packet = SnapshotReadPacket::new(vec![
        SnapshotReadRequest::for_coarse(
            "entity-1",
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_native_subscription_slice(
            "entity-1",
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
            AspectLocator::new(LocatorAuthority::Planned, aspect_key("status")),
            None::<AspectFieldLocator>,
            AspectMask::<ProjectionMask>::whole_aspect(),
            crate::mapping::SubscriptionSliceKind::SignalRegion,
        ),
    ]);

    let error = materialize_bridge_row_set(
        &observation_with_reader_and_packet(
            Box::new(FixtureReader) as Box<dyn TruthSnapshotReader>,
            packet,
        ),
        execution,
    )
    .expect_err("row assembly must not overwrite duplicate materialized fields");

    assert!(matches!(
        error,
        BridgeRowSetMaterializationError::DuplicateMaterializedField { .. }
    ));
}

fn aspect_key(value: &str) -> worth_foundational::facade::AspectKey {
    worth_foundational::facade::AspectKey::new(value).expect("valid snapshot aspect key")
}
