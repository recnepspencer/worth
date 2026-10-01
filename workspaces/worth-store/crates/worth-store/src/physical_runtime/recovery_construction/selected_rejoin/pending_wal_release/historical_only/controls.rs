//! Classify all controls still selected at the ordinary tip. Historical V3
//! controls are checked at their addressed candidate roots by the edge walk;
//! unrelated retained failed-ingest controls need an independent payload join.

use std::collections::BTreeSet;

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::VerifiedOrderedHistoricalReleaseCustody;

use super::super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    tier, SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_ENTRIES,
};
use super::{
    budget,
    selection::{self, Selection},
};
use crate::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

pub(super) fn observe(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &Selection,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    remaining_bytes: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let route_peak = budget::route_walk(
        selected.manifest.record_count(),
        selected
            .manifest
            .routing_root()
            .as_ref()
            .map_or(0, |reference| std::mem::size_of_val(reference) as u64),
        remaining_bytes,
    )?;
    let mut discovery = media
        .bounded_discovery(
            MAX_DISCOVERY_ENTRIES,
            remaining_bytes
                .checked_sub(route_peak)
                .ok_or(Denial::BoundExceeded)?,
        )
        .map_err(Denial::Qualification)?;
    if !selection::observe(&mut discovery, reopen, claim)?.same_bytes(selected) {
        return Err(Denial::RootBinding);
    }
    let routes = tier::routes::verify(
        &mut discovery,
        &selected.manifest,
        &selected.free_header,
        reopen.format(),
        None,
    )?;
    let admitted_ids = (claim.released_batches().len() as u64)
        .checked_mul(3)
        .ok_or(Denial::BoundExceeded)?;
    let control_peak = budget::control_decode(
        routes.selected_routes().len() as u64,
        admitted_ids,
        routes.retained_memory_bytes(),
        remaining_bytes,
    )?;
    let max_slices = budget::slice_limit(control_peak, remaining_bytes)?;
    let media = discovery.finish();
    let mut discovery = media
        .bounded_discovery(
            MAX_DISCOVERY_ENTRIES,
            remaining_bytes
                .checked_sub(control_peak)
                .ok_or(Denial::BoundExceeded)?,
        )
        .map_err(Denial::Qualification)?;
    let partition = tier::released_partition::partition_with_slice_limit(
        &mut discovery,
        routes.selected_routes(),
        reopen.format(),
        max_slices,
    )?;
    let mut admitted = BTreeSet::new();
    for batch in claim.released_batches() {
        for record in [
            batch.descriptor_frame().record(),
            batch.reservation_frame().record(),
            batch.manifest_frame().record(),
        ] {
            if !admitted.insert(record) {
                return Err(Denial::CertificateRoster);
            }
        }
    }
    if partition
        .released()
        .keys()
        .any(|record| !admitted.contains(record))
    {
        return Err(Denial::CertificateRoster);
    }
    let reservation_count = partition.reservation_slices().len();
    if reservation_count > max_slices {
        return Err(Denial::BoundExceeded);
    }
    let mut slices: Vec<SelectedArtifactSlice> = Vec::new();
    slices
        .try_reserve_exact(reservation_count)
        .map_err(|_| Denial::BoundExceeded)?;
    slices.extend_from_slice(partition.reservation_slices());
    tier::verify_failed_ingest_subset_bounded(
        &mut discovery,
        partition.failed_ingest(),
        reopen.format(),
        selected.manifest.generation(),
        claim.checkpoint().source().identity().sequence().get(),
        &mut slices,
        max_slices,
    )?;
    let fingerprint = SelectedControlMediaFingerprint::observed(slices);
    (fingerprint.retained_memory_bytes() <= remaining_bytes)
        .then_some((discovery.finish(), fingerprint))
        .ok_or(Denial::BoundExceeded)
}
