//! Leading presented hit rows to Scroll samples a witness displays while
//! mounted geometry cannot yet commit them.
//!
//! A presentation attempt in flight, or geometry no witness has shown staged
//! over the surface, holds a displayed sample's settle owed rather than
//! applied. The host already shows the content where the sample put it, and a
//! pointer reaches what the host shows, so the hit rows lead there until the
//! settle is paid, and return to committed geometry once nothing is owed.

impl super::WorthUiMountedSessionState {
    /// Lead the presented hit rows of every surface in `poses` to the offset
    /// its witness displays, without committing any geometry, and report the
    /// hit transitions that crossing left behind.
    ///
    /// `shown` is where the rows of the frame the host shows committed each
    /// owner. Mounted geometry can be staged past it, so a lead measures from
    /// it, and an owner staged past it that no sample moves stands there.
    pub(crate) fn lead_presented_scroll_hits(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::presentation::UiDisplayedScrollOffset,
        )],
        shown: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<
        Box<[crate::mounting::UiCommittedPresentedHitTransition]>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        let mut settled = self.displayed_scroll_poses(poses)?;
        let unsampled = shown
            .iter()
            .filter(|(surface, owner, _)| {
                !settled
                    .iter()
                    .any(|(settled, moved, _)| settled == surface && moved == owner)
            })
            .copied()
            .collect::<Vec<_>>();
        settled.extend(unsampled);
        let prepared = self.prepare_scroll_poses(&settled, shown)?;
        let mut transitions = Vec::new();
        for pose in prepared {
            // One witness displays every sample of a surface.
            let Some(displayed) = poses
                .iter()
                .find(|(surface, _, _)| *surface == pose.surface())
                .map(|(_, _, offset)| offset.displayed_basis())
            else {
                continue;
            };
            let (transition, _) = self.retention.lead_presented_hit_scroll(
                displayed,
                pose.surface(),
                pose.translations(),
            );
            transitions.extend(transition);
        }
        Ok(transitions.into_boxed_slice())
    }

    /// Retire the leads the hit rows of `surface` hold, in its current frame
    /// and in the frame `displayed` names, once no settle is owed there or no
    /// lead can follow the sample its witness shows. Report the hit
    /// transitions that crossing left behind.
    pub(crate) fn retire_presented_scroll_leads(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
    ) -> Box<[crate::mounting::UiCommittedPresentedHitTransition]> {
        self.retention
            .retire_presented_hit_scroll_leads(surface, displayed.basis().frame())
            .into_boxed_slice()
    }
}
