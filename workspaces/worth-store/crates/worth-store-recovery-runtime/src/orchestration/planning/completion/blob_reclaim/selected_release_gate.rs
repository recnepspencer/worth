//! Selected release authority is the V2 checkpoint-source head roster plus
//! exact V14 post-checkpoint effects. Headless released V1 custody is obsolete.

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use crate::progression::PlanningCustody;
use worth_store_physical_format::{
    decode_checkpoint_certificate, BlobRecordKind, CheckpointCertificateKind,
    ReleaseCheckpointCertificateV1, SelectedRecordContentClass,
};

#[path = "selected_release_gate/head_v2.rs"]
mod head_v2;
#[path = "selected_release_gate/head_v2_controls.rs"]
mod head_v2_controls;
#[path = "selected_release_gate/pending_wal.rs"]
mod pending_wal;
#[cfg(test)]
#[path = "selected_release_gate/posture_tests.rs"]
mod posture_tests;

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
    match checkpoint_posture(
        &context.selection,
        context
            .coordination
            .owner()
            .checkpoint()
            .map(|shared| shared.stream()),
    ) {
        Ok(CheckpointReleasePosture::HeadV2) => {
            if let PlanningCustody::PendingPrepared { claim: pending, .. } = &basis.custody {
                let admitted = pending.selected_head_v2().is_some_and(|custody| {
                    custody.selected_root() == context.selection.root().selected().manifest()
                }) && pending_replay_matches(basis);
                return if admitted {
                    Ok(context)
                } else {
                    Err(context.redo_block(basis.planning_counters(), None))
                };
            }
            if matches!(&basis.custody, PlanningCustody::Unresolved) {
                context = head_v2::admit(context, basis)?;
            }
            if basis.observed_pages.ordered_releases.is_some() {
                return pending_wal::admit_completed_history(context, basis);
            }
            let PlanningCustody::SourceHeads(custody) = &basis.custody else {
                return Err(context.redo_block(basis.planning_counters(), None));
            };
            let selected = context.selection.root().selected().manifest();
            if custody.checkpoint_source_root().release_custody_head_root()
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
            if let PlanningCustody::PendingPrepared { claim: pending, .. } = &basis.custody {
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
            let Some(shared) = context.coordination.owner().checkpoint() else {
                return Err(context.redo_block(basis.planning_counters(), None));
            };
            let claim = match worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody::claim_selected_no_release(
                &context.selection,
                shared.stream(),
            ) {
                Ok(claim) => claim,
                Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
            };
            if !matches!(&basis.custody, PlanningCustody::Unresolved) {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            basis.custody = PlanningCustody::NoRelease(claim);
            Ok(context)
        }
        Ok(CheckpointReleasePosture::Absent) => {
            let (next, has_release) = selected_has_release(context, basis)?;
            if has_release {
                Err(next.redo_block(basis.planning_counters(), None))
            } else {
                if !matches!(&basis.custody, PlanningCustody::Unresolved) {
                    return Err(next.redo_block(basis.planning_counters(), None));
                }
                basis.custody = PlanningCustody::NoCheckpoint;
                Ok(next)
            }
        }
        _ => Err(context.redo_block(basis.planning_counters(), None)),
    }
}

fn pending_replay_matches(basis: &ResolvedPlanningBasis) -> bool {
    match &basis.custody {
        PlanningCustody::PendingPrepared {
            claim: pending,
            replay,
        } => {
            pending.selected_head_replay() == Some(replay.head())
                && replay.head().operation()
                    == pending.descriptor().custody().request().idempotency()
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckpointReleasePosture {
    Absent,
    NoRelease,
    HeadV2,
}

fn checkpoint_posture(
    selected: &worth_store_recovery_physics::PhysicalSourceSelection,
    stream: Option<&worth_store_physical_integrity::VerifiedCheckpointStream>,
) -> Result<CheckpointReleasePosture, ()> {
    let Some(checkpoint) = selected.checkpoint() else {
        return if stream.is_none() {
            Ok(CheckpointReleasePosture::Absent)
        } else {
            Err(())
        };
    };
    let stream = stream.ok_or(())?;
    if stream.facts() != *checkpoint.checkpoint() {
        return Err(());
    }
    checkpoint_records_posture(stream.certificate_records())
}

fn checkpoint_records_posture(records: &[Box<[u8]>]) -> Result<CheckpointReleasePosture, ()> {
    let mut released = false;
    let mut no_release = false;
    let mut head_v2 = false;
    let mut batches = false;
    for frame in records {
        let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| ())?;
        if kind != CheckpointCertificateKind::ReleasedDrop {
            continue;
        }
        released = true;
        match ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| ())? {
            ReleaseCheckpointCertificateV1::Batch(_) if !no_release && !head_v2 => batches = true,
            ReleaseCheckpointCertificateV1::AccumulatorV2(_) if !head_v2 => head_v2 = true,
            ReleaseCheckpointCertificateV1::NoRelease(_) if !no_release && !batches => {
                no_release = true
            }
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
