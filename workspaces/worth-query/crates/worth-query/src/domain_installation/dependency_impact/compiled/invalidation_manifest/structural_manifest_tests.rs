use std::sync::Arc;

use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectMask,
    AuthoritativeAspectChangeKind, CanonicalFieldPath, FieldKey, ProjectionMask, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeAuthoritativeSourceProfile, BridgeAuthoritativeSourceProvenance,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedRecordChange, BridgeCommittedRecordChangeKind, BridgeDeliveryReceipt,
    BridgeMappingId, BridgeMappingRegistration, BridgeProducerMetadata,
    BridgeSemanticCorrespondenceRegistration, BridgeSemanticDependencyCandidate,
    BridgeSemanticDependencyCandidateParts, BridgeSemanticLocality,
    BridgeSignalAspectTargetDeclaration, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
    MappingSelector, RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadPacketResult, SnapshotReadSource, SubscriptionSliceKind, TruthBranchHeadSource,
    TruthBranchIdentity, TruthCommitIdentity, TruthDeltaSurfaceKind, TruthPatchIdentity,
    TruthPatchScope, TruthPatchTargetSelector, TruthSnapshotIdentity, TruthSnapshotReader,
};
use worth_signal::facade::SignalGraph;

use super::*;
use crate::domain_installation::dependency_impact::compiled::{
    WorthQuerySemanticAspectDependencyLocus, WorthQuerySemanticAspectDependencySource,
};

#[test]
fn bound_primary_manifest_adds_direct_structural_roles_without_signal_consequences() {
    for case in [
        StructuralCase::region(
            BridgeCommittedRecordChangeKind::Created,
            AuthoritativeAspectChangeKind::StructuralCreate,
            WorthQuerySemanticDependencyRole::SelectionOrMembership,
        ),
        StructuralCase::region(
            BridgeCommittedRecordChangeKind::Deleted,
            AuthoritativeAspectChangeKind::StructuralDelete,
            WorthQuerySemanticDependencyRole::SelectionOrMembership,
        ),
        StructuralCase::lifecycle(
            BridgeCommittedRecordChangeKind::Deleted,
            AuthoritativeAspectChangeKind::LifecycleDelete,
            WorthQuerySemanticDependencyRole::SupportAndLifecycle,
        ),
    ] {
        let candidate = bridge_candidate(&case);
        let receipt = deliver_structural_change(&case, candidate.clone());
        assert_eq!(receipt.change_set().changes().len(), 1);

        let manifest = manifest(&case, &candidate);
        let (roles, _) = manifest
            .select_bound_primary_roles(
                &[&candidate],
                &candidate,
                receipt.change_set().changes(),
                false,
                1,
            )
            .expect("the installed bound dependency must match its Bridge delivery");
        assert_eq!(roles, [case.expected_role]);
    }
}

#[derive(Clone)]
struct StructuralCase {
    binding: AspectBinding,
    target: TruthPatchTargetSelector,
    surface: TruthDeltaSurfaceKind,
    record_kind: BridgeCommittedRecordChangeKind,
    effective_kind: AuthoritativeAspectChangeKind,
    expected_role: WorthQuerySemanticDependencyRole,
}

impl StructuralCase {
    fn region(
        record_kind: BridgeCommittedRecordChangeKind,
        effective_kind: AuthoritativeAspectChangeKind,
        expected_role: WorthQuerySemanticDependencyRole,
    ) -> Self {
        Self {
            binding: AspectBinding::StructuralRegion,
            target: TruthPatchTargetSelector::region(),
            surface: TruthDeltaSurfaceKind::EntityRegion,
            record_kind,
            effective_kind,
            expected_role,
        }
    }

    fn lifecycle(
        record_kind: BridgeCommittedRecordChangeKind,
        effective_kind: AuthoritativeAspectChangeKind,
        expected_role: WorthQuerySemanticDependencyRole,
    ) -> Self {
        Self {
            binding: AspectBinding::LifecycleTransition,
            target: TruthPatchTargetSelector::lifecycle_transition(),
            surface: TruthDeltaSurfaceKind::LifecycleTransition,
            record_kind,
            effective_kind,
            expected_role,
        }
    }
}

