//! Public Bridge planning over a named-face runtime: one exact mapping per
//! committed patch item, then the continuity packet a UI selection plans.

use std::sync::Arc;

use worth_foundational::facade::{
    aspects, AspectBinding, AspectIdentity, AspectKey, ScalarAspectType,
};
use worth_relational::facade::config::{CascadeDeletePolicy, RelationalRuntimeProfile};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::DeclaredAspectContractBinding;
use worth_relational::facade::transactions::CommitResult;
use worth_runtime_bridge::facade::*;

/// The CAD runtime whose faces declare a lifecycle aspect, so every face
/// birth and retirement publishes a patch item a selection can hold.
pub fn lifecycle_face_runtime() -> RelationalRuntime {
    runtime_over(
        crate::naming_execution_fixture::cad_schema_registry_with_face_aspects(
            vec![lifecycle_aspect()],
            CascadeDeletePolicy::CascadeDeleteRelations,
        ),
    )
}

/// The same kinds with no persistent name scheme installed.
pub fn nameless_lifecycle_face_runtime() -> RelationalRuntime {
    runtime_over(crate::naming_execution_fixture::cad_kind_registry(
        vec![lifecycle_aspect()],
        CascadeDeletePolicy::CascadeDeleteRelations,
    ))
}

fn lifecycle_aspect() -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::LifecycleTransition,
        contract: aspects()
            .contract()
            .for_key(AspectKey::new("lifecycle").expect("valid aspect key"))
            .identified_by(AspectIdentity(0x11fe_c1c1e))
            .at_revision(aspects().vocabulary().revision(1))
            .scalar(ScalarAspectType::String),
    }
}

fn runtime_over(
    registry: worth_relational::facade::schema::RelationalSchemaRegistry,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("valid lifecycle-face runtime")
}

/// One Bridge source over a named runtime that retains each head it is
/// shown, so a selection over any retained commit plans at the latest head.
pub struct RetainedBridge {
    runtime: Arc<RelationalRuntime>,
    source: RuntimeBridgeRelationalSource,
    leases: Vec<(u64, RelationalBridgeObservationLease)>,
}

/// The planned continuity of every record in one commit's patch: each
/// selection's prior slice and the authority Bridge planned for it.
pub type PlannedContinuity = Vec<(PriorSubscriptionSlice, BridgeHistoricalLineageAuthority)>;

impl RetainedBridge {
    pub fn new(runtime: &Arc<RelationalRuntime>) -> Self {
        let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(runtime), "model")
            .expect("court graph role");
        Self {
            runtime: Arc::clone(runtime),
            source,
            leases: Vec::new(),
        }
    }

    /// Retains the main branch head for Bridge reads.
    pub fn retain_head(&mut self) {
        let branch = self.runtime.main_branch_identity();
        let head = self
            .runtime
            .admit_branch_basis(&branch)
            .expect("head basis")
            .observation()
            .commit_id()
            .expect("a committed head")
            .0;
        let (_, basis) = self
            .source
            .observe_branch_basis(&branch)
            .expect("admitted head basis");
        let lease = self
            .source
            .retain_branch_basis_for_bridge(&basis)
            .expect("retained head observation");
        self.leases.push((head, lease));
    }

    /// The authority Bridge's continuity source answers for `prior` at the
    /// latest retained head.
    pub fn at_head(
        &self,
        prior: &PriorSubscriptionSlice,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> BridgeHistoricalLineageAuthority {
        let (_, head) = self.leases.last().expect("a retained head");
        let basis = BridgeContinuityAuthorityBasis::new(
            TruthBranchIdentity::from_relational_branch_id("main"),
            head.snapshot_identity().clone(),
        );
        self.source
            .historical_lineage(
                BridgeHistoricalLineageRequest::new(basis, prior.clone()),
                execution,
            )
            .expect("continuity source answers at the head")
    }

    /// The continuity Bridge plans for every record in `commit`'s patch at
    /// `commit`'s own retained snapshot.
    pub fn planned_continuity(
        &self,
        commit: &CommitResult,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> PlannedContinuity {
        let (_, head) = self
            .leases
            .iter()
            .find(|(retained, _)| *retained == commit.commit.commit_id.0)
            .expect("the commit was a retained head");
        let commit_identity =
            TruthCommitIdentity::from_relational_commit_id(commit.commit.commit_id.0);
        let envelope = self
            .source
            .load_committed_patch(
                RelationalCommittedPatchRequest::new(commit_identity.clone()),
                execution,
            )
            .expect("committed patch loads");
        let items = envelope.patch_body().canonical_items();
        let first = items.first().expect("a named commit carries patch items");
        let mut builder = RuntimeBridgeBuilder::new()
            .with_relational_source(self.source.clone())
            .with_truth_branch_head_source(self.source.clone())
            .with_signal_sink(CountingSink)
            .with_continuity_lineage_source(self.source.clone())
            .register_mapping(mapping(0, first))
            .register_aspect_mapping(aspect(0, first));
        for (index, item) in items.iter().enumerate().skip(1) {
            builder = builder
                .register_mapping(mapping(index, item))
                .register_aspect_mapping(aspect(index, item));
        }
        let bridge = builder
            .build()
            .expect("bridge builds over the named runtime");
        let context = BridgeMappingContext::default().with_lineage_context(
            BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                TruthBranchIdentity::from_relational_branch_id("main"),
                head.snapshot_identity().clone(),
            )),
        );
        let route = bridge
            .plan_committed_patch_with_mapping_context(
                BridgeRouteRequest::for_commit(commit_identity),
                context,
                execution,
            )
            .expect("lineage-context route plans");
        let requests = bridge
            .plan_continuity_requests_from_planned_route(&route)
            .expect("continuity requests derive");
        let packet = bridge
            .plan_historical_lineage_packet(&requests, execution)
            .expect("historical lineage packet resolves");
        packet
            .entries()
            .iter()
            .map(|entry| {
                (
                    entry.prior_slice().clone(),
                    entry.lineage_authority().clone(),
                )
            })
            .collect()
    }
}

