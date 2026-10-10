use worth_foundational::facade::{
    aspects, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectLocator,
    AspectMask, AspectValue, LocatorAuthority, ProjectionMask, ScalarAspectType,
};

use crate::diagnostics::BridgeHistoricalMaterializationPath;
use crate::policy::BridgeDiagnosticsTier;
use crate::snapshot::{
    AdmittedSnapshotContext, BridgeDeliveryIntent, BridgeReplayMode, BridgeSnapshotContext,
    BridgeSnapshotToken, BridgeTruthViewAuthorityBasis, BridgeTruthViewSelector,
    HistoricalEvaluationDeclaration, PlannedTruthViewPacket, ResolvedTruthViewPolicy,
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRequest,
    TruthSnapshotIdentity, TruthSnapshotReader, TruthViewReplayContinuity,
    TruthViewRetentionAdmission, TruthViewSourceCapability,
};

use crate::source::{
    materialize_bridge_row_set, GroupedProjectionMemberSource, GroupedProjectionSource,
};

#[derive(Debug)]
struct FixtureReader {
    identity_binding_shape: FixtureBindingValueShape,
    grouping_binding_shape: FixtureBindingValueShape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FixtureBindingValueShape {
    Scalar,
    Struct,
}

impl TruthSnapshotReader for FixtureReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let records = request
            .reads()
            .iter()
            .map(|read| {
                if read.entity_identity() == "entity-1"
                    && read.aspect_key().as_str() == "identity.id"
                    && self.identity_binding_shape == FixtureBindingValueShape::Struct
                {
                    return crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        fixture_struct_value("task-1"),
                    );
                }
                if read.entity_identity() == "entity-1"
                    && read.aspect_key().as_str() == "status.lane"
                    && self.grouping_binding_shape == FixtureBindingValueShape::Struct
                {
                    return crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        fixture_struct_value("todo"),
                    );
                }
                let snapshot_value = match (read.entity_identity(), read.aspect_key().as_str()) {
                    ("entity-1", "identity.id") => AspectValue::String("task-1".into()),
                    ("entity-1", "status.lane") => AspectValue::String("todo".into()),
                    ("entity-2", "identity.id") => AspectValue::String("task-2".into()),
                    ("entity-2", "status.lane") => AspectValue::String("doing".into()),
                    _ => AspectValue::String("unknown".into()),
                };
                crate::snapshot::SnapshotReadRecord::for_request(read, snapshot_value)
            })
            .collect();
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            records,
        ))
    }
}

#[derive(Clone)]
pub(super) struct TestProjectionMember {
    pub(super) row_identity: String,
    pub(super) identity_value: AspectValue,
    pub(super) grouping_value: AspectValue,
}

impl GroupedProjectionMemberSource for TestProjectionMember {
    fn row_identity(&self) -> &str {
        &self.row_identity
    }

    fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    fn grouping_value(&self) -> &AspectValue {
        &self.grouping_value
    }
}

pub(super) struct TestProjection {
    snapshot_identity: TruthSnapshotIdentity,
    grouping_aspect: AspectKey,
    identity_binding_aspect_key: AspectKey,
    grouping_binding_aspect_key: AspectKey,
    members: Vec<TestProjectionMember>,
}

impl GroupedProjectionSource for TestProjection {
    type Member = TestProjectionMember;

    fn basis_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    fn grouping_aspect_key(&self) -> &AspectKey {
        &self.grouping_aspect
    }

    fn identity_binding_aspect_key(&self) -> &AspectKey {
        &self.identity_binding_aspect_key
    }

    fn grouping_binding_aspect_key(&self) -> &AspectKey {
        &self.grouping_binding_aspect_key
    }

    fn members(&self) -> &[Self::Member] {
        &self.members
    }
}

pub(super) fn row_set(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
    row_set_with_binding_shapes(
        FixtureBindingValueShape::Scalar,
        FixtureBindingValueShape::Scalar,
        resource_request,
    )
}

pub(super) fn row_set_with_struct_identity_binding(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
    row_set_with_binding_shapes(
        FixtureBindingValueShape::Struct,
        FixtureBindingValueShape::Scalar,
        resource_request,
    )
}

pub(super) fn row_set_with_struct_grouping_binding(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
    row_set_with_binding_shapes(
        FixtureBindingValueShape::Scalar,
        FixtureBindingValueShape::Struct,
        resource_request,
    )
}

pub(super) fn row_set_with_ambiguous_grouping_binding(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
    row_set_from_packet(
        SnapshotReadPacket::new(vec![
            native_string_read("entity-1", "identity.id"),
            native_string_read("entity-1", "status.lane"),
            native_support_only_whole_aspect_read("entity-1", "status.lane"),
            native_string_read("entity-2", "identity.id"),
            native_string_read("entity-2", "status.lane"),
        ]),
        FixtureBindingValueShape::Scalar,
        FixtureBindingValueShape::Scalar,
        resource_request,
    )
}

