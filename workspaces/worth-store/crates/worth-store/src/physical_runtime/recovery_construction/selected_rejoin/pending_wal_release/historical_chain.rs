//! Store's independent bounded rewalk of ordinary edges in the private C.8
//! root chain. This proves no custody by itself; V3 edges and the selected
//! endpoint must also be rejoined before the one-shot seal is issued.

use std::collections::BTreeSet;

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{PhysicalInventoryTranscriptV1, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedHistoricalReleaseRootChain, VerifiedOrdinaryRootStep,
};

use super::super::{
    control_frames::SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{addressed_root, delta, ordinary_member};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
};

const DISCOVERY_HEADROOM: u64 = 64 << 20;
const OPERATION_ROSTER_WIDTH: u64 = 4 * std::mem::size_of::<[u8; 32]>() as u64;

/// The first V3 edge is deliberately excluded here: its exact descriptor,
/// controls, C.9 member and legal drop delta are rejoined by the batch path.
#[allow(clippy::too_many_arguments)]
pub(super) fn verify_ordinary_parts(
    mut media: AdmittedRecoveryFilesystemMedia,
    chain: &VerifiedHistoricalReleaseRootChain,
    checkpoint_generation: u64,
    pending_source_generation: u64,
    pending_source_root_sha256: [u8; 32],
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    pending_operation: [u8; 32],
    pending_lsn_start: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
    retained_peak_bytes: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    verify_global_order(chain, cutoff, pending_operation, pending_lsn_start)?;
    let checkpoint_sha = chain
        .checkpoint_root_frame_sha256()
        .ok_or(Denial::CheckpointBinding)?;
    let mut fingerprint = SelectedControlMediaFingerprint::observed(Vec::new());
    if let Some(prefix) = chain.checkpoint_prefix() {
        if prefix.checkpoint_generation() != checkpoint_generation
            || prefix.checkpoint_root_frame_sha256() != checkpoint_sha
            || prefix.result_topology() != chain.source_topology()
        {
            return Err(Denial::CheckpointBinding);
        }
        let (next_media, prefix_fingerprint) = verify_ordinary_steps(
            media,
            prefix.steps(),
            checkpoint_generation,
            prefix.first_topology(),
            Some(checkpoint_sha),
            chain.source_root_generation(),
            chain.source_topology(),
            Some(chain.source_root_frame_sha256()),
            sample,
            selected_wal,
            cutoff,
            format,
            node_capacity,
            retained_peak_bytes,
        )?;
        media = next_media;
        fingerprint = prefix_fingerprint;
    } else if checkpoint_generation != chain.source_root_generation()
        || checkpoint_sha != chain.source_root_frame_sha256()
    {
        return Err(Denial::CheckpointBinding);
    }
    let suffix = chain.ordinary_steps();
    if chain
        .first_result_generation()
        .checked_add(suffix.len() as u64)
        != Some(pending_source_generation)
        || chain.selected_root_frame_sha256() != pending_source_root_sha256
    {
        return Err(Denial::RootBinding);
    }
    let (media, suffix_fingerprint) = verify_ordinary_steps(
        media,
        suffix,
        chain.first_result_generation(),
        chain.first_result_topology(),
        None,
        pending_source_generation,
        chain.selected_topology(),
        Some(pending_source_root_sha256),
        sample,
        selected_wal,
        cutoff,
        format,
        node_capacity,
        retained_peak_bytes
            .checked_add(fingerprint.retained_memory_bytes())
            .ok_or(Denial::BoundExceeded)?,
    )?;
    if !fingerprint.try_extend_bounded(
        suffix_fingerprint,
        delta::MAX_TRANSITION_MEMORY
            .checked_sub(retained_peak_bytes)
            .ok_or(Denial::BoundExceeded)?,
    ) {
        return Err(Denial::BoundExceeded);
    }
    Ok((media, fingerprint))
}

