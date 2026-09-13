use worth_ui_host_contract::{
    UiHostSurfacePresentationMode, UiMountedEffectFamily, UiMountedPresentationWorkView,
};

use super::{super::UiMountedPresentationWork, UiMountedPresentationState};

impl UiMountedPresentationState {
    pub(in crate::mounting::presentation) fn expected_completion_effects(
        &self,
        predecessor: Option<&Self>,
        work: &UiMountedPresentationWork,
        mode: UiHostSurfacePresentationMode,
    ) -> Vec<UiMountedEffectFamily> {
        if mode == UiHostSurfacePresentationMode::RecordOnly {
            return vec![UiMountedEffectFamily::RecordedProjection];
        }
        let mut effects = match work.view() {
            UiMountedPresentationWorkView::Initial(_)
            | UiMountedPresentationWorkView::Reconstruction(_) => {
                let mut effects = self.effects.to_vec();
                effects.push(UiMountedEffectFamily::NativePaint);
                effects.sort();
                effects.dedup();
                effects
            }
            UiMountedPresentationWorkView::Delta(delta) => {
                let mut effects = Vec::new();
                if !delta.changes().is_empty()
                    || !delta.order().is_empty()
                    || !delta.damage().is_empty()
                {
                    effects.push(UiMountedEffectFamily::NativePaint);
                }
                if overlay_changed(
                    self,
                    predecessor,
                    delta.auxiliary().is_some() || !delta.nodes().is_empty(),
                ) {
                    effects.push(UiMountedEffectFamily::IdentityOverlay);
                }
                effects
            }
            UiMountedPresentationWorkView::Sample(sample) => (!sample.changes().is_empty()
                || !sample.damage().is_empty())
            .then_some(UiMountedEffectFamily::NativePaint)
            .into_iter()
            .collect(),
            UiMountedPresentationWorkView::Unchanged(_) => Vec::new(),
        };
        if work.appearance().is_some_and(|appearance| {
            appearance.fragments().iter().any(|fragment| {
                !fragment.work().damage().is_empty()
                    || fragment.work().text_damage_requirements().next().is_some()
            })
        }) && !effects.contains(&UiMountedEffectFamily::NativePaint)
        {
            effects.push(UiMountedEffectFamily::NativePaint);
            effects.sort();
        }
        effects
    }
}

fn overlay_changed(
    successor: &UiMountedPresentationState,
    predecessor: Option<&UiMountedPresentationState>,
    presentation_changed: bool,
) -> bool {
    presentation_changed
        && predecessor.is_some_and(|prior| {
            prior
                .effects
                .contains(&UiMountedEffectFamily::IdentityOverlay)
                || successor
                    .effects
                    .contains(&UiMountedEffectFamily::IdentityOverlay)
        })
}
