//! Sample targets addressing staged node surfaces and derived scrollbar parts.
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
        if identity.is_appearance_sample() {
            self.appearance_sample_bounds(identity).is_ok()
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
        if identity.is_appearance_sample() {
            let (bounds, clip) = self.appearance_sample_bounds(identity)?;
            let visible = if change.is_some_and(|change| change.clip().is_some()) {
                Some(bounds)
            } else {
                bounds.intersection(clip)
            };
            return visible.map_or(Ok(None), |bounds| sampled_bounds(bounds, change));
        }
        let command = self
            .commands
            .get(&identity)
            .ok_or(Denial::CommandMismatch)?;
        sampled_visible_bounds(command, change)
    }

    fn appearance_sample_bounds(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<(UiMountedCanonicalBox, UiMountedCanonicalBox), Denial> {
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let key = appearance
            .key_for_identity(
                &self
                    .appearance_sample_identity(identity)
                    .ok_or(Denial::CommandMismatch)?,
            )
            .ok_or(Denial::CommandMismatch)?;
        let (bounds, clip) = match appearance.command(key) {
            Some(UiNativeAppearanceCommand::Surface(surface)) => (
                canonical_visual(surface.visual_bounds()),
                canonical_clip(surface.clip()),
            ),
            Some(UiNativeAppearanceCommand::ScrollChrome(chrome)) => (
                super::appearance_regions::canonical(
                    chrome.rect().x(),
                    chrome.rect().y(),
                    chrome.rect().width(),
                    chrome.rect().height(),
                ),
                canonical_clip(chrome.clip()),
            ),
            _ => return Err(Denial::CommandMismatch),
        };
        Ok((
            bounds.ok_or(Denial::CommandMismatch)?,
            clip.ok_or(Denial::CommandMismatch)?,
        ))
    }

    pub(super) fn appearance_sample_identity(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<UiNativeAppearanceCommandIdentity> {
        if identity.is_appearance_surface() {
            Some(UiNativeAppearanceCommandIdentity::Surface(
                identity.mounted_instance(),
            ))
        } else {
            identity.scroll_chrome_identity().map(|identity| {
                UiNativeAppearanceCommandIdentity::ScrollChrome {
                    surface: self.surface,
                    identity,
                }
            })
        }
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
                Some(UiMountedAppearanceMechanicIdentity::ScrollChrome(identity)) => {
                    Some(UiMountedPaintCommandIdentity::scroll_chrome(*identity))
                }
                _ => None,
            })
            .filter(|identity| {
                !reissued.contains(identity) && self.sample_overrides.contains_key(identity)
            })
            .collect()
    }
}
