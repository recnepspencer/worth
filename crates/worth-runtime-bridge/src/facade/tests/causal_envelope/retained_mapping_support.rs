use super::*;
use crate::facade::{
    BridgeCanonicalBulkPlanRecord, BridgeCanonicalContinuityRecord, BridgeCanonicalMergeRecord,
    BridgeCanonicalStructuralBranchComparisonRecord, BridgeCanonicalStructuralRemapRecord,
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReference, BridgeCausalEvidenceReferenceIdentity,
    BridgeHistoricalEvaluationFailureRecord, BridgeHistoricalResolvedLineageIdentity,
    BridgeHistoricalResolvedRecordIdentity, BridgeMergeAuthorityBasis,
    BridgeMergeAuthorityBasisKind, BridgeMergeConsumptionClass, BridgeMergeOntologyMappingSurface,
    BridgeMergeParentOrderProof, BridgeMergeStructuralAdvisoryDisposition,
    BridgeRouteResultSummary, CanonicalStreamReplayRecord, ConsumerCheckpointToken,
    MergeHistoryDeclaration, MergeHistoryDeclarationIdentity, SourceFailureRecord,
    SourceMaterializationRecord,
};

#[derive(Clone)]
struct CausalLineageSource;

impl crate::adapter::ContinuityLineageSource for CausalLineageSource {
    fn historical_lineage(
        &self,
        request: crate::adapter::BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::adapter::BridgeHistoricalLineageAuthority,
        crate::error::BridgeLineageSourceError,
    > {
        crate::adapter::BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:causal-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "record:causal-successor",
            )],
            vec![1],
        )
    }
}

pub(super) fn bridge_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    let family = identity.family();
    BridgeCausalEvidenceReference::new(BridgeCausalEvidenceOwner::RuntimeBridge, family, identity)
        .expect("bridge reference should be valid")
}

pub(super) fn query_observation_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::Query,
        BridgeCausalEvidenceFamily::QueryObservation,
        identity,
    )
    .expect("query observation reference should be valid")
}

pub(super) fn missing_bridge_reference(
    family: BridgeCausalEvidenceFamily,
    identity: &str,
) -> BridgeCausalEvidenceReference {
    bridge_reference(
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            family,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(identity),
        )
        .expect("bridge reference identity should be valid"),
    )
}

pub(super) fn bridge_route_reference(
    route_summary: &BridgeRouteResultSummary,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeRoute,
        route_summary.route_identity().as_str(),
    )
}

pub(super) fn bridge_bulk_planning_reference(
    record: &BridgeCanonicalBulkPlanRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeBulkPlanning,
        record.workload_identity().as_str(),
    )
}

pub(super) fn bridge_source_materialization_reference(
    record: &SourceMaterializationRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeSourceMaterialization,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_source_failure_reference(
    record: &SourceFailureRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeSourceFailure,
        record.failure_identity().as_str(),
    )
}

pub(super) fn bridge_structural_remap_reference(
    record: &BridgeCanonicalStructuralRemapRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeStructuralRemap,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_structural_branch_comparison_reference(
    record: &BridgeCanonicalStructuralBranchComparisonRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_stream_replay_reference(
    record: &CanonicalStreamReplayRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeStreamReplay,
        record.replay_record_identity().as_str(),
    )
}

pub(super) fn bridge_stream_checkpoint_reference(
    checkpoint: &ConsumerCheckpointToken,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeStreamCheckpoint,
        checkpoint.checkpoint_token_identity_for_reporting(),
    )
}

pub(super) fn bridge_continuity_reference(
    record: &BridgeCanonicalContinuityRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeContinuity,
        record.route_identity().as_str(),
    )
}

pub(super) fn bridge_merge_reference(
    record: &BridgeCanonicalMergeRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeMerge,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_historical_evaluation_failure_reference(
    record: &BridgeHistoricalEvaluationFailureRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeHistoricalEvaluationFailure,
        record.failure_identity().as_str(),
    )
}

pub(super) fn binding_for<'a>(
    bindings: &'a [BridgeCausalEvidenceBinding],
    family: BridgeCausalEvidenceFamily,
    reference_identity: &str,
) -> &'a BridgeCausalEvidenceBinding {
    bindings
        .iter()
        .find(|binding| {
            binding.owner() == BridgeCausalEvidenceOwner::RuntimeBridge
                && binding.family() == family
                && binding.reference_evidence_identity().as_str() == reference_identity
        })
        .expect("expected retained bridge binding should be present")
}

pub(super) fn registered_causal_merge(
    declaration_identity: MergeHistoryDeclarationIdentity,
) -> MergeHistoryDeclaration {
    let authority_artifact_identity = format!("merge-artifact:{}", declaration_identity.as_str());
    MergeHistoryDeclaration::new(
        declaration_identity,
        BridgeMergeConsumptionClass::AspectReconciliationMerge,
        BridgeMergeOntologyMappingSurface::direct_phase_m9_0("rel-merge-v1"),
        BridgeMergeAuthorityBasis::new(
            BridgeMergeAuthorityBasisKind::OrderedMergeCommit,
            authority_artifact_identity,
            "rel-merge-v1",
            "schema-policy-v1",
            BridgeMergeParentOrderProof::new(vec![
                crate::truth_identity_fixtures::truth_commit_fixture("parent-a"),
                crate::truth_identity_fixtures::truth_commit_fixture("parent-b"),
            ]),
        ),
    )
    .with_structural_advisory(BridgeMergeStructuralAdvisoryDisposition::AdvisoryConsistent)
}

pub(super) fn branch_comparison_declaration(
    declaration_identity: StructuralIdentityDeclarationIdentity,
) -> StructuralIdentityDeclaration {
    StructuralIdentityDeclaration::branch_comparison(
        declaration_identity,
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        StructuralFingerprintEquivalenceContract::new(
            StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
            StructuralFingerprintFamily::BranchComparisonFingerprint,
            "geometry-branch-v1",
            StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
            StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
            StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
        ),
        StructuralTruthViewBasis::explicit_branch_pair(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("left"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("right"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
        ),
    )
}

pub(super) fn retained_runtime(
    merge_declaration: MergeHistoryDeclaration,
    branch_declaration: StructuralIdentityDeclaration,
) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_continuity_lineage_source(CausalLineageSource)
        .with_signal_sink(StaticSink)
        .register_source(registered_source(
            "source:analysis-snapshot",
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ],
        ))
        .register_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .register_structural(registered_structural(
            "structural:analysis-snapshot",
            StructuralFingerprintFamily::TopologyFingerprint,
            StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )),
        ))
        .register_structural(branch_declaration)
        .register_merge(merge_declaration)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::admit_bridge_owned("mapping"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("entity-1"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::admit_bridge_owned("signal:profile"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("retained mapping runtime should build")
}
