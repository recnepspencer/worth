//! How the interleaving model presents a Motion tick.
//!
//! The host is owed a tick that moves something on the basis it still
//! displays, and a publication the host holds open does not stop it: the
//! tick presents beside it. The runtime holds a tick back only while the host
//! holds open a frame that, once it lands, retires or re-issues the samples
//! of commands on the surface; the tick waits for that frame, and the host
//! takes nothing it was scripted with.

use super::Model;
use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::presentation::motion_sampling::UiPreparedMotionSampling;
use crate::mounting::presentation::UiDisplayedSurfaceBasis;

impl Model {
    /// Whether the host is owed this tick: it moves something, on the basis
    /// the host still displays.
    pub(super) fn host_takes(
        &self,
        prepared: &UiPreparedMotionSampling,
        displayed: UiDisplayedSurfaceBasis,
    ) -> bool {
        !prepared.receipt().samples().is_empty() && self.displayed() == displayed
    }

    /// Present `prepared`, scripting the host with `outcome` when it is owed
    /// the tick.
    pub(super) fn present_tick(
        &mut self,
        prepared: UiPreparedMotionSampling,
        displayed: UiDisplayedSurfaceBasis,
        outcome: fn(&ScriptedPresentationHost),
    ) {
        let owed = self.host_takes(&prepared, displayed);
        if owed {
            outcome(&self.scroll.world.host);
        }
        self.scroll
            .world
            .session
            .present_prepared_motion_tick(prepared, displayed);
        let host = &self.scroll.world.host;
        if owed && host.pending_presentation_count() > 0 {
            assert!(
                !host
                    .held_open_displaced_commands(self.scroll.surface())
                    .is_empty(),
                "a tick waits only for a frame the host holds open that displaces samples"
            );
            host.withdraw_untaken_presentations();
        }
    }
}
