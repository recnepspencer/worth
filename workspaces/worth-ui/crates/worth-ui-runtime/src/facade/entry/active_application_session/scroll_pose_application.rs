//! Committing a scroll pose, and re-resolving what the pointer is over.
//!
//! Three things move a region's content: a wheel that routes its whole delta
//! at once, a thumb or track that places an offset outright, and a settle
//! paying out an accepted sample. All three end in the same place, applying a
//! prepared pose to mounted geometry, and all three move content that a reader
//! may be resting a pointer on.
//!
//! A pointer that did not move is still over something different afterwards.
//! The host will not say so -- it reports pointers, and no pointer event
//! happened -- so the frame that moves the content is the frame that has to
//! re-resolve what is under it. Waiting for the next publication would leave
//! one frame in which the row the reader sees under the cursor is not the row
//! a click would reach, and would leave hover appearance painted on a row that
//! has already travelled away.
//!
//! The evidence is the pose itself. Mounted geometry displaces the presented
//! hit rows by the same translations it displaces the displayed geometry by,
//! and hands back the hit transitions that crossing produced; this file is
//! where those transitions reach interaction. Hover therefore resolves against
//! the accepted offset that produced the pixels, never against the target the
//! content is still travelling toward.

impl super::super::WorthUiActiveApplicationSession {
    /// Apply one frame's scroll poses and re-resolve pointer affordance and
    /// hover appearance against where the content now is.
    ///
    /// The re-resolution runs only after the pose has landed, because a
    /// refused pose moves nothing and must leave the pointer resolved against
    /// the content it was already over.
    pub(in crate::facade::entry) fn apply_scroll_poses(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        let transitions = self.mounted.apply_scroll_geometries(poses)?;
        for transition in transitions.iter() {
            self.interaction
                .observe_presented_hit_transition(transition, &self.mounted);
        }
        Ok(())
    }

    /// Presented hit-index maintenance done for the last applied scroll pose,
    /// which is what re-resolving hover under moving content cost.
    pub fn last_scroll_hit_index_work(&self) -> crate::facade::mounted::UiHitTestSpatialWork {
        self.mounted.last_scroll_hit_index_work()
    }
}
