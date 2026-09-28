use crate::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SliceWideningPolicy,
    SnapshotReadContract, SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope,
    TruthPatchTargetSelector,
};
use worth_foundational::facade::{AspectKey, ScalarAspectType};
use worth_relational::facade::config::CascadeDeletePolicy;

use super::super::RuntimeBridgeRelationalSource;

pub(super) fn bridge_envelopes_at_current_observation(
    runtime: worth_relational::facade::runtime::RelationalRuntime,
    commit_ids: impl IntoIterator<Item = worth_relational::facade::history::CommitId>,
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
                .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
                    TruthCommitIdentity::from_relational_commit_id(commit_id.0),
                    _lease.snapshot_identity().clone(),
                ))
                .expect("exact observed Bridge publication")
        })
        .collect()
}

pub(super) struct TestSink;

impl InvalidationSink for TestSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::facade::BridgeSignalInvalidationDelivery,
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

pub(super) fn runtime_with_test_schema() -> worth_relational::facade::runtime::RelationalRuntime {
    crate::relational_source::relational_test_support::runtime_with_declared_aspect_schema(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
}

/// The ancestry work of one full walk from `head` down a linear history of
/// `N` commits: `N` node visits, `N` catalog probes, and `N - 1` parent edges.
///
/// Counted from the head's ancestor closure, not from Relational's own
/// selection, so an adapter that walks twice or walks less is caught.
pub(super) fn linear_ancestry_work(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    head: worth_relational::facade::history::CommitId,
) -> usize {
    let commits = runtime
        .history()
        .ancestor_closure_by_commit_id_order(head)
        .len();
    3 * commits - 1
}