fn manifest(
    case: &StructuralCase,
    candidate: &BridgeSemanticDependencyCandidate,
) -> WorthQueryInstalledInvalidationManifest {
    let query_dependency =
        worth_query_installation::facade::WorthQuerySemanticTruthDependency::new(
            worth_query_installation::facade::WorthQueryConditionalGraphReadRole::new("primary")
                .unwrap(),
            contract(),
            AspectMask::<ProjectionMask>::whole_aspect(),
            case.binding.clone(),
            worth_query_installation::facade::WorthQuerySemanticLocality::SourceRecord,
            [case.effective_kind],
        )
        .unwrap();
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            candidate,
        );
    let mut dependencies = vec![WorthQueryCompiledSemanticAspectDependency::new(
        WorthQuerySemanticAspectDependencyLocus::ConditionalTruth {
            location,
            dependency_ordinal: 0,
        },
        WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness,
        WorthQuerySemanticAspectDependencySource::ConditionalTruth(query_dependency),
    )];
    if case.expected_role == WorthQuerySemanticDependencyRole::SelectionOrMembership {
        dependencies.push(WorthQueryCompiledSemanticAspectDependency::new(
            WorthQuerySemanticAspectDependencyLocus::CollectionRowIdentity,
            WorthQuerySemanticDependencyRole::SelectionOrMembership,
            WorthQuerySemanticAspectDependencySource::CollectionField(
                worth_query_installation::facade::WorthQueryOperationCollectionField::new(
                    aspect_key(),
                    CanonicalFieldPath::single(FieldKey::new("identity").unwrap()),
                ),
            ),
        ));
    }
    let index = super::super::impact_index::WorthQuerySemanticImpactIndex::compile(&dependencies);
    WorthQueryInstalledInvalidationManifest::compile(&authority_basis(), &dependencies, &index)
}

fn deliver_structural_change(
    case: &StructuralCase,
    candidate: BridgeSemanticDependencyCandidate,
) -> worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let TransitionOutcome::Success(node) = graph.admit_installed_node(node) else {
        panic!("fixture node must be installed")
    };
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("structural-manifest"),
        TruthPatchScope::new(
            MappingSelector::exact(record().terminal_projection_for_reporting()),
            AspectKeySelector::exact(aspect_key()),
            case.target.clone(),
        ),
        SnapshotReadContract::new(contract()),
        SignalInvalidationScope::from_stable_name("structural-manifest"),
        CoarseRoutingMode::Direct,
    );
    let registration_id = BridgeAspectRegistrationId::from_stable_name("structural-manifest");
    let aspect = BridgeAspectRegistration::new(
        registration_id.clone(),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        case.surface,
        SubscriptionSliceKind::SignalAspect,
        SliceWideningPolicy::Disallow,
    );
    let target = BridgeSignalAspectTargetDeclaration::allocate(
        registration_id,
        worth_signal::facade::PartitionToken::new("primary"),
        node,
    );
    let registration =
        BridgeSemanticCorrespondenceRegistration::new(candidate.clone(), vec![target])
            .expect("fixture correspondence must be valid");
    let envelope = BridgeCommittedPatchEnvelope::new_with_record_changes(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::registered_authoritative_source().with_authoritative_source(
                BridgeAuthoritativeSourceProvenance::from_owner_publication(
                    1,
                    "primary",
                    "primary-adapter",
                    "primary-basis",
                ),
            ),
            TruthCommitIdentity::from_relational_commit_id(1),
            TruthPatchIdentity::from_relational_patch_position(1),
            snapshot(),
            TruthBranchIdentity::from_relational_branch_id("main"),
        ),
        Vec::new(),
        vec![BridgeCommittedRecordChange::from_relational_publication(
            record(),
            case.record_kind,
        )],
    )
    .expect("structural committed envelope must be canonical");
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(StructuralSource(envelope))
        .with_signal_sink(StructuralSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect)
        .register_semantic_correspondence(registration)
        .build()
        .expect("structural Bridge runtime must build");
    let mut binding = runtime
        .bind_signal_graph(&mut graph)
        .expect("runtime and Signal graph must bind");
    let TransitionOutcome::Success(installed) = binding.install_semantic_correspondence(candidate)
    else {
        panic!("structural semantic correspondence must install")
    };
    match binding.deliver_installed_correspondence(
        &installed,
        worth_runtime_bridge::facade::RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(1),
        ),
    ) {
        TransitionOutcome::Success(receipt) => receipt,
        TransitionOutcome::Denied(denial) => {
            panic!("structural committed change denied by Bridge: {denial:?}")
        }
        TransitionOutcome::Deferred(deferred) => {
            panic!("structural committed change deferred by Bridge: {deferred:?}")
        }
        TransitionOutcome::Stale(stale) => {
            panic!("structural committed change was stale in Bridge: {stale:?}")
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("structural committed change required rebind: {rebind:?}")
        }
        TransitionOutcome::Failed(failure) => {
            panic!("structural committed change failed in Bridge: {failure:?}")
        }
    }
}

