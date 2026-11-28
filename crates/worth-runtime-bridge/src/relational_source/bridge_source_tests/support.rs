use crate::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SliceWideningPolicy,
    SnapshotReadContract, SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope,
    TruthPatchTargetSelector,
};
use crate::facade::{
    RelationalBridgeBranchHeadLease, RelationalBridgeObservationLease, TruthCommitIdentity,
};
use crate::relational_source::relational_test_support::{
    aspect_key, create_entity_outcome_on_branch, fork_branch, AspectSchemaFixture,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, ScalarAspectType};
use worth_relational::facade::{
    config::CascadeDeletePolicy, history::BranchId, runtime::RelationalRuntime,
};

use super::super::RuntimeBridgeRelationalSource;

pub(super) fn bridge_envelopes_at_current_observation(
    runtime: worth_relational::facade::runtime::RelationalRuntime,
    commit_ids: impl IntoIterator<Item = worth_relational::facade::history::CommitId>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<BridgeCommittedPatchEnvelope> {
    use crate::facade::{
        CommittedPatchSource, RelationalCommittedPatchRequest, TruthCommitIdentity,
    };

    let branch = runtime
        .publication()
        .latest_bundle()
        .expect("Bridge publication fixture requires a committed branch")
        .commit
        .branch_id
        .clone();
    let identity = runtime
        .branch_identity(&branch)
        .expect("Bridge publication fixture branch identity");
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(std::sync::Arc::new(runtime), "model")
            .expect("Bridge publication fixture graph role");
    let (_, basis) = source
        .observe_branch_basis(&identity)
        .expect("Bridge publication fixture exact basis");
    let _lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("Bridge publication fixture retained observation");
    commit_ids
        .into_iter()
        .map(|commit_id| {
            source
                .load_committed_patch(
                    RelationalCommittedPatchRequest::at_snapshot(
                        TruthCommitIdentity::from_relational_commit_id(commit_id.0),
                        _lease.snapshot_identity().clone(),
                    ),
                    execution,
                )
                .expect("exact observed Bridge publication")
        })
        .collect()
}

pub(super) struct TestSink;

impl InvalidationSink for TestSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(super) fn exact_registration(
    mapping_id: &str,
    patch_item: &BridgeCommittedPatchItem,
) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name(mapping_id),
        truth_scope_for_patch_item(patch_item),
        snapshot_read_contract(patch_item.aspect_key()),
        SignalInvalidationScope::from_stable_name("signal.user.profile"),
        CoarseRoutingMode::Direct,
    )
}

pub(super) fn exact_aspect_registration(
    registration_id: &str,
    patch_item: &BridgeCommittedPatchItem,
) -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name(registration_id),
        truth_scope_for_patch_item(patch_item),
        snapshot_read_contract(patch_item.aspect_key()),
        patch_item.surface_kind(),
        subscription_slice_kind_for_patch_item(patch_item),
        SliceWideningPolicy::Disallow,
    )
}

pub(super) fn runtime_bridge_for_envelope(
    source: RuntimeBridgeRelationalSource,
    envelope: &BridgeCommittedPatchEnvelope,
) -> crate::facade::RuntimeBridge {
    let patch_item = envelope
        .patch_body()
        .canonical_items()
        .first()
        .expect("lineage publication fixture must carry one native patch item");
    let mut builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source.clone())
        .with_signal_sink(TestSink)
        .with_continuity_lineage_source(source)
        .register_mapping(exact_registration("lineage-publication-item-0", patch_item))
        .register_aspect_mapping(exact_aspect_registration(
            "lineage-publication-item-field-0",
            patch_item,
        ));
    register_remaining_patch_items!(builder, envelope, "lineage-publication");
    builder
        .build()
        .expect("runtime bridge should build from runtime-backed relational source")
}

macro_rules! register_remaining_patch_items {
    ($builder:ident, $envelope:expr, $mapping_prefix:literal) => {
        for (index, patch_item) in $envelope
            .patch_body()
            .canonical_items()
            .iter()
            .enumerate()
            .skip(1)
        {
            $builder = $builder
                .register_mapping(crate::relational_source::bridge_source_tests::support::exact_registration(
                    &format!("{}-item-{index}", $mapping_prefix),
                    patch_item,
                ))
                .register_aspect_mapping(crate::relational_source::bridge_source_tests::support::exact_aspect_registration(
                    &format!("{}-item-field-{index}", $mapping_prefix),
                    patch_item,
                ));
        }
    };
}

