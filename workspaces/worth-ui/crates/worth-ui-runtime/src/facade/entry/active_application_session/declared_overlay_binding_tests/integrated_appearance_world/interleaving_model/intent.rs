//! Where the interleaving model's content must come to rest: where the last
//! intent put it.
//!
//! A notch puts the content at the target its settle travels to, and a page
//! at the offset it places. A rebind pays what the host shows and ends the
//! settle there, so the content rests where it stands once the rebind is
//! done, or where a page it carries places it. A resize the host shows
//! clamps whatever was intended to the travel it leaves, and a settle that
//! arrived beside it is placed there, like a page the next frame shows, since
//! the frame the host showed pulled it back. Nothing else moves
//! the intent: a refused tick or publication, a presentation held open, or a
//! layout staged beside a settle still owes the content the same place.

use super::super::super::scroll_pose_authority::block;
use super::{Model, Step, TRAVELS};

impl Model {
    /// Follow what `effect` intended, once a step has run. `landed` is
    /// whether the step showed a staged resize.
    pub(super) fn follow_intent(&mut self, effect: Option<Step>, landed: bool) {
        let accepted = self.scroll.accepted_offset();
        match effect {
            Some(Step::Notch | Step::Retarget) => {
                self.rests_at = self
                    .scroll
                    .world
                    .session
                    .scroll
                    .as_ref()
                    .and_then(|scroll| {
                        scroll.transition_target(self.scroll.owner, self.scroll.incarnation)
                    })
                    .map_or(accepted, |target| target.target_offset());
            }
            Some(Step::OwnerEdit) => self.rests_at = accepted,
            Some(Step::StageEdit | Step::Rebind) => {
                self.rests_at = self.direct.unwrap_or(accepted);
            }
            _ => {}
        }
        if landed && effect != Some(Step::Rebind) {
            let travel = block(i64::from(TRAVELS[self.travel]));
            if self.rests_at.block_subpixels() > travel.block_subpixels() {
                self.rests_at = travel;
            }
        }
        let surface = self.scroll.surface();
        let placed = self
            .scroll
            .world
            .session
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.has_pending_direct(surface));
        if placed && self.direct.is_none() {
            assert!(
                landed,
                "only a relayout the host shows places a settle it pulled back"
            );
            self.direct = Some(self.rests_at);
        }
    }
}
