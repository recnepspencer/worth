//! Sample targets addressing the staged surface paint of appearance-only instances.
use super::appearance_regions::{canonical_clip, canonical_visual};
use super::sample_transaction::{sampled_bounds, sampled_visible_bounds};
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity,
};
use worth_ui_host_contract::{
    UiMountedAppearanceMechanicIdentity, UiMountedAppearancePresentationWork,
    UiMountedCanonicalBox, UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange,
};

impl UiNativeRetainedDrawList {
    /// A sample may address a retained paint command or the staged surface
    /// paint of an appearance-only instance.
    pub(super) fn admits_sample_target(&self, identity: UiMountedPaintCommandIdentity) -> bool {
        if identity.is_appearance_surface() {
            self.appearance_surface_visible_bounds(identity).is_ok()
        } else {
            self.commands.contains(&identity)
        }
    }

    /// Viewport bounds the target covers under `change`; `None` when hidden.
    pub(super) fn sampled_target_bounds(
        &self,
        identity: UiMountedPaintCommandIdentity,
        change: Option<UiMountedPresentationSampleChange>,
    ) -> Result<Option<UiMountedCanonicalBox>, Denial> {
        if identity.is_appearance_surface() {
            return self
                .appearance_surface_visible_bounds(identity)?
                .map_or(Ok(None), |bounds| sampled_bounds(bounds, change));
        }
        let command = self
            .commands
            .get(&identity)
            .ok_or(Denial::CommandMismatch)?;
        sampled_visible_bounds(command, change)
    }

    fn appearance_surface_visible_bounds(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<Option<UiMountedCanonicalBox>, Denial> {
        if !identity.is_appearance_surface() {
            return Err(Denial::CommandMismatch);
        }
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let key = appearance
            .key_for_identity(&UiNativeAppearanceCommandIdentity::Surface(
                identity.mounted_instance(),
            ))
            .ok_or(Denial::CommandMismatch)?;
        let Some(UiNativeAppearanceCommand::Surface(surface)) = appearance.command(key) else {
            return Err(Denial::CommandMismatch);
        };
        let (Some(visual), Some(clip)) = (
            canonical_visual(surface.visual_bounds()),
            canonical_clip(surface.clip()),
        ) else {
            return Ok(None);
        };
        Ok(visual.intersection(clip))
    }

    /// Retained surface samples whose staged surface mechanic this work
    /// replaces or removes without re-issuing them.
    pub(super) fn retired_appearance_surface_samples(
        &self,
        work: &UiMountedAppearancePresentationWork,
    ) -> Vec<UiMountedPaintCommandIdentity> {
        let reissued = work
            .sample_overrides()
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>();
        work.fragments()
            .iter()
            .flat_map(|fragment| fragment.work().changes())
            .filter_map(|change| match change.identity() {
                Some(UiMountedAppearanceMechanicIdentity::Surface(instance)) => {
                    Some(UiMountedPaintCommandIdentity::appearance_surface(*instance))
                }
                _ => None,
            })
            .filter(|identity| {
                !reissued.contains(identity) && self.sample_overrides.contains_key(identity)
            })
            .collect()
    }
}
