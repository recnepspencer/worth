use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::routing::context::BridgeMappingContext;
use crate::routing::eligibility::EligibleRouteEntry;
use crate::routing::lowering::{BridgeInvalidationTarget, BridgeSubscriptionSlice};
use crate::routing::planning::BridgeRouteIdentity;
use crate::snapshot::SnapshotReadRequest;

pub(crate) fn canonical_route_entry_order(
    left: &EligibleRouteEntry,
    right: &EligibleRouteEntry,
) -> std::cmp::Ordering {
    left.normalized_surface()
        .entity_identity()
        .cmp(right.normalized_surface().entity_identity())
        .then_with(|| {
            left.normalized_surface()
                .aspect_key()
                .cmp(right.normalized_surface().aspect_key())
        })
        .then_with(|| {
            left.normalized_surface()
                .surface_identity()
                .cmp(right.normalized_surface().surface_identity())
        })
        .then_with(|| {
            left.registration()
                .registration_identity()
                .cmp(right.registration().registration_identity())
        })
}

pub(crate) fn canonical_snapshot_request_order(
    left: &SnapshotReadRequest,
    right: &SnapshotReadRequest,
) -> std::cmp::Ordering {
    left.entity_identity()
        .cmp(right.entity_identity())
        .then_with(|| left.aspect_key().cmp(right.aspect_key()))
        .then_with(|| left.slice_kind().cmp(&right.slice_kind()))
        .then_with(|| {
            left.target()
                .target_identity()
                .cmp(right.target().target_identity())
        })
        .then_with(|| left.correlation_id().cmp(right.correlation_id()))
}

pub(crate) fn route_digest_basis(
    envelope: &BridgeCommittedPatchEnvelope,
    mapping_context: &BridgeMappingContext,
    entries: &[EligibleRouteEntry],
) -> String {
    let mut basis = format!(
        "route|commit={}|patch={}|snapshot={}|branch={}|mapping-context={}|entry-count={}",
        envelope.commit_identity().as_str(),
        envelope.patch_identity().as_str(),
        envelope.snapshot_identity().as_str(),
        envelope.branch_identity().as_str(),
        mapping_context.digest(),
        entries.len()
    );

    for entry in entries {
        basis.push_str("|entry=");
        basis.push_str(&canonical_route_entry_key(entry));
    }

    basis
}

pub(crate) fn invalidation_digest_basis(
    route_identity: &BridgeRouteIdentity,
    source_commit: &str,
    source_patch: &str,
    source_snapshot: &str,
    targets: &[BridgeInvalidationTarget],
) -> String {
    let mut basis = format!(
        "invalidation|route={}|commit={}|patch={}|snapshot={}|target-count={}",
        route_identity.as_str(),
        source_commit,
        source_patch,
        source_snapshot,
        targets.len()
    );
    for target in targets {
        basis.push_str("|target=");
        basis.push_str(target.target_identity().as_str());
    }
    basis
}

pub(crate) fn subscription_slice_digest_basis(
    source_snapshot: &str,
    slices: &[BridgeSubscriptionSlice],
) -> String {
    let mut basis = format!(
        "subscription-slices|snapshot={}|slice-count={}",
        source_snapshot,
        slices.len()
    );
    for slice in slices {
        basis.push_str("|slice=");
        basis.push_str(slice.canonical_basis());
    }
    basis
}

