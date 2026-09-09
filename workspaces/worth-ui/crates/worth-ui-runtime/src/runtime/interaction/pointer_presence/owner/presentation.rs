use super::*;
use crate::runtime::interaction::targeting::{
    current_pointer_surface, UiInteractionTargetingDenial, UiPresentedPointerPosition,
};

impl UiPointerPresenceOwner {
    /// Returns semantic target changes; zero may still refresh current presentation evidence.
    #[cfg(test)]
    pub(crate) fn retest_committed_presentation(
        &mut self,
        trigger: &super::super::UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<usize, UiInteractionTargetingDenial> {
        let surface = current_pointer_surface(mounted, trigger.presentation())?;
        // Resolve the entire selected pointer set before changing owner state.
        let resolved = self
            .pointers
            .iter()
            .filter(|(_, record)| {
                record.surface == Some(surface)
                    && record.binding == Some(trigger.presentation().binding())
                    && trigger.affects_position(record.position, record.target())
            })
            .map(|(pointer, record)| {
                UiPresentedPointerPosition::resolve(
                    mounted,
                    trigger.presentation(),
                    record.position,
                )
                .map(|position| (*pointer, position))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let target_changes = resolved
            .iter()
            .filter(|(pointer, position)| {
                self.pointers[pointer].target()
                    != position.target().map(|target| target.mounted_instance())
            })
            .count();
        if target_changes != 0 {
            self.bump_revision();
        }
        for (pointer, position) in resolved {
            let record = self
                .pointers
                .get_mut(&pointer)
                .expect("resolved pointer remains owned during presentation refresh");
            record.target = position.target().map(|target| target.view());
            record.presentation = position.presentation();
            // Presentation evidence cannot become a new input sequence or steal primary ownership.
        }
        Ok(target_changes)
    }
}

impl UiPointerPresenceOwner {
    pub(crate) fn refresh_hit_transition(
        &mut self,
        changes: &crate::mounting::UiPresentedHitChanges,
        force_retest: bool,
        neighborhood_work: &mut crate::mounting::UiHitTestSpatialWork,
        presentations: &[(
            UiSemanticSurfaceIdentity,
            UiHostObservationPresentationBasis,
        )],
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<
        crate::runtime::interaction::UiPointerPresentationRefreshReport,
        UiInteractionTargetingDenial,
    > {
        let mut report = crate::runtime::interaction::UiPointerPresentationRefreshReport::default();
        let mut updates = Vec::new();
        for (pointer, record) in &self.pointers {
            let Some((_, presentation)) = presentations.iter().find(|(surface, presentation)| {
                record.surface == Some(*surface) && record.binding == Some(presentation.binding())
            }) else {
                report.unmatched += 1;
                continue;
            };
            current_pointer_surface(mounted, *presentation)?;
            let affected = force_retest
                || crate::runtime::interaction::presentation_refresh::affected(
                    changes,
                    *presentation,
                    record.position,
                    record.target(),
                    neighborhood_work,
                )?;
            let target = if affected {
                report.retested += 1;
                UiPresentedPointerPosition::resolve(mounted, *presentation, record.position)?
                    .target()
                    .map(|target| target.view())
            } else {
                record
                    .target
                    .map(|target| {
                        crate::runtime::interaction::targeting::refresh_pointer_target(
                            mounted,
                            *presentation,
                            target,
                            neighborhood_work,
                        )
                    })
                    .transpose()?
            };
            report.changed +=
                usize::from(target.map(|target| target.mounted_instance()) != record.target());
            report.evidence_refreshed += usize::from(
                *presentation != record.presentation
                    || target.map(|target| target.node_receipt()) != record.node_receipt(),
            );
            updates.push((*pointer, *presentation, target));
        }
        if report.changed != 0 {
            self.bump_revision();
        }
        for (pointer, presentation, target) in updates {
            let record = self
                .pointers
                .get_mut(&pointer)
                .expect("preflight retains pointer ownership");
            record.presentation = presentation;
            record.target = target;
        }
        Ok(report)
    }
}
