use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem,
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration,
    BridgeSignalInvalidationDelivery, BridgeSnapshotReadError, BridgeSourceAdapter,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadSource,
    SourceDeclaration, SourceDeclarationIdentity, StructuralFingerprintEquivalenceContract,
    StructuralFingerprintFamily, StructuralFingerprintNormalizationRule,
    StructuralFingerprintOmissionPolicy, StructuralFingerprintOrderingRule,
    StructuralIdentityDeclaration, StructuralIdentityDeclarationIdentity, StructuralSchemaIdentity,
    StructuralTruthViewBasis, TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity,
    TruthPatchIdentity, TruthPatchScope, TruthPatchTargetSelector, TruthSnapshotIdentity,
    TruthSnapshotReader, TruthWritebackAuthority, TruthWritebackAuthorityError,
    TruthWritebackReceipt, TruthWritebackRequest,
};

pub(in crate::runtime::tests::causal_inspection) use super::references::*;

struct MaterializationSource;

impl worth_runtime_bridge::facade::CommittedPatchSource for MaterializationSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                materialization_patch_identity_for_commit(request.commit_identity()),
                causal_materialization_snapshot_identity(),
                causal_materialization_branch_identity(),
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

impl SnapshotReadSource for MaterializationSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(MaterializationSnapshotReader::new(
            identity.clone(),
        )))
    }
}

impl TruthBranchHeadSource for MaterializationSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                materialization_head_commit_identity_for_branch(branch_identity),
                materialization_patch_identity_for_branch(branch_identity),
                causal_materialization_snapshot_identity(),
                branch_identity.clone(),
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
        .expect("native bridge branch head envelope fixture must construct"))
    }
}

fn materialization_patch_identity_for_commit(
    commit_identity: &TruthCommitIdentity,
) -> TruthPatchIdentity {
    commit_identity
        .relational_commit_id()
        .map(TruthPatchIdentity::from_relational_patch_position)
        .unwrap_or_else(|| TruthPatchIdentity::from_relational_patch_position(1))
}

fn materialization_head_commit_identity_for_branch(
    branch_identity: &TruthBranchIdentity,
) -> TruthCommitIdentity {
    branch_identity
        .relational_branch_id()
        .map(|branch_id| {
            TruthCommitIdentity::from_relational_commit_id(stable_causal_position(
                "branch-head",
                branch_id,
            ))
        })
        .unwrap_or_else(|| TruthCommitIdentity::from_relational_commit_id(1))
}

fn materialization_patch_identity_for_branch(
    branch_identity: &TruthBranchIdentity,
) -> TruthPatchIdentity {
    branch_identity
        .relational_branch_id()
        .map(|branch_id| {
            TruthPatchIdentity::from_relational_patch_position(stable_causal_position(
                "branch-head-patch",
                branch_id,
            ))
        })
        .unwrap_or_else(|| TruthPatchIdentity::from_relational_patch_position(1))
}

pub(in crate::runtime::tests::causal_inspection) fn causal_materialization_branch_identity(
) -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id("analysis")
}

pub(in crate::runtime::tests::causal_inspection) fn causal_truth_analysis_branch_identity(
) -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id("truth:analysis")
}

pub(in crate::runtime::tests::causal_inspection) fn causal_materialization_commit_identity(
) -> TruthCommitIdentity {
    TruthCommitIdentity::from_relational_commit_id(1)
}

pub(in crate::runtime::tests::causal_inspection) fn causal_materialization_snapshot_identity(
) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
}

pub(in crate::runtime::tests::causal_inspection) fn stable_causal_position(
    namespace: impl AsRef<str>,
    evidence: impl AsRef<str>,
) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.as_ref().bytes().chain(evidence.as_ref().bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211_u64);
    }
    acc
}

impl BridgeSourceAdapter for MaterializationSource {
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
        Ok(Box::new(MaterializationSnapshotReader::new(
            identity.clone(),
        )))
    }
}

struct MaterializationSnapshotReader {
    snapshot_identity: TruthSnapshotIdentity,
}

impl MaterializationSnapshotReader {
    fn new(snapshot_identity: TruthSnapshotIdentity) -> Self {
        Self { snapshot_identity }
    }
}

impl TruthSnapshotReader for MaterializationSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.snapshot_identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            self.snapshot_identity.clone(),
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

struct MaterializationSink;

impl InvalidationSink for MaterializationSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

struct MaterializationWritebackAuthority;

impl TruthWritebackAuthority for MaterializationWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(TruthWritebackReceipt::new(
            worth_runtime_bridge::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

pub(in crate::runtime::tests::causal_inspection) fn bridge_runtime() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(MaterializationSource)
        .with_source_adapter(MaterializationSource)
        .with_truth_branch_head_source(MaterializationSource)
        .with_signal_sink(MaterializationSink)
        .with_writeback_authority(MaterializationWritebackAuthority)
        .register_source(registered_source(
            "source:causal-materialization-history",
            BridgeTruthViewSelector::historical_commit(
                causal_materialization_branch_identity(),
                causal_materialization_commit_identity(),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .register_structural(registered_structural(
            "structural:causal-materialization-snapshot",
            StructuralFingerprintFamily::TopologyFingerprint,
            StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
                causal_materialization_branch_identity(),
                causal_materialization_snapshot_identity(),
            )),
        ))
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("mapping:causal-materialization"),
            TruthPatchScope::new(
                MappingSelector::exact("entity-1"),
                AspectKeySelector::exact(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native mapping aspect key"),
                ),
                TruthPatchTargetSelector::entity_field(
                    worth_foundational::facade::FieldKey::new("name".to_owned())
                        .expect("valid native mapping field key"),
                ),
            ),
            SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native snapshot aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("signal:profile"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("bridge runtime should build")
}

pub(in crate::runtime::tests::causal_inspection) fn registered_source(
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

pub(in crate::runtime::tests::causal_inspection) fn registered_structural(
    id: &str,
    family: StructuralFingerprintFamily,
    truth_view_basis: StructuralTruthViewBasis,
) -> StructuralIdentityDeclaration {
    StructuralIdentityDeclaration::advisory_remap(
        StructuralIdentityDeclarationIdentity::from_stable_name(id),
        StructuralSchemaIdentity::from_stable_name("schema:causal-materialization"),
        StructuralFingerprintEquivalenceContract::new(
            StructuralSchemaIdentity::from_stable_name("schema:causal-materialization"),
            family,
            "causal-materialization-v1",
            StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
            StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
            StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
        ),
        truth_view_basis,
    )
}
