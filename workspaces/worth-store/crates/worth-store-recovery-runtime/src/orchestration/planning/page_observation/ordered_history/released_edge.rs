//! Admit one completed V3 root edge from its exact C.9 member and addressed
//! source/result media. The head replay precedes the inventory transition;
//! the resulting edge then binds that same owned replay without another read.

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
    pub(super) maximum_scratch_bytes: u64,
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
) -> Option<()> {
    let member = step.member;
    let source_root = step.source.root;
    let result_root = step.result.root;
    let format = step.format;
    let trace_backing = trace.owned_heap_bytes()?;
    let control_base = step
        .live_inventory
        .checked_add(step.input_scratch)?
        .checked_add(step.retained_before)?
        .checked_add(trace_backing)?;
    let mut control_resident =
        ResidentAllowance::new(step.maximum_scratch_bytes.checked_sub(control_base)?);
    let binding = match member.materialization().blob_semantic() {
        worth_store_physical_format::PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(
            binding,
        ) => binding,
        _ => return None,
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
    )?;
    let control_live = control_resident.used();
    retained.peak_scratch = retained
        .peak_scratch
        .max(control_base.checked_add(control_resident.peak())?);
    let descriptor = controls.descriptor;
    let manifest = &controls.manifest;
    if descriptor.base().source_root_generation() != source_root.generation()
        || descriptor.base().candidate_root_generation() != result_root.generation()
        || descriptor.custody().source_root_frame_sha256()
            != <[u8; 32]>::from(Sha256::digest(source_root.encode(format)))
    {
        return None;
    }
    let next_manifest_bytes = retained
        .manifest_bytes
        .checked_add(manifest_retained_bytes(manifest)?)?;
    let next_roster = u64::try_from(releases.len().checked_add(1)?)
        .ok()?
        .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)?;
    let prospective_retained = next_manifest_bytes
        .checked_add(next_roster)?
        .checked_add(retained.control_bytes)?;
    let dropped_limit = step
        .maximum_scratch_bytes
        .checked_sub(step.live_inventory)?
        .checked_sub(step.input_scratch)?
        .checked_sub(prospective_retained)?
        .checked_sub(trace_backing)?
        .checked_sub(control_live)?;
    let derived = member
        .materialization()
        .derived_retirement()
        .map_or(&[][..], |retirement| retirement.dropped_records());
    let dropped = dropped_bounded(
        manifest.dropped(),
        derived,
        step.maximum_entries,
        dropped_limit,
    )?;
    let dropped_bytes = u64::try_from(dropped.capacity())
        .ok()?
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)?;
    let head_limit = dropped_limit.checked_sub(dropped_bytes)?;

    // The addressed reader's observed frame and its owned Vec copy coexist.
    // Debit that window with the controls and dropped vector still resident.
    let read_overlap = u64::from(format.page_size().bytes()).checked_mul(2)?;
    let added_retained = prospective_retained.saturating_sub(step.retained_before);
    control_resident
        .transient(
            added_retained
                .checked_add(dropped_bytes)?
                .checked_add(read_overlap)?,
        )
        .ok()?;
    let selected_replay = head_replay::admit_addressed_member(
        discovery,
        budget,
        member,
        source_root,
        result_root,
        format,
        head_limit,
    )?;
    let head_retained = selected_replay
        .owned_heap_bytes()?
        .checked_add(std::mem::size_of::<VerifiedOrderedReleasedHeadReplayV14>() as u64)?;
    let matcher_limit = head_limit.checked_sub(head_retained)?;
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
        format,
        step.maximum_entries,
        matcher_limit,
    )?;
    let matcher_peak = step
        .input_scratch
        .checked_add(dropped_bytes)?
        .checked_add(adapter_scratch)?
        .checked_add(transition.scratch_bytes())?
        .checked_add(step.live_inventory)?
        .checked_add(prospective_retained)?
        .checked_add(trace_backing)?
        .checked_add(control_live)?
        .checked_add(head_retained)?;
    if matcher_peak > step.maximum_scratch_bytes {
        return None;
    }
    retained.peak_scratch = retained.peak_scratch.max(matcher_peak);

    // A Vec growth can hold both its old backing and its replacement until
    // the reserve completes. Keep the prior roster in the live charge.
    let roster_growth = next_roster.checked_add(
        u64::try_from(releases.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)?,
    )?;
    let reserve_live = matcher_peak
        .checked_sub(adapter_scratch)?
        .checked_add(roster_growth)?;
    if reserve_live > step.maximum_scratch_bytes {
        return None;
    }
    releases.try_reserve_exact(1).ok()?;
    let roster_bytes = u64::try_from(releases.capacity())
        .ok()?
        .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)?;
    let bound_live = matcher_peak
        .checked_sub(adapter_scratch)?
        .checked_sub(next_roster)?
        .checked_add(roster_bytes)?;
    if bound_live > step.maximum_scratch_bytes {
        return None;
    }
    let edge = history
        .advance_released(member, transition, result_root, step.result.free, format)
        .ok()?;
    let bind_remaining = step.maximum_scratch_bytes.checked_sub(bound_live)?;
    let head_replay = head_replay::bind_edge(
        edge,
        selected_replay,
        source_root,
        result_root,
        format,
        bind_remaining,
    )?;
    let addressed_remaining = step.maximum_scratch_bytes.checked_sub(bound_live)?;
    let addressed = controls::bind_addressed(
        edge,
        step.result,
        &controls,
        format,
        step.maximum_entries,
        addressed_remaining,
    )?;
    let next_control_bytes = addressed.retained_bytes;
    let post_bind_peak = bound_live.checked_add(next_control_bytes)?;
    if post_bind_peak > step.maximum_scratch_bytes {
        return None;
    }
    retained.peak_scratch = retained.peak_scratch.max(post_bind_peak);
    retained.control_bytes = retained
        .control_bytes
        .checked_add(next_control_bytes)?
        .checked_add(head_retained)?;
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
    Some(())
}
