//! A declared Bridge adapter supplies authoritative absence for one selected packet.
use super::*;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::*;
use worth_signal::facade::{SignalConditionalArtifactReuse, SignalConditionalVersionComparator};
#[derive(Default)]
pub(crate) struct Contacts {
    pub reads: AtomicUsize,
    pub predicates: AtomicUsize,
    pub computes: AtomicUsize,
}
static NEXT_INSTALLATION: AtomicU64 = AtomicU64::new(1);
pub(crate) fn snapshot() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
}
pub(crate) fn owner(
    contacts: Arc<Contacts>,
    readers: Arc<AtomicUsize>,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    let contract = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(31),
        AspectContractRevision(4),
        StructAspectShape::new([FieldDeclaration::new(
            FieldKey::new("name").unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()])
        .unwrap(),
    );
    let scope = TruthPatchScope::for_entity_field(
        MappingSelector::exact("relational-record:entity:0:1:1"),
        AspectKey::new("profile").unwrap(),
        FieldKey::new("name").unwrap(),
    );
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("profile-name"),
        scope.clone(),
        SnapshotReadContract::new(contract.clone()),
        SignalInvalidationScope::from_stable_name("signal.profile.name"),
        CoarseRoutingMode::Direct,
    );
    let aspect = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name("profile-name"),
        scope,
        SnapshotReadContract::new(contract.clone()),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    let installation = NEXT_INSTALLATION.fetch_add(1, Ordering::Relaxed);
    let source = AbsenceSource {
        contacts: Arc::clone(&contacts),
        readers,
        installation,
    };
    let bridge = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(source.clone())
        .with_snapshot_read_source(source)
        .with_signal_sink(SourceFreePorts)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect)
        .build()
        .unwrap();
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let dependency =
        BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
            source_installation_identity: Arc::from(format!("absence-adapter:{installation}")),
            source_basis: Arc::from("absence-adapter-basis"),
            source_runtime_authority: installation,
            source_installation_generation: installation,
            source_authority_binding_identity: Arc::from(format!("absence-binding:{installation}")),
            source_stage_identity: None,
            source_node_identity: Arc::from("query:one"),
            dependency_ordinal: 0,
            declared_graph_role: Arc::from("model"),
            graph_participation_identity: Arc::from(format!(
                "absence-participation:{installation}"
            )),
            graph_adapter_identity: Arc::from("absence-adapter"),
            source_record_identity: Some(RelationalBridgeRecordIdentityParts::entity(0, 1, 1)),
            observation_record_identity: Some(RelationalBridgeRecordIdentityParts::entity(0, 1, 1)),
            contract,
            projection_mask: AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(
                FieldKey::new("name").unwrap(),
            )]),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("profile").unwrap(),
            },
            locality: BridgeSemanticLocality::SourceRecord,
            relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
        })
        .unwrap();
    let providers = Providers(contacts);
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                identity: Arc::from("query:one"),
                dependency_count: 1,
                condition_dependency_ordinals: Vec::new(),
                condition: BridgeConditionalCondition::RuntimePredicate,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            }),
            location: BridgeConditionalLocation::operation("query:one"),
            dependencies: vec![dependency],
            providers: BridgeConditionalProviderSet::new()
                .condition(providers.clone())
                .compute(providers),
        })
        .unwrap();
    (builder.seal().unwrap(), lowering)
}
#[derive(Clone)]
struct Providers(Arc<Contacts>);
impl BridgeConditionalProviderSemantics for Providers {
    type SemanticContract = ();
    fn semantic_contract(&self) {}
    fn retained_heap_bytes(
        &self,
        _: &(),
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        BridgeConditionalProviderHeapRetention::try_from_parts(
            [BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.0.as_ref())],
            [],
        )
    }
}
impl BridgeConditionalConditionProvider for Providers {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        self.0.predicates.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.observations().len(), 1);
        assert!(context.observations()[0].current().is_none());
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}
impl BridgeConditionalComputeProvider for Providers {
    fn compute(&self, _: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        self.0.computes.fetch_add(1, Ordering::SeqCst);
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(Aspect::new(0), 1)]),
        ))
    }
}
#[derive(Clone)]
struct AbsenceSource {
    contacts: Arc<Contacts>,
    readers: Arc<AtomicUsize>,
    installation: u64,
}
impl CommittedPatchSource for AbsenceSource {
    fn authoritative_source_profile(&self) -> Option<BridgeAuthoritativeSourceProfile> {
        Some(BridgeAuthoritativeSourceProfile::new(self.installation, "absence-adapter").unwrap())
    }
    fn load_committed_patch(
        &self,
        _: RelationalCommittedPatchRequest,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        panic!("readonly observation and reentry must not load a committed patch")
    }
}
impl SnapshotReadSource for AbsenceSource {
    fn open_snapshot(
        &self,
        selected: &TruthSnapshotIdentity,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        assert_eq!(selected, &snapshot());
        self.readers.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(AbsenceReader(self.clone())))
    }
}
struct AbsenceReader(AbsenceSource);
impl TruthSnapshotReader for AbsenceReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        snapshot()
    }
    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.0.contacts.reads.fetch_add(1, Ordering::SeqCst);
        Ok(SnapshotReadPacketResult::new(
            snapshot(),
            packet
                .reads()
                .iter()
                .map(SnapshotReadRecord::absent_for_request)
                .collect(),
        ))
    }
}
impl Drop for AbsenceReader {
    fn drop(&mut self) {
        self.0.readers.fetch_sub(1, Ordering::SeqCst);
    }
}