pub(crate) fn planning_provenance_digest_basis(
    route_identity: &BridgeRouteIdentity,
    envelope: &BridgeCommittedPatchEnvelope,
    mapping_context: &BridgeMappingContext,
    entries: &[EligibleRouteEntry],
    read_packet: &SnapshotReadRequestSetView<'_>,
) -> String {
    let mut basis = format!(
        "planning-provenance|route={}|commit={}|patch={}|snapshot={}|branch={}|patch-digest={}|mapping-context={}|entry-count={}|read-count={}",
        route_identity.as_str(),
        envelope.commit_identity().as_str(),
        envelope.patch_identity().as_str(),
        envelope.snapshot_identity().as_str(),
        envelope.branch_identity().as_str(),
        envelope.digest().as_str(),
        mapping_context.digest(),
        entries.len(),
        read_packet.len(),
    );
    for entry in entries {
        basis.push_str("|entry=");
        basis.push_str(&canonical_route_entry_key(entry));
    }
    for read in read_packet.reads() {
        basis.push_str("|read=");
        basis.push_str(read.correlation_id().as_str());
    }
    basis
}

pub(crate) fn planning_summary_digest_basis(
    route_identity: &BridgeRouteIdentity,
    routing_entry_count: usize,
    invalidation_target_count: usize,
    subscription_slice_count: usize,
    snapshot_read_count: usize,
) -> String {
    format!(
        "planning-summary|route={}|routing-entry-count={}|invalidation-target-count={}|subscription-slice-count={}|snapshot-read-count={}",
        route_identity.as_str(),
        routing_entry_count,
        invalidation_target_count,
        subscription_slice_count,
        snapshot_read_count,
    )
}

pub(crate) fn lowering_provenance_digest_basis(
    route_identity: &BridgeRouteIdentity,
    planning_provenance_digest: &str,
    source_commit: &str,
    source_patch: &str,
    source_snapshot: &str,
) -> String {
    format!(
        "lowering-provenance|route={}|planning={}|commit={}|patch={}|snapshot={}",
        route_identity.as_str(),
        planning_provenance_digest,
        source_commit,
        source_patch,
        source_snapshot,
    )
}

pub(crate) fn lowering_summary_digest_basis(
    route_identity: &BridgeRouteIdentity,
    invalidation_targets: &[BridgeInvalidationTarget],
    subscription_slices: &[BridgeSubscriptionSlice],
    planned_read_count: usize,
) -> String {
    let mut basis = format!(
        "lowering-summary|route={}|target-count={}|slice-count={}|planned-read-count={}",
        route_identity.as_str(),
        invalidation_targets.len(),
        subscription_slices.len(),
        planned_read_count,
    );
    for target in invalidation_targets {
        basis.push_str("|target=");
        basis.push_str(target.target_identity().as_str());
    }
    for slice in subscription_slices {
        basis.push_str("|slice=");
        basis.push_str(slice.canonical_basis());
    }
    basis
}

pub(crate) fn digest_string(kind: &str, basis: &str) -> Arc<str> {
    digest_value(kind, basis)
}

pub(crate) struct SnapshotReadRequestSetView<'a> {
    reads: &'a [SnapshotReadRequest],
}

impl<'a> SnapshotReadRequestSetView<'a> {
    pub(crate) fn new(reads: &'a [SnapshotReadRequest]) -> Self {
        Self { reads }
    }

    pub(crate) fn len(&self) -> usize {
        self.reads.len()
    }

    pub(crate) fn reads(&self) -> &'a [SnapshotReadRequest] {
        self.reads
    }
}

fn canonical_route_entry_key(entry: &EligibleRouteEntry) -> String {
    canonical_route_entry_identity(entry).to_string()
}

fn canonical_route_entry_identity(entry: &EligibleRouteEntry) -> Arc<str> {
    digest_string("route-entry", &canonical_route_entry_basis(entry))
}

fn canonical_route_entry_basis(entry: &EligibleRouteEntry) -> String {
    format!(
        "route-entry|surface-identity={}|mapping-registration={}",
        entry.normalized_surface().surface_identity().as_str(),
        entry.registration().registration_identity().as_str(),
    )
}

fn digest_value(kind: &str, basis: &str) -> Arc<str> {
    let digest = Sha256::digest(basis.as_bytes());
    format!("{kind}:sha256:{digest:x}").into()
}

#[cfg(test)]
#[path = "canonicalization_tests.rs"]
mod canonicalization_tests;