pub(super) use register_remaining_patch_items;

fn truth_scope_for_patch_item(patch_item: &BridgeCommittedPatchItem) -> TruthPatchScope {
    TruthPatchScope::new(
        MappingSelector::exact(patch_item.entity_identity()),
        AspectKeySelector::exact(patch_item.aspect_key().clone()),
        target_selector_for_patch_item(patch_item),
    )
}

fn target_selector_for_patch_item(
    patch_item: &BridgeCommittedPatchItem,
) -> TruthPatchTargetSelector {
    match patch_item.surface_kind() {
        TruthDeltaSurfaceKind::AuthoritativeAspect => {
            TruthPatchTargetSelector::authoritative_aspect()
        }
        TruthDeltaSurfaceKind::EntityField => patch_item
            .field_locator()
            .map(|locator| {
                TruthPatchTargetSelector::entity_field_path(locator.field_path().clone())
            })
            .expect("entity field patch item must carry a field locator"),
        TruthDeltaSurfaceKind::EntityRelationEndpoint => {
            TruthPatchTargetSelector::relation_endpoint()
        }
        TruthDeltaSurfaceKind::EntityRegion => TruthPatchTargetSelector::region(),
        TruthDeltaSurfaceKind::EntityPartition => TruthPatchTargetSelector::partition(),
        TruthDeltaSurfaceKind::EntityFacet => TruthPatchTargetSelector::facet(),
        TruthDeltaSurfaceKind::LifecycleTransition => {
            TruthPatchTargetSelector::lifecycle_transition()
        }
    }
}

fn subscription_slice_kind_for_patch_item(
    patch_item: &BridgeCommittedPatchItem,
) -> SubscriptionSliceKind {
    match patch_item.surface_kind() {
        TruthDeltaSurfaceKind::AuthoritativeAspect => SubscriptionSliceKind::SignalAspect,
        TruthDeltaSurfaceKind::EntityField => SubscriptionSliceKind::SignalField,
        TruthDeltaSurfaceKind::EntityRelationEndpoint => SubscriptionSliceKind::SignalLens,
        TruthDeltaSurfaceKind::EntityRegion => SubscriptionSliceKind::SignalRegion,
        TruthDeltaSurfaceKind::EntityPartition => SubscriptionSliceKind::SignalPartition,
        TruthDeltaSurfaceKind::EntityFacet => SubscriptionSliceKind::SignalFacet,
        TruthDeltaSurfaceKind::LifecycleTransition => SubscriptionSliceKind::SignalLifecycle,
    }
}

fn snapshot_read_contract(aspect_key: &AspectKey) -> SnapshotReadContract {
    SnapshotReadContract::scalar(aspect_key.clone(), ScalarAspectType::String)
}

pub(super) use crate::relational_source::relational_test_support::{
    linear_ancestry_work, runtime_with_test_schema,
};

#[derive(Clone)]
pub struct RelationalFixture {
    pub runtime: Arc<RelationalRuntime>,
    pub source: RuntimeBridgeRelationalSource,
}

impl RelationalFixture {
    pub fn attach(runtime: RelationalRuntime, role: &str) -> Self {
        let runtime = Arc::new(runtime);
        let source = RuntimeBridgeRelationalSource::for_graph_role(runtime.clone(), role)
            .expect("fixture graph role");
        Self { runtime, source }
    }
}

impl std::fmt::Debug for RelationalFixture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.source, formatter)
    }
}

pub(crate) fn empty_bridge_relational_source() -> RuntimeBridgeRelationalSource {
    RelationalFixture::attach(AspectSchemaFixture::default().build_runtime(), "model").source
}

pub struct NamedTruth {
    truth: Arc<RelationalRuntime>,
    pub source: RuntimeBridgeRelationalSource,
}

impl NamedTruth {
    pub fn native(&self) -> RelationalFixture {
        RelationalFixture {
            runtime: self.truth.clone(),
            source: self.source.clone(),
        }
    }