fn row_set_with_binding_shapes(
    identity_binding_shape: FixtureBindingValueShape,
    grouping_binding_shape: FixtureBindingValueShape,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
    row_set_from_packet(
        SnapshotReadPacket::new(vec![
            native_read_for("entity-1", "identity.id", identity_binding_shape),
            native_read_for("entity-1", "status.lane", grouping_binding_shape),
            native_string_read("entity-2", "identity.id"),
            native_string_read("entity-2", "status.lane"),
        ]),
        identity_binding_shape,
        grouping_binding_shape,
        resource_request,
    )
}

fn row_set_from_packet(
    read_packet: SnapshotReadPacket,
    identity_binding_shape: FixtureBindingValueShape,
    grouping_binding_shape: FixtureBindingValueShape,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::source::BridgeMaterializedRowSetArtifact {
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
        read_packet,
    );
    let snapshot = BridgeSnapshotContext::bind(Box::new(FixtureReader {
        identity_binding_shape,
        grouping_binding_shape,
    }) as Box<dyn TruthSnapshotReader>);
    let admitted = AdmittedSnapshotContext::admit_for(
        snapshot,
        &crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    )
    .expect("snapshot should admit");
    let observation = crate::snapshot::MaterializedTruthViewObservation::new(
        packet,
        BridgeSnapshotToken::issued(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            "grouped-truth-test",
        ),
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot,
        admitted,
    );
    materialize_bridge_row_set(&observation, execution).expect("row set")
}

pub(super) fn projection(
    snapshot_identity: &str,
    identity_binding_aspect_key: &str,
    grouping_binding_aspect_key: &str,
    members: Vec<TestProjectionMember>,
) -> TestProjection {
    projection_with_grouping(
        "status",
        snapshot_identity,
        identity_binding_aspect_key,
        grouping_binding_aspect_key,
        members,
    )
}

pub(super) fn projection_with_grouping(
    grouping_aspect: &str,
    snapshot_identity: &str,
    identity_binding_aspect_key: &str,
    grouping_binding_aspect_key: &str,
    members: Vec<TestProjectionMember>,
) -> TestProjection {
    TestProjection {
        snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
            snapshot_identity,
        ),
        grouping_aspect: native_aspect_key(grouping_aspect),
        identity_binding_aspect_key: native_aspect_key(identity_binding_aspect_key),
        grouping_binding_aspect_key: native_aspect_key(grouping_binding_aspect_key),
        members,
    }
}

pub(super) fn standard_members() -> Vec<TestProjectionMember> {
    vec![
        TestProjectionMember {
            row_identity: "entity-1".to_string(),
            identity_value: AspectValue::String("task-1".into()),
            grouping_value: AspectValue::String("todo".into()),
        },
        TestProjectionMember {
            row_identity: "entity-2".to_string(),
            identity_value: AspectValue::String("task-2".into()),
            grouping_value: AspectValue::String("doing".into()),
        },
    ]
}

fn native_string_read(entity_identity: &str, aspect_key: &str) -> SnapshotReadRequest {
    SnapshotReadRequest::for_coarse(
        entity_identity,
        SnapshotReadContract::scalar(native_aspect_key(aspect_key), ScalarAspectType::String),
    )
}

fn native_read_for(
    entity_identity: &str,
    aspect_key: &str,
    binding_shape: FixtureBindingValueShape,
) -> SnapshotReadRequest {
    match binding_shape {
        FixtureBindingValueShape::Scalar => native_string_read(entity_identity, aspect_key),
        FixtureBindingValueShape::Struct => SnapshotReadRequest::for_coarse(
            entity_identity,
            native_struct_read_contract(aspect_key),
        ),
    }
}

fn native_support_only_whole_aspect_read(
    entity_identity: &str,
    aspect_key: &str,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_native_subscription_slice(
        entity_identity,
        SnapshotReadContract::scalar(native_aspect_key(aspect_key), ScalarAspectType::String),
        AspectLocator::new(LocatorAuthority::SupportOnly, native_aspect_key(aspect_key)),
        None,
        AspectMask::<ProjectionMask>::whole_aspect(),
        crate::mapping::SubscriptionSliceKind::RegisteredCoarseWidening,
    )
}

fn native_struct_read_contract(aspect_key: &str) -> SnapshotReadContract {
    let shape = aspects()
        .struct_fields()
        .required("value", ScalarAspectType::String)
        .finish()
        .expect("valid fixture struct shape");
    SnapshotReadContract::new(AspectContract::struct_aspect(
        native_aspect_key(aspect_key),
        AspectIdentity(91),
        AspectContractRevision(1),
        shape,
    ))
}

fn native_aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("fixture aspect key must be foundational")
}

fn fixture_struct_value(value: &str) -> worth_foundational::facade::StructAspectValue {
    aspects()
        .vocabulary()
        .struct_value()
        .with_field("value", AspectValue::String(value.into()))
        .finish()
        .expect("valid fixture struct value")
}
