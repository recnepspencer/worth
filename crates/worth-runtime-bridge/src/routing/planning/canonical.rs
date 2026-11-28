use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::diagnostics::{BridgeRouteRecordEntityIdentity, BridgeRouteRecordEntry};
use crate::routing::canonicalization::canonical_snapshot_request_order;
use crate::routing::eligibility::EligibleRouteEntry;
use crate::routing::lowering::{BridgeInvalidationTarget, BridgeSubscriptionSlice};
use crate::routing::matching::FineGrainedMatchStatus;
use crate::snapshot::{SnapshotReadPacket, SnapshotReadRequest};

pub(super) fn canonical_invalidation_targets(
    entries: &[EligibleRouteEntry],
) -> Vec<BridgeInvalidationTarget> {
    let mut deduped = BTreeSet::new();
    for entry in entries {
        deduped.insert(BridgeInvalidationTarget::new(
            Arc::<str>::from(entry.registration().signal_scope().as_str()),
            entry.registration().routing_mode(),
            entry.normalized_surface().native_target_basis(),
            entry.normalized_surface().surface_identity().clone(),
        ));
    }

    let mut targets = deduped.into_iter().collect::<Vec<_>>();
    targets.sort();
    targets
}

pub(super) fn canonical_read_packet(
    subscription_slices: &[BridgeSubscriptionSlice],
    entries: &[EligibleRouteEntry],
) -> SnapshotReadPacket {
    if subscription_slices.is_empty() {
        return canonical_coarse_read_packet(entries);
    }

    let mut deduped = BTreeSet::new();
    for slice in subscription_slices {
        deduped.insert(slice.clone());
    }

    let mut reads = deduped
        .into_iter()
        .map(|slice| {
            SnapshotReadRequest::from_native_subscription_slice(
                slice.record().clone(),
                slice.snapshot_read_contract().clone(),
                slice.aspect_locator().clone(),
                slice.field_locator().cloned(),
                slice.projection_mask().clone(),
                slice.slice_kind().clone(),
            )
        })
        .collect::<Vec<_>>();
    reads.sort_by(canonical_snapshot_request_order);
    SnapshotReadPacket::new(reads)
}

fn canonical_coarse_read_packet(entries: &[EligibleRouteEntry]) -> SnapshotReadPacket {
    let mut deduped = BTreeMap::new();
    for entry in entries {
        let contract = entry.registration().snapshot_read_contract().clone();
        deduped.insert(
            (
                entry.item().record().clone(),
                Arc::<str>::from(contract.canonical_basis()),
            ),
            contract,
        );
    }

    let mut reads = deduped
        .into_iter()
        .map(|((record, _), contract)| SnapshotReadRequest::for_record(record, contract))
        .collect::<Vec<_>>();
    reads.sort_by(canonical_snapshot_request_order);
    SnapshotReadPacket::new(reads)
}

pub(super) fn canonical_subscription_slices(
    entries: &[EligibleRouteEntry],
) -> Vec<BridgeSubscriptionSlice> {
    let mut deduped = BTreeSet::new();
    for entry in entries {
        match entry.fine_grained_match().status() {
            FineGrainedMatchStatus::Matched | FineGrainedMatchStatus::WideningAdmitted => {
                let Some(slice_kind) = entry.fine_grained_match().subscription_slice_kind() else {
                    continue;
                };
                let Some(snapshot_read_contract) =
                    entry.fine_grained_match().snapshot_read_contract()
                else {
                    continue;
                };

                deduped.insert(BridgeSubscriptionSlice::from_truth_delta_surface(
                    entry.normalized_surface(),
                    snapshot_read_contract.clone(),
                    slice_kind.clone(),
                    entry.fine_grained_match().status(),
                ));
            }
            FineGrainedMatchStatus::SuppressedByRegistrationPolicy
            | FineGrainedMatchStatus::UnsupportedSurfaceCategory
            | FineGrainedMatchStatus::AmbiguousRegistration => {}
        }
    }

    let mut slices = deduped.into_iter().collect::<Vec<_>>();
    slices.sort();
    slices
}

pub(super) fn canonical_route_record_entries(
    entries: &[EligibleRouteEntry],
) -> Arc<[BridgeRouteRecordEntry]> {
    Arc::from(
        entries
            .iter()
            .map(|entry| {
                BridgeRouteRecordEntry::new(
                    route_record_entity_identity(entry),
                    entry.normalized_surface().aspect_key().clone(),
                    entry.normalized_surface().target().clone(),
                    entry.item().target().clone(),
                    entry.normalized_surface().surface_identity().clone(),
                    entry.registration().mapping_id().clone(),
                    entry.registration().signal_scope().as_str(),
                    entry.registration().routing_mode(),
                    entry.widening_class(),
                    entry.fine_grained_match().clone(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn route_record_entity_identity(entry: &EligibleRouteEntry) -> BridgeRouteRecordEntityIdentity {
    entry
        .normalized_surface()
        .relational_record_identity_parts()
        .map(BridgeRouteRecordEntityIdentity::RelationalRecord)
        .unwrap_or_else(|| {
            BridgeRouteRecordEntityIdentity::TruthSurface(
                entry.normalized_surface().surface_identity().clone(),
            )
        })
}
