//! Law 1, reach: injection doors are isolated from ordinary cancellation.
use super::*;
impl RelationalOperationControl {
    pub fn with_injected_interruption_for_test(
        mut self,
        boundary: RelationalInterruptionBoundary,
        interruption: RelationalOperationInterruption,
        trigger_on_visit: usize,
    ) -> Self {
        assert!(
            trigger_on_visit > 0,
            "an interruption seam needs a positive visit"
        );
        self.injected_interruption = Some(RelationalInjectedInterruption {
            boundary,
            interruption,
            trigger_on_visit,
            visits: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        });
        self
    }

    pub fn with_post_linearization_pause_for_test(
        mut self,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.post_linearization_pause = Some(RelationalPostLinearizationPause { reached, release });
        self
    }

    pub fn with_critical_section_pause_for_test(
        mut self,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.critical_section_pause = Some(RelationalCriticalSectionPause { reached, release });
        self
    }

    /// Pause inside the caller-owned publication cutover, the one region that
    /// provably runs while the global patch-position reservation is held.
    /// Holding the reservation there makes a second publisher's contention
    /// deferral a scheduled fact rather than a timing coincidence.
    ///
    /// The handshake is bounded on both sides. The observer is told the seam was
    /// reached over `reached`, so a seam that never runs is a timed-out receive
    /// rather than a stalled run, and the publisher stops waiting once the
    /// observer is gone.
    pub fn with_patch_position_reservation_pause_for_test(
        mut self,
        reached: std::sync::mpsc::SyncSender<()>,
        release: Arc<RelationalPatchPositionReservationGateForTest>,
    ) -> Self {
        self.patch_position_reservation_pause =
            Some(RelationalPatchPositionReservationPause { reached, release });
        self
    }

    pub fn with_boundary_pause_for_test(
        mut self,
        boundary: RelationalInterruptionBoundary,
        reached: Arc<std::sync::Barrier>,
        release: Arc<std::sync::Barrier>,
    ) -> Self {
        self.boundary_pause = Some(RelationalBoundaryPause {
            boundary,
            reached,
            release,
            used: Arc::new(AtomicBool::new(false)),
        });
        self
    }
}
