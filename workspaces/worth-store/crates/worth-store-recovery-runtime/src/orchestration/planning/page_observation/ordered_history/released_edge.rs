//! Admit one completed V3 root edge from its exact C.9 member and addressed
//! source/result media. The head replay precedes the inventory transition;
//! the resulting edge then binds that same owned replay without another read.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{PersistedRecordIdentity, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, AdmittedRootStepMemberView, OrderedRootHistoryBuilder,
    ReleasedInventoryView, VerifiedOrderedReleasedHeadReplayV14,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory::ResidentAllowance,
};
use crate::progression::{verified_historical_release_transition, RecoverySelectedSourceInventory};

use super::walk_failure::{Verdict, WalkFailure, WalkFailure::Unverified};
use super::{
    controls, dropped_bounded, head_replay, manifest_retained_bytes, OrderedReleasedObservation,
};

#[derive(Default)]
pub(super) struct ReleasedRetention {
    pub(super) manifest_bytes: u64,
    pub(super) control_bytes: u64,
    pub(super) peak_scratch: u64,
}

impl ReleasedRetention {
    pub(super) fn prior_bytes(&self, roster_capacity: usize) -> Option<u64> {
        self.manifest_bytes
            .checked_add(self.control_bytes)?
            .checked_add(
                u64::try_from(roster_capacity)
                    .ok()?
                    .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)?,
            )
    }
}

pub(super) struct ReleasedStep<'member, 'media> {
    pub(super) member: AdmittedRootStepMemberView<'member>,
    pub(super) redo: &'member AdmittedPhysicalRedoMembers,
    pub(super) source: ReleasedInventoryView<'media>,
    pub(super) result: ReleasedInventoryView<'media>,
    pub(super) source_inventory: &'media RecoverySelectedSourceInventory,
    pub(super) result_inventory: &'media RecoverySelectedSourceInventory,
    pub(super) format: PhysicalRecordFormatDeclaration,
    pub(super) maximum_entries: u64,
    pub(super) staging: RecoveryAllowance,
    pub(super) live_inventory: u64,
    pub(super) input_scratch: u64,
    pub(super) retained_before: u64,
}

pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    step: ReleasedStep<'_, '_>,
    history: &mut OrderedRootHistoryBuilder,
    releases: &mut Vec<OrderedReleasedObservation>,
    retained: &mut ReleasedRetention,
) -> Result<(), WalkFailure> {
    let member = step.member;
    let source_root = step.source.root;
    let result_root = step.result.root;
    let format = step.format;
    let maximum = step.staging;
    let trace_backing = trace.owned_heap_bytes().proven()?;
    let control_base = held([
        step.live_inventory,
        step.input_scratch,
        step.retained_before,
        trace_backing,
    ])?;
    let mut control_resident = ResidentAllowance::new(WalkFailure::left(maximum, control_base)?);
    let binding = match member.materialization().operation() {
        worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            ..
        } => binding,
        _ => return Err(Unverified),
    };
    let controls = controls::released_controls(
        discovery,
        step.result.routes,
        step.redo,
        member.operation(),
        binding.record(),
        binding.record_payload_sha256(),
        format,
        budget,
        trace,
        &mut control_resident,
        maximum,
    )?;
    let mut control_live = control_resident.used();
    retained.peak_scratch = retained
        .peak_scratch
        .max(control_base.checked_add(control_resident.peak()).proven()?);
    let descriptor = controls.descriptor;
    let manifest = &controls.manifest;
    if descriptor.base().source_root_generation() != source_root.generation()
        || descriptor.base().candidate_root_generation() != result_root.generation()
        || descriptor.custody().source_root_frame_sha256()
            != <[u8; 32]>::from(Sha256::digest(source_root.encode(format)))
    {
        return Err(Unverified);
    }
    let next_manifest_bytes = retained
        .manifest_bytes
        .checked_add(manifest_retained_bytes(manifest).proven()?)
        .proven()?;
    let next_roster = u64::try_from(releases.len().checked_add(1).proven()?)
        .proven()?
        .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)
        .proven()?;
    let prospective_retained = next_manifest_bytes
        .checked_add(next_roster)
        .proven()?
        .checked_add(retained.control_bytes)
        .proven()?;
    let dropped_held = held([
        step.live_inventory,
        step.input_scratch,
        prospective_retained,
        trace_backing,
        control_live,
    ])?;
    let dropped_limit = WalkFailure::left(maximum, dropped_held)?;
    let derived = match member.materialization().operation() {
        worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } => retirement.dropped_records(),
        _ => &[],
    };
    let dropped = dropped_bounded(manifest.dropped(), derived, budget, dropped_limit, maximum)?;
    let dropped_bytes = u64::try_from(dropped.capacity())
        .proven()?
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)
        .proven()?;
    let head_limit = WalkFailure::take(maximum, dropped_limit, dropped_bytes)?;

    let mut directory_resident = ResidentAllowance::new(head_limit);
    let directory_replacement = match member.materialization().operation() {
        worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
            directory_replacement: Some(replacement),
            ..
        } => {
            let route = step
                .source
                .routes
                .iter()
                .find(|route| route.record() == replacement.expected_previous().directory_record())
                .copied()
                .proven()?;
            let read = crate::orchestration::planning::released_directory::read(
                discovery,
                route,
                step.source_inventory,
                format,
                budget,
                trace,
                &mut directory_resident,
            );
            let bytes = read.map_err(|denial| {
                WalkFailure::unread(denial, budget, &directory_resident, maximum)
            })?;
            directory_resident.transient(
                worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement::maximum_decode_heap_bytes(),
            ).map_err(|_| WalkFailure::resident(&directory_resident, maximum))?;
            let proof = worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement::admit(
                &member,
                source_root,
                route,
                &bytes,
                &dropped,
                format,
            )
            .proven()?;
            let backing = bytes.capacity() as u64;
            drop(bytes);
            directory_resident.release(backing).proven()?;
            Some(proof)
        }
        _ => None,
    };
    retained.peak_scratch = retained.peak_scratch.max(
        maximum
            .admitted()
            .checked_sub(head_limit)
            .proven()?
            .checked_add(directory_resident.peak())
            .proven()?,
    );
    let directory_trace_growth = directory_resident.used();
    control_resident
        .bytes(directory_trace_growth)
        .map_err(|_| WalkFailure::resident(&control_resident, maximum))?;
    control_live = control_live.checked_add(directory_trace_growth).proven()?;
    let head_limit = WalkFailure::take(maximum, head_limit, directory_trace_growth)?;

    // The addressed reader's observed frame and its owned Vec copy coexist.
    // Debit that window with the controls and dropped vector still resident.
    let read_overlap = u64::from(format.page_size().bytes())
        .checked_mul(2)
        .proven()?;
    let added_retained = prospective_retained.saturating_sub(step.retained_before);
    control_resident
        .transient(
            added_retained
                .checked_add(dropped_bytes)
                .proven()?
                .checked_add(read_overlap)
                .proven()?,
        )
        .map_err(|_| WalkFailure::resident(&control_resident, maximum))?;
    let selected_replay = head_replay::admit_addressed_member(
        discovery,
        budget,
        member,
        source_root,
        result_root,
        format,
        head_limit,
        step.staging,
    )?;
    let head_retained = selected_replay
        .owned_heap_bytes()
        .proven()?
        .checked_add(std::mem::size_of::<VerifiedOrderedReleasedHeadReplayV14>() as u64)
        .proven()?;
    let matcher_limit = WalkFailure::take(maximum, head_limit, head_retained)?;
    let (transition, adapter_scratch) = verified_historical_release_transition(
        source_root,
        step.source_inventory,
        step.source.routes,
        result_root,
        step.result_inventory,
        step.result.routes,
        &dropped,
        member.materialization().placements(),
        Some(&selected_replay),
        directory_replacement.as_ref(),
        format,
        step.maximum_entries,
        matcher_limit,
    )
    .map_err(|exceeded| WalkFailure::refused(exceeded, budget, step.staging))?;
    let matcher_peak = step
        .input_scratch
        .checked_add(dropped_bytes)
        .proven()?
        .checked_add(adapter_scratch)
        .proven()?
        .checked_add(transition.scratch_bytes())
        .proven()?
        .checked_add(step.live_inventory)
        .proven()?
        .checked_add(prospective_retained)
        .proven()?
        .checked_add(trace_backing)
        .proven()?
        .checked_add(control_live)
        .proven()?
        .checked_add(head_retained)
        .proven()?;
    WalkFailure::hold(maximum, matcher_peak)?;
    retained.peak_scratch = retained.peak_scratch.max(matcher_peak);

    // A Vec growth can hold both its old backing and its replacement until
    // the reserve completes. Keep the prior roster in the live charge.
    let roster_growth = next_roster
        .checked_add(
            u64::try_from(releases.capacity())
                .proven()?
                .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)
                .proven()?,
        )
        .proven()?;
    let reserve_live = matcher_peak
        .checked_sub(adapter_scratch)
        .proven()?
        .checked_add(roster_growth)
        .proven()?;
    WalkFailure::hold(maximum, reserve_live)?;
    releases.try_reserve_exact(1).proven()?;
    let roster_bytes = u64::try_from(releases.capacity())
        .proven()?
        .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)
        .proven()?;
    let bound_live = matcher_peak
        .checked_sub(adapter_scratch)
        .proven()?
        .checked_sub(next_roster)
        .proven()?
        .checked_add(roster_bytes)
        .proven()?;
    WalkFailure::hold(maximum, bound_live)?;
    let edge = history
        .advance_released(member, transition, result_root, step.result.free, format)
        .map_err(|denial| WalkFailure::refused(denial.exceeded_bound(), budget, step.staging))?;
    let bind_remaining = WalkFailure::left(maximum, bound_live)?;
    let head_replay = head_replay::bind_edge(
        edge,
        selected_replay,
        source_root,
        result_root,
        format,
        bind_remaining,
        step.staging,
    )?;
    let addressed_remaining = WalkFailure::left(maximum, bound_live)?;
    let addressed = controls::bind_addressed(
        edge,
        step.result,
        &controls,
        format,
        step.maximum_entries,
        addressed_remaining,
        maximum,
    )?;
    let next_control_bytes = addressed.retained_bytes;
    let post_bind_peak = bound_live.checked_add(next_control_bytes).proven()?;
    WalkFailure::hold(maximum, post_bind_peak)?;
    retained.peak_scratch = retained.peak_scratch.max(post_bind_peak);
    retained.control_bytes = retained
        .control_bytes
        .checked_add(next_control_bytes)
        .proven()?
        .checked_add(head_retained)
        .proven()?;
    retained.manifest_bytes = next_manifest_bytes;
    releases.push(OrderedReleasedObservation {
        operation: member.operation(),
        descriptor,
        manifest: controls.manifest,
        candidate_root_generation: result_root.generation(),
        descriptor_frame: addressed.descriptor_frame,
        reservation_frame: addressed.reservation_frame,
        manifest_frame: addressed.manifest_frame,
        head_replay,
    });
    Ok(())
}

/// What the walk holds at once. A sum past every count is no limit.
fn held<const N: usize>(parts: [u64; N]) -> Result<u64, WalkFailure> {
    parts
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
        .ok_or(WalkFailure::CountOverflow)
}
