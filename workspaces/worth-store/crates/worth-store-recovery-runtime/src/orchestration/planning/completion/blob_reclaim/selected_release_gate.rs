//! Selected release authority is the V2 checkpoint-source head roster plus
//! exact V14 post-checkpoint effects. Headless released V1 custody is obsolete.

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use worth_store_physical_format::{
    decode_checkpoint_certificate, BlobRecordKind, CheckpointCertificateKind,
    ReleaseCheckpointCertificateV1, SelectedRecordContentClass,
};

#[path = "selected_release_gate/certificates.rs"]
mod certificates;
#[path = "selected_release_gate/head_v2.rs"]
mod head_v2;
#[path = "selected_release_gate/head_v2_controls.rs"]
mod head_v2_controls;
#[path = "selected_release_gate/pending_wal.rs"]
mod pending_wal;

#[path = "selected_release_gate/resident_basis.rs"]
mod resident_basis;

pub(super) fn admit_pending_wal_release(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    projection_index: usize,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    pending_wal::admit(context, basis, projection_index)
}

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    match checkpoint_posture(&context.selection) {
        Ok(CheckpointReleasePosture::HeadV2) => {
            if let Some(pending) = basis.verified_pending_wal_release_custody.as_ref() {
                let admitted = pending.selected_head_v2().is_some_and(|custody| {
                    custody.selected_root() == context.selection.root().selected().manifest()
                }) && pending_replay_matches(basis)
                    && basis.verified_selected_head_custody_v2.is_none()
                    && basis.verified_selected_checkpoint_custody.is_none()
                    && basis.verified_selected_no_release_custody.is_none();
                return if admitted {
                    Ok(context)
                } else {
                    Err(context.redo_block(basis.planning_counters(), None))
                };
            }
            if basis.verified_selected_head_custody_v2.is_none() {
                context = head_v2::admit(context, basis)?;
            }
            if basis.observed_pages.ordered_releases.is_some() {
                return pending_wal::admit_completed_history(context, basis);
            }
            let custody = basis
                .verified_selected_head_custody_v2
                .as_ref()
                .expect("V2 checkpoint-source custody was joined");
            let selected = context.selection.root().selected().manifest();
            if basis.verified_selected_checkpoint_custody.is_some()
                || basis.verified_selected_no_release_custody.is_some()
                || custody.checkpoint_source_root().release_custody_head_root()
                    != selected.release_custody_head_root()
                || custody
                    .checkpoint_source_root()
                    .next_release_custody_head_block()
                    != selected.next_release_custody_head_block()
            {
                // A newer selected head root requires its complete ordered V14
                // history, not a blessing of checkpoint-source entries.
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            Ok(context)
        }
        Ok(CheckpointReleasePosture::NoRelease) => {
            if let Some(pending) = basis.verified_pending_wal_release_custody.as_ref() {
                if pending.marker().is_none() || !pending_replay_matches(basis) {
                    return Err(context.redo_block(basis.planning_counters(), None));
                }
                return Ok(context);
            }
            if basis.observed_pages.ordered_releases.is_some() {
                return pending_wal::admit_completed_history(context, basis);
            }
            let (next, has_release) = selected_has_release(context, basis)?;
            context = next;
            if has_release {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            let claim = match worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody::claim_selected_no_release(
                &context.selection,
            ) {
                Ok(claim) => claim,
                Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
            };
            basis.verified_selected_no_release_custody = Some(claim);
            Ok(context)
        }
        Ok(CheckpointReleasePosture::Absent) => {
            let (next, has_release) = selected_has_release(context, basis)?;
            if has_release {
                Err(next.redo_block(basis.planning_counters(), None))
            } else {
                Ok(next)
            }
        }
        _ => Err(context.redo_block(basis.planning_counters(), None)),
    }
}

fn pending_replay_matches(basis: &ResolvedPlanningBasis) -> bool {
    match (
        basis.verified_pending_wal_release_custody.as_ref(),
        basis.verified_pending_release_head_replay.as_ref(),
    ) {
        (Some(pending), Some(replay)) => {
            pending.selected_head_replay() == Some(replay)
                && replay.operation() == pending.descriptor().custody().request().idempotency()
        }
        _ => false,
    }
}

#[derive(Clone, Copy)]
enum CheckpointReleasePosture {
    Absent,
    NoRelease,
    HeadV2,
}

fn checkpoint_posture(
    selected: &worth_store_recovery_physics::PhysicalSourceSelection,
) -> Result<CheckpointReleasePosture, ()> {
    let Some(checkpoint) = selected.checkpoint() else {
        return Ok(CheckpointReleasePosture::Absent);
    };
    let mut released = false;
    let mut no_release = false;
    let mut head_v2 = false;
    for frame in checkpoint.checkpoint().certificate_records() {
        let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| ())?;
        if kind != CheckpointCertificateKind::ReleasedDrop {
            continue;
        }
        released = true;
        match ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| ())? {
            ReleaseCheckpointCertificateV1::Batch(_) => {}
            ReleaseCheckpointCertificateV1::AccumulatorV2(_) if !head_v2 => head_v2 = true,
            ReleaseCheckpointCertificateV1::NoRelease(_) if !no_release => no_release = true,
            // V1 released accumulators cannot reconstruct keyed custody.
            _ => return Err(()),
        }
    }
    match (released, no_release, head_v2) {
        (false, false, false) => Ok(CheckpointReleasePosture::Absent),
        (true, true, false) => Ok(CheckpointReleasePosture::NoRelease),
        (true, false, true) => Ok(CheckpointReleasePosture::HeadV2),
        _ => Err(()),
    }
}

fn selected_has_release(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<(PlanningContext, bool), crate::entry::PhysicalRecoveryOutcome> {
    for index in 0..context.selection.page_facts().placements().len() {
        let route = context.selection.page_facts().placements()[index];
        match route.content_class() {
            SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3) => {
                return Ok((context, true))
            }
            SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV2) => {
                let generation = context.selection.root().selected().manifest().generation();
                let (next, bytes) = super::record::selected_record(
                    context,
                    basis,
                    generation,
                    route.record(),
                    worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                )?;
                context = next;
                match worth_store_physical_format::decode_blob_record(&bytes) {
                    Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV2(value))
                        if value.source_kind()
                            == worth_store_physical_format::BlobReclaimSourceKind::FailedIngest => {
                    }
                    Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV2(_)) => {
                        return Ok((context, true))
                    }
                    _ => return Err(context.redo_block(basis.planning_counters(), None)),
                }
            }
            _ => {}
        }
    }
    Ok((context, false))
}
