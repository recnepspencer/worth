impl super::UiPointerGestureRuntimeState {
    #[allow(
        dead_code,
        reason = "Gate 1 retains pointer gesture presentation retesting for later mounted cutover"
    )]
    #[cfg(test)]
    pub(crate) fn retest_committed_presentation(
        &mut self,
        trigger: &crate::runtime::interaction::pointer_presence::UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<usize, crate::runtime::interaction::targeting::UiInteractionTargetingDenial> {
        use crate::runtime::interaction::targeting::current_pointer_surface;
        if !self.appearance_enabled || self.active.is_empty() {
            return Ok(0);
        }
        let surface = current_pointer_surface(mounted, trigger.presentation())?;
        let mut updates = Vec::new();
        for (pointer, active) in &self.active {
            if active.target.surface() != surface
                || active.target.binding() != trigger.presentation().binding()
                || !trigger
                    .affects_position(active.position, Some(active.target.mounted_instance()))
            {
                continue;
            }
            let mut appearance = active.appearance;
            if appearance.refresh(
                &active.target,
                trigger.presentation(),
                active.position,
                mounted,
            )? {
                updates.push((*pointer, appearance));
            }
        }
        let changed = updates.len();
        let successor_revision = self
            .appearance_revision
            .checked_add(changed as u64)
            .expect("bounded pressed-appearance revision exhausted");
        for (pointer, appearance) in updates {
            self.active
                .get_mut(&pointer)
                .expect("selected gesture remains owned during presentation refresh")
                .appearance = appearance;
        }
        self.appearance_revision = successor_revision;
        Ok(changed)
    }
}

impl super::UiPointerGestureRuntimeState {
    pub(crate) fn refresh_hit_transition(
        &mut self,
        changes: &crate::mounting::UiPresentedHitChanges,
        force_retest: bool,
        neighborhood_work: &mut crate::mounting::UiHitTestSpatialWork,
        presentations: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiHostObservationPresentationBasis,
        )],
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<
        crate::runtime::interaction::UiPointerPresentationRefreshReport,
        crate::runtime::interaction::UiInteractionTargetingDenial,
    > {
        let mut report = crate::runtime::interaction::UiPointerPresentationRefreshReport::default();
        if !self.appearance_enabled {
            return Ok(report);
        }
        let mut updates = Vec::new();
        for (pointer, active) in &self.active {
            let Some((_, presentation)) = presentations.iter().find(|(surface, presentation)| {
                active.target.surface() == *surface
                    && active.target.binding() == presentation.binding()
            }) else {
                report.unmatched += 1;
                continue;
            };
            crate::runtime::interaction::targeting::current_pointer_surface(
                mounted,
                *presentation,
            )?;
            let mut appearance = active.appearance;
            if force_retest
                || crate::runtime::interaction::presentation_refresh::affected(
                    changes,
                    *presentation,
                    active.position,
                    Some(active.target.mounted_instance()),
                    neighborhood_work,
                )?
            {
                report.retested += 1;
                appearance.refresh(&active.target, *presentation, active.position, mounted)?;
            } else {
                appearance.refresh_evidence(&active.target, *presentation, mounted)?;
            }
            report.changed += usize::from(appearance.inside() != active.appearance.inside());
            report.evidence_refreshed += usize::from(appearance != active.appearance);
            updates.push((*pointer, appearance));
        }
        let revision = self
            .appearance_revision
            .checked_add(report.changed as u64)
            .expect("bounded pressed-appearance revision exhausted");
        for (pointer, appearance) in updates {
            self.active
                .get_mut(&pointer)
                .expect("preflight retains gesture ownership")
                .appearance = appearance;
        }
        self.appearance_revision = revision;
        Ok(report)
    }
}