fn verify_global_order(
    chain: &VerifiedHistoricalReleaseRootChain,
    cutoff: u64,
    pending_operation: [u8; 32],
    pending_lsn_start: u64,
) -> Result<(), Denial> {
    let mut seen = BTreeSet::new();
    let mut previous_end = cutoff;
    let prefix = chain
        .checkpoint_prefix()
        .map_or(&[][..], |prefix| prefix.steps());
    for step in prefix
        .iter()
        .map(|step| (step.operation(), step.lsn_range()))
        .chain(std::iter::once((
            chain.descriptor_operation(),
            Some(chain.first_lsn()),
        )))
        .chain(
            chain
                .ordinary_steps()
                .iter()
                .map(|step| (step.operation(), step.lsn_range())),
        )
    {
        let range = step.1.ok_or(Denial::WalFate)?;
        if !seen.insert(step.0) || range.start().get() < previous_end {
            return Err(Denial::WalFate);
        }
        previous_end = range.end_exclusive().get();
    }
    if !seen.insert(pending_operation) || pending_lsn_start < previous_end {
        return Err(Denial::WalFate);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_ordinary_steps(
    mut media: AdmittedRecoveryFilesystemMedia,
    steps: &[VerifiedOrdinaryRootStep],
    start_generation: u64,
    start_topology: PhysicalInventoryTranscriptV1,
    start_root_sha256: Option<[u8; 32]>,
    end_generation: u64,
    end_topology: PhysicalInventoryTranscriptV1,
    end_root_sha256: Option<[u8; 32]>,
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
    retained_peak_bytes: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    if steps.len() as u64 > MAX_DISCOVERY_ENTRIES
        || start_generation.checked_add(steps.len() as u64) != Some(end_generation)
        || retained_peak_bytes >= delta::MAX_TRANSITION_MEMORY
    {
        return Err(Denial::BoundExceeded);
    }
    let mut fingerprint = SelectedControlMediaFingerprint::observed(Vec::new());
    if steps.is_empty() {
        return (start_topology == end_topology
            && start_root_sha256
                .zip(end_root_sha256)
                .is_none_or(|(a, b)| a == b))
        .then_some((media, fingerprint))
        .ok_or(Denial::RootBinding);
    }
    let roster_charge = (steps.len() as u64)
        .checked_mul(OPERATION_ROSTER_WIDTH)
        .ok_or(Denial::BoundExceeded)?;
    let fixed_charge = retained_peak_bytes
        .checked_add(DISCOVERY_HEADROOM)
        .and_then(|bytes| bytes.checked_add(roster_charge))
        .ok_or(Denial::BoundExceeded)?;
    let mut seen_operations = BTreeSet::new();
    let mut previous_topology = start_topology;
    let mut previous_lsn_end = cutoff;
    for (index, step) in steps.iter().enumerate() {
        let source_generation = start_generation
            .checked_add(index as u64)
            .ok_or(Denial::BoundExceeded)?;
        let result_generation = source_generation
            .checked_add(1)
            .ok_or(Denial::BoundExceeded)?;
        let range = step.lsn_range().ok_or(Denial::WalFate)?;
        if step.source_topology() != previous_topology
            || range.start().get() < previous_lsn_end
            || !seen_operations.insert(step.operation())
        {
            return Err(Denial::WalFate);
        }
        let matched = ordinary_member::match_step(
            step,
            sample,
            selected_wal,
            cutoff,
            delta::MAX_TRANSITION_MEMORY
                .checked_sub(fixed_charge)
                .and_then(|bytes| bytes.checked_sub(fingerprint.retained_memory_bytes()))
                .ok_or(Denial::BoundExceeded)?,
            format,
        )?;
        let mut remaining = delta::MAX_TRANSITION_MEMORY
            .checked_sub(fixed_charge)
            .and_then(|bytes| bytes.checked_sub(matched.retained_scratch_bytes()))
            .and_then(|bytes| bytes.checked_sub(fingerprint.retained_memory_bytes()))
            .ok_or(Denial::BoundExceeded)?;
        let mut discovery = media
            .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
            .map_err(Denial::Qualification)?;
        let source = addressed_root::observe(
            &mut discovery,
            source_generation,
            node_capacity,
            format,
            step.source_topology(),
            (index == 0).then_some(start_root_sha256).flatten(),
        )?;
        let result = addressed_root::observe(
            &mut discovery,
            result_generation,
            node_capacity,
            format,
            step.result_topology(),
            (index + 1 == steps.len())
                .then_some(end_root_sha256)
                .flatten(),
        )?;
        let source_snapshot = delta::snapshot(
            &mut discovery,
            &source.root,
            &source.free,
            format,
            &mut remaining,
        )?;
        let result_snapshot = delta::snapshot(
            &mut discovery,
            &result.root,
            &result.free,
            format,
            &mut remaining,
        )?;
        if source_snapshot.transcript != step.source_topology()
            || result_snapshot.transcript != step.result_topology()
        {
            return Err(Denial::RoutingFrame);
        }
        step.recheck_actual_media(
            ReleasedInventoryView::new(
                &source.root,
                &source.free,
                &source_snapshot.routes,
                &source_snapshot.segments,
                &source_snapshot.free_entries,
            ),
            ReleasedInventoryView::new(
                &result.root,
                &result.free,
                &result_snapshot.routes,
                &result_snapshot.segments,
                &result_snapshot.free_entries,
            ),
            matched.observed(),
            format,
            delta::MAX_TRANSITION_ENTRIES,
            remaining,
        )
        .map_err(|_| Denial::RoutingFrame)?;
        let maximum_fingerprint = delta::MAX_TRANSITION_MEMORY
            .checked_sub(fixed_charge)
            .and_then(|bytes| bytes.checked_sub(matched.retained_scratch_bytes()))
            .ok_or(Denial::BoundExceeded)?;
        if !fingerprint.try_extend_bounded(source_snapshot.fingerprint, maximum_fingerprint)
            || !fingerprint.try_extend_bounded(result_snapshot.fingerprint, maximum_fingerprint)
        {
            return Err(Denial::BoundExceeded);
        }
        previous_topology = step.result_topology();
        previous_lsn_end = range.end_exclusive().get();
        media = discovery.finish();
    }
    (previous_topology == end_topology)
        .then_some((media, fingerprint))
        .ok_or(Denial::RootBinding)
}