    pub fn scenario_on_branch(
        branch: &str,
        name: &str,
    ) -> (
        crate::harness::fixtures::ScenarioRelationalTruth,
        crate::facade::TruthCommitIdentity,
    ) {
        let truth = Self::new();
        let serial = crate::snapshot::test_serial_request();
        let execution = ExecutionRequest::serial(&serial);
        let (_base, _observation) = truth.commit_name(execution, "base");
        truth.fork_main(execution, branch);
        let (commit, observation) = truth.commit_name_on_branch(execution, branch, name);
        let head = truth.bind_head(branch);
        (
            crate::harness::fixtures::ScenarioRelationalTruth::new(
                truth.native(),
                commit.clone(),
                vec![observation],
                Some(head),
            ),
            commit,
        )
    }

    pub fn new() -> Self {
        let mut schema = AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        );
        schema
            .entity_aspects
            .retain(|binding| binding.aspect_key().as_str() == "name");
        let native = RelationalFixture::attach(schema.build_runtime(), "model");
        Self {
            truth: native.runtime,
            source: native.source,
        }
    }

    pub(crate) fn commit_initial_name(
        &self,
        name: &str,
    ) -> (TruthCommitIdentity, RelationalBridgeObservationLease) {
        let serial = crate::snapshot::test_serial_request();
        self.commit_name(ExecutionRequest::serial(&serial), name)
    }

    pub fn commit_name(
        &self,
        execution: ExecutionRequest<'_, '_>,
        name: &str,
    ) -> (TruthCommitIdentity, RelationalBridgeObservationLease) {
        self.commit_name_on_branch(execution, "main", name)
    }

    pub fn fork_main(&self, execution: ExecutionRequest<'_, '_>, branch: &str) {
        fork_branch(
            &execution,
            &self.truth,
            BranchId(branch.to_owned()),
            &BranchId("main".to_owned()),
        );
    }

    pub fn commit_name_on_branch(
        &self,
        execution: ExecutionRequest<'_, '_>,
        branch: &str,
        name: &str,
    ) -> (TruthCommitIdentity, RelationalBridgeObservationLease) {
        let branch = BranchId(branch.to_owned());
        let outcome =
            create_entity_outcome_on_branch(&execution, &self.truth, name, branch.clone());
        let branch = self.truth.branch_identity(&branch).unwrap();
        let (_, basis) = self.source.observe_branch_basis(&branch).unwrap();
        (
            TruthCommitIdentity::from_relational_commit_id(outcome.commit.commit_id.0),
            self.source.retain_branch_basis_for_bridge(&basis).unwrap(),
        )
    }

    pub fn bind_main_head(&self) -> RelationalBridgeBranchHeadLease {
        self.bind_head("main")
    }

    pub fn bind_head(&self, branch: &str) -> RelationalBridgeBranchHeadLease {
        let branch = self
            .truth
            .branch_identity(&BranchId(branch.to_owned()))
            .unwrap();
        let (_, basis) = self.source.observe_branch_basis(&branch).unwrap();
        self.source
            .bind_branch_head_basis_for_bridge(&basis)
            .unwrap()
    }

    pub fn mapping() -> BridgeMappingRegistration {
        Self::mapping_for(SignalInvalidationScope::from_stable_name("signal.name"))
    }

    pub fn mapping_for(scope: SignalInvalidationScope) -> BridgeMappingRegistration {
        let name = aspect_key("name");
        BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("named-truth"),
            TruthPatchScope::for_target(
                MappingSelector::any(),
                name.clone(),
                TruthPatchTargetSelector::authoritative_aspect(),
            ),
            SnapshotReadContract::scalar(name, ScalarAspectType::String),
            scope,
            CoarseRoutingMode::Direct,
        )
    }

    pub fn aspect_mapping() -> BridgeAspectRegistration {
        let mapping = Self::mapping();
        BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::from_stable_name("named-truth-aspect"),
            mapping.truth_scope().clone(),
            mapping.snapshot_read_contract().clone(),
            TruthDeltaSurfaceKind::AuthoritativeAspect,
            SubscriptionSliceKind::SignalAspect,
            SliceWideningPolicy::Disallow,
        )
    }
}
