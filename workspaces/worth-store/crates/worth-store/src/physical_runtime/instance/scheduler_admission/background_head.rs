use std::sync::atomic::{AtomicBool, Ordering};

use worth_store_io_scheduler::{BackgroundPacingOutcome, PhysicalDispatchSelection};

/// One checkpoint or reclamation producer may keep a background head across a
/// temporary pacing result. Capacity shortage and yield do not clear it.
/// A completed quantum or an explicit cancel does.
#[derive(Debug, Default)]
pub(super) struct RetainedBackgroundHeads {
    checkpoint: AtomicBool,
    reclamation: AtomicBool,
}

#[derive(Clone, Copy)]
pub(super) enum BackgroundHeadKind {
    Checkpoint,
    Reclamation,
}

impl RetainedBackgroundHeads {
    fn flag(&self, kind: BackgroundHeadKind) -> &AtomicBool {
        match kind {
            BackgroundHeadKind::Checkpoint => &self.checkpoint,
            BackgroundHeadKind::Reclamation => &self.reclamation,
        }
    }

    pub(super) fn is_retained(&self, kind: BackgroundHeadKind) -> bool {
        self.flag(kind).load(Ordering::Acquire)
    }

    pub(super) fn note(&self, kind: BackgroundHeadKind, dispatch: &PhysicalDispatchSelection) {
        if !self.flag(kind).swap(true, Ordering::AcqRel) {
            dispatch.note_ready_background();
        }
    }

    pub(super) fn commit_quantum(
        &self,
        kind: BackgroundHeadKind,
        dispatch: &PhysicalDispatchSelection,
    ) {
        dispatch.commit_background_quantum();
        self.cancel(kind, dispatch);
    }

    pub(super) fn cancel(&self, kind: BackgroundHeadKind, dispatch: &PhysicalDispatchSelection) {
        if self.flag(kind).swap(false, Ordering::AcqRel) {
            dispatch.release_ready_background();
        }
    }

    pub(super) fn reserve_preservation(
        owner: &super::PhysicalSchedulerAdmissionOwner,
        kind: BackgroundHeadKind,
        lane: worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
        super::RecordSchedulerReservationDenial,
    > {
        // A retained head already owns the owed turn. Another foreground
        // selection turn would block this producer's own retry.
        if owner.heads.is_retained(kind) {
            return owner
                .foreground
                .reserve(lane, &owner.fsync, security)
                .map_err(super::RecordSchedulerReservationDenial::Admission);
        }
        owner.reserve_selected_foreground(lane, &owner.fsync, security)
    }

    pub(super) fn settle(
        &self,
        kind: BackgroundHeadKind,
        dispatch: &PhysicalDispatchSelection,
        pacing: &BackgroundPacingOutcome,
    ) {
        if matches!(pacing, BackgroundPacingOutcome::AdmittedWithDebt(_)) {
            self.commit_quantum(kind, dispatch);
        } else if matches!(
            pacing,
            BackgroundPacingOutcome::Yield(_)
                | BackgroundPacingOutcome::Deferred(_)
                | BackgroundPacingOutcome::Throttled(_)
        ) {
            // The producer is still waiting. The next attempt reuses this head.
        } else {
            self.cancel(kind, dispatch);
        }
    }
}