fn bridge_candidate(case: &StructuralCase) -> BridgeSemanticDependencyCandidate {
    BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
        source_installation_identity: Arc::from("primary-installation"),
        source_basis: Arc::from("primary-basis"),
        source_runtime_authority: 1,
        source_installation_generation: 1,
        source_authority_binding_identity: Arc::from("primary-binding"),
        source_stage_identity: None,
        source_node_identity: Arc::from("structural-node"),
        dependency_ordinal: 0,
        declared_graph_role: Arc::from("primary"),
        graph_participation_identity: Arc::from("primary-graph"),
        graph_adapter_identity: Arc::from("primary-adapter"),
        source_record_identity: Some(record()),
        observation_record_identity: Some(record()),
        contract: contract(),
        projection_mask: AspectMask::<ProjectionMask>::whole_aspect(),
        binding: case.binding.clone(),
        locality: BridgeSemanticLocality::SourceRecord,
        relevant_changes: vec![case.effective_kind],
    })
    .expect("structural dependency candidate must be portable")
}

fn authority_basis() -> WorthQueryOperationAuthorityBasis {
    WorthQueryOperationAuthorityBasis {
        runtime_authority: 1,
        installation_runtime_authority: 1,
        installation_generation: 1,
        domain_authority_identity: "domain".into(),
        operation_identity: "operation".into(),
        binding_identity: "binding".into(),
        capability_identity: 1,
        basis_identity: "basis".into(),
        graph_authority_identities: vec!["primary-graph".into()],
        required_domain_authority_identities: Vec::new(),
        resource_admission_identity: None,
    }
}

fn contract() -> AspectContract {
    AspectContract::scalar(
        aspect_key(),
        AspectIdentity(41),
        AspectContractRevision(1),
        ScalarAspectType::UInt64,
    )
}

fn aspect_key() -> AspectKey {
    AspectKey::new("structural.facts").unwrap()
}

fn record() -> RelationalBridgeRecordIdentityParts {
    RelationalBridgeRecordIdentityParts::entity(0, 1, 1)
}

fn snapshot() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
}

struct StructuralSource(BridgeCommittedPatchEnvelope);

impl CommittedPatchSource for StructuralSource {
    fn authoritative_source_profile(&self) -> Option<BridgeAuthoritativeSourceProfile> {
        Some(BridgeAuthoritativeSourceProfile::new(1, "primary-adapter").unwrap())
    }

    fn load_committed_patch(
        &self,
        _request: worth_runtime_bridge::facade::RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(self.0.clone())
    }
}

impl SnapshotReadSource for StructuralSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(StructuralSnapshotReader))
    }
}

impl TruthBranchHeadSource for StructuralSource {
    fn load_branch_head_patch(
        &self,
        _branch: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        unreachable!("the court never reads a branch head")
    }
}

struct StructuralSnapshotReader;

impl TruthSnapshotReader for StructuralSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        snapshot()
    }

    fn read_packet(
        &self,
        _request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, worth_runtime_bridge::facade::BridgeSnapshotReadError>
    {
        unreachable!("structural correspondence delivery does not project a snapshot")
    }
}

struct StructuralSink;

impl InvalidationSink for StructuralSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: Option<&worth_runtime_bridge::facade::ExecutionResourceLease<'_>>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
