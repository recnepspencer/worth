use worth_foundational::facade::{
    AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeDeliveryIntent,
    BridgeDiagnosticsTier, BridgeReplayMode, BridgeRuntimePolicy, BridgeSignalInvalidationDelivery,
    BridgeSnapshotReadError, BridgeSourceAdapter, BridgeSourceCapability,
    BridgeSourceCapabilitySet, BridgeTruthViewEvaluationRequest, BridgeTruthViewSelector,
    CoarseRoutingMode, ExecutionRequest, HistoricalEvaluationDeclaration, InvalidationSink,
    RelationalBridgeSnapshotIdentityParts, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult,
    SnapshotReadRecord, SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity,
    TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity,
    TruthPatchScope, TruthPatchTargetSelector, TruthSnapshotIdentity, TruthSnapshotReader,
};

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::historical::{
    admit_historical_evaluation_path, bridge_historical_basis,
    lower_materialization_from_decision_log, lower_policy_resolution,
    resolve_historical_materialization_path, HistoricalEvaluationRequest,
    HistoricalPathReuseDescriptor,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};

use super::super::{RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource};

pub(crate) fn representative_historical_bridge_lowering_row() -> RepresentativeArtifacts {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(analysis_branch_identity(), commit_a_identity()),
        BridgeReplayMode::Required,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let basis = bridge_historical_basis(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
    );
    let request = HistoricalEvaluationRequest::delta_replay(
        &basis,
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    );
    let capability = lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("historical policy should lower");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("historical path should admit");
    let evaluation = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                analysis_branch_identity(),
                commit_a_identity(),
            )
            .with_replay_mode(BridgeReplayMode::Required),
        )
        .expect("historical evaluation should succeed");
    let lowered = lower_materialization_from_decision_log(evaluation.record().decision_log())
        .expect("historical decision log should lower");
    let resolved = resolve_historical_materialization_path(admission, lowered)
        .expect("historical path should resolve");

    let historical_evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                &declaration
                    .declaration_identity()
                    .bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("admitted_path_class"),
                resolved.admitted_path_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("resolved_path_class"),
                resolved.resolved_path_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("complexity_contract"),
                resolved.complexity_contract().contract_name(),
            )
            .seal();
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        "Historical policy lowering",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "historical-bridge-lowering-subject",
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("declaration"),
            &historical_evidence,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("resolved_path_class"),
            resolved.resolved_path_class().as_str(),
        )
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &historical_evidence,
    );
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "historical-bridge-lowering-route",
            &historical_evidence,
        ),
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-historical-route",
            &historical_evidence,
        );
    let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route_plan,
        &retained_evidence_identity,
    );
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
        &route_plan,
        &boundary_receipt,
        &retained_evidence_identity,
    );
    RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
        request,
        eligibility,
        route_plan: Some(route_plan),
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    }
}

#[derive(Clone)]
struct StaticSource;

impl worth_runtime_bridge::facade::CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(136),
                snapshot_a_identity(),
                analysis_branch_identity(),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                worth_runtime_bridge::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid native bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("name".to_owned())
                            .expect("valid native bridge patch field key"),
                    ),
                ),
            )],
        )
        .expect("native bridge patch envelope fixture must construct"))
    }
}

impl SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity.relational_snapshot_parts().is_some() {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{:?}`",
                identity
            )))
        }
    }
}

impl TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new(
                TruthCommitIdentity::from_relational_commit_id(136),
                TruthPatchIdentity::from_relational_patch_position(137),
                snapshot_a_identity(),
                branch_identity.clone(),
            ),
            vec![profile_name_patch_item("entity-1")],
        )
        .expect("historical branch-head fixture must build a native patch envelope"))
    }
}

#[derive(Clone)]
struct StaticSnapshotReader;

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        snapshot_a_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            snapshot_a_identity(),
            request
                .reads()
                .iter()
                .map(|read| {
                    SnapshotReadRecord::for_request(
                        read,
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "fixture",
                        ),
                    )
                })
                .collect(),
        ))
    }
}

#[derive(Clone)]
struct StaticSourceAdapter;

impl BridgeSourceAdapter for StaticSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity.relational_snapshot_parts().is_some() {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{:?}`",
                identity
            )))
        }
    }
}

struct StaticSink;

impl InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: ExecutionRequest<'_, '_>,
    ) -> Result<worth_runtime_bridge::facade::BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(worth_runtime_bridge::facade::BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

fn runtime(policy: BridgeRuntimePolicy) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_source(registered_source(
            "source:analysis-snapshot",
            BridgeTruthViewSelector::branch_snapshot(
                analysis_branch_identity(),
                snapshot_a_identity(),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ],
        ))
        .register_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                analysis_branch_identity(),
                commit_a_identity(),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .register_mapping(
            worth_runtime_bridge::facade::BridgeMappingRegistration::new(
                worth_runtime_bridge::facade::BridgeMappingId::from_stable_name("mapping"),
                TruthPatchScope::new(
                    worth_runtime_bridge::facade::MappingSelector::exact("entity-1"),
                    AspectKeySelector::exact(aspect_key("profile")),
                    TruthPatchTargetSelector::entity_field(field_key("name")),
                ),
                SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
                SignalInvalidationScope::from_stable_name("signal:profile"),
                CoarseRoutingMode::Direct,
            ),
        )
        .build()
        .expect("historical lower-runtime runtime should build")
}

fn profile_name_patch_item(entity_identity: &str) -> BridgeCommittedPatchItem {
    BridgeCommittedPatchItem::with_target(
        entity_identity,
        BridgeCommittedPatchTarget::entity_field_path(
            AspectLocator::new(LocatorAuthority::Authoritative, aspect_key("profile")),
            CanonicalFieldPath::single(field_key("name")),
        ),
    )
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid historical fixture aspect key")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid historical fixture field key")
}

fn analysis_branch_identity() -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id("analysis")
}

fn commit_a_identity() -> TruthCommitIdentity {
    TruthCommitIdentity::from_relational_commit_id(6)
}

fn snapshot_a_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        6, 1,
    ))
}

fn registered_source(
    id: &str,
    selector: BridgeTruthViewSelector,
    capabilities: Vec<BridgeSourceCapability>,
) -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name(id),
        selector,
        BridgeSourceCapabilitySet::new(capabilities),
    )
}