/// Plans `commit`'s continuity on a fresh source retaining only the head,
/// which must be `commit`.
pub fn planned_continuity(
    runtime: &Arc<RelationalRuntime>,
    commit: &CommitResult,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PlannedContinuity {
    let mut bridge = RetainedBridge::new(runtime);
    bridge.retain_head();
    bridge.planned_continuity(commit, execution)
}

fn mapping(index: usize, item: &BridgeCommittedPatchItem) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name(format!("named-item-{index}").as_str()),
        scope(item),
        SnapshotReadContract::scalar(item.aspect_key().clone(), ScalarAspectType::String),
        SignalInvalidationScope::from_stable_name("signal.named.selection"),
        CoarseRoutingMode::Direct,
    )
}

fn aspect(index: usize, item: &BridgeCommittedPatchItem) -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name(format!("named-field-{index}").as_str()),
        scope(item),
        SnapshotReadContract::scalar(item.aspect_key().clone(), ScalarAspectType::String),
        item.surface_kind(),
        slice_kind(item.surface_kind()),
        SliceWideningPolicy::Disallow,
    )
}

fn scope(item: &BridgeCommittedPatchItem) -> TruthPatchScope {
    let target = match item.surface_kind() {
        TruthDeltaSurfaceKind::AuthoritativeAspect => {
            TruthPatchTargetSelector::authoritative_aspect()
        }
        TruthDeltaSurfaceKind::EntityField => TruthPatchTargetSelector::entity_field_path(
            item.field_locator()
                .expect("field item carries a locator")
                .field_path()
                .clone(),
        ),
        TruthDeltaSurfaceKind::EntityRelationEndpoint => {
            TruthPatchTargetSelector::relation_endpoint()
        }
        TruthDeltaSurfaceKind::EntityRegion => TruthPatchTargetSelector::region(),
        TruthDeltaSurfaceKind::EntityPartition => TruthPatchTargetSelector::partition(),
        TruthDeltaSurfaceKind::EntityFacet => TruthPatchTargetSelector::facet(),
        TruthDeltaSurfaceKind::LifecycleTransition => {
            TruthPatchTargetSelector::lifecycle_transition()
        }
    };
    TruthPatchScope::new(
        MappingSelector::exact(item.entity_identity()),
        AspectKeySelector::exact(item.aspect_key().clone()),
        target,
    )
}

fn slice_kind(surface: TruthDeltaSurfaceKind) -> SubscriptionSliceKind {
    match surface {
        TruthDeltaSurfaceKind::AuthoritativeAspect => SubscriptionSliceKind::SignalAspect,
        TruthDeltaSurfaceKind::EntityField => SubscriptionSliceKind::SignalField,
        TruthDeltaSurfaceKind::EntityRelationEndpoint => SubscriptionSliceKind::SignalLens,
        TruthDeltaSurfaceKind::EntityRegion => SubscriptionSliceKind::SignalRegion,
        TruthDeltaSurfaceKind::EntityPartition => SubscriptionSliceKind::SignalPartition,
        TruthDeltaSurfaceKind::EntityFacet => SubscriptionSliceKind::SignalFacet,
        TruthDeltaSurfaceKind::LifecycleTransition => SubscriptionSliceKind::SignalLifecycle,
    }
}

struct CountingSink;

impl InvalidationSink for CountingSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
