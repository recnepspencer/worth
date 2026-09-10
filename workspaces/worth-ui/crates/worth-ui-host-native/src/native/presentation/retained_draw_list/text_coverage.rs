//! Staged appearance coverage participates in the existing presentation delta undo.
use super::{
    UiNativeRetainedDeltaUndo, UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial,
    UiNativeRetainedReplayPlan,
};
use crate::native::presentation::appearance::text_foreground::UiNativeFinalizedTextForeground;
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommandIdentity, UiNativeAppearanceRetained, UiNativeAppearanceScale,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::{UiGlyphRunView, UiMountedPresentationDelta};

#[path = "text_coverage/staging.rs"]
mod staging;
use staging::{same_successor_affinity, stage_text_coverage_changes, validate_replacement_set};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn inherit_text_foreground_replacement(
        &self,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
        mechanic: &worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic,
        atlas: &UiNativeTextAtlas,
    ) -> Result<UiNativeFinalizedTextForeground, Denial> {
        let identity = UiNativeAppearanceCommandIdentity::TextForeground {
            target: mechanic.node_receipt().mounted_instance(),
            command: mechanic
                .command()
                .semantic_text_identity_parts()
                .ok_or(Denial::CommandMismatch)?,
            span_digest: mechanic.paint_span().digest(),
        };
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let key = appearance
            .key_for_identity(&identity)
            .ok_or(Denial::CommandMismatch)?;
        let crate::native::presentation::appearance::UiNativeAppearanceCommand::TextForeground(
            foreground,
        ) = appearance.command(key).ok_or(Denial::CommandMismatch)?
        else {
            return Err(Denial::CommandMismatch);
        };
        foreground
            .inherit_paint_replacement(mechanic, fragment, view, atlas)
            .map_err(|_| Denial::CommandMismatch)
    }

    pub(in crate::native::presentation) fn apply_text_paint(
        &self,
        command: &worth_ui_host_contract::UiMountedSemanticTextMechanic,
        glyphs: &mut [crate::native::presentation::text::UiNativeGlyphCommand],
    ) -> Result<(), Denial> {
        if let Some((_, appearance)) = &self.staged_appearance {
            appearance
                .apply_text_paint(command, glyphs)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        Ok(())
    }

    /// Explicit staging activation; ordinary complete constructors remain inactive.
    pub(crate) fn initialize_text_coverage(
        &mut self,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
    ) -> Result<(), Denial> {
        if self.staged_appearance.is_some() || candidates.is_empty() {
            return Err(Denial::CommandMismatch);
        }
        for candidate in &candidates {
            self.validate_text_coverage(candidate)?;
            candidate
                .validate_images(atlas)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        let binding = candidates[0].binding();
        if candidates
            .iter()
            .any(|candidate| candidate.binding() != binding)
        {
            return Err(Denial::AffinityMismatch);
        }
        let scale =
            u16::try_from(binding.device_scale_milli()).map_err(|_| Denial::AffinityMismatch)?;
        let scale =
            UiNativeAppearanceScale::qualified(scale).map_err(|_| Denial::AffinityMismatch)?;
        let mut appearance = UiNativeAppearanceRetained::new(scale);
        let mut predecessor = None;
        for candidate in candidates {
            let (key, _) = appearance
                .stage_text_insert(candidate, atlas, predecessor)
                .map_err(|_| Denial::CommandMismatch)?;
            predecessor = Some(key);
        }
        appearance.take_damage();
        self.staged_appearance = Some((binding, appearance));
        Ok(())
    }

    pub(crate) fn stage_text_coverage_replacements(
        &mut self,
        delta: &UiMountedPresentationDelta,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        glyph_runs: &[UiGlyphRunView],
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
    ) -> Result<(UiNativeRetainedReplayPlan, UiNativeRetainedDeltaUndo), Denial> {
        if self.staged_appearance.is_none() {
            return Err(Denial::CommandMismatch);
        }
        let changes = validate_replacement_set(fragment, delta.affinity(), attempt, &candidates)?;
        for candidate in &candidates {
            if !same_successor_affinity(candidate.affinity(), delta.affinity())
                || candidate.presentation_attempt() != attempt
            {
                return Err(Denial::AffinityMismatch);
            }
            candidate
                .validate_images(atlas)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        let basis = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .basis;
        let (mut replay, mut undo) = self.stage_delta(delta, glyph_runs)?;
        if let Err(denial) =
            self.refresh_physical_delta(delta, &mut undo, basis, atlas, &mut replay)
        {
            self.rollback_delta(undo)
                .expect("physical preparation refusal preserves predecessor");
            return Err(denial);
        }
        let coverage_result = stage_text_coverage_changes(
            &mut self
                .staged_appearance
                .as_mut()
                .expect("admitted staged coverage owner")
                .1,
            fragment,
            &candidates,
            &changes,
            atlas,
        );
        let coverage_undo = match coverage_result {
            Ok(value) => value,
            Err(denial) => {
                self.rollback_delta(undo)
                    .expect("reverse transaction undo preserves predecessor");
                return Err(denial);
            }
        };
        for text in coverage_undo {
            undo.retain_text_coverage(text);
        }
        let regions = self
            .staged_appearance
            .as_mut()
            .expect("admitted staged coverage owner")
            .1
            .prepare_replay();
        match regions {
            Ok(regions) => replay.staged_appearance_regions = regions,
            Err(_) => {
                self.rollback_delta(undo)
                    .expect("replay preparation refusal preserves predecessor");
                return Err(Denial::CommandMismatch);
            }
        }
        Ok((replay, undo))
    }

    pub(crate) fn stage_text_coverage_fragment_after_delta(
        &mut self,
        delta: &UiMountedPresentationDelta,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
        undo: &mut UiNativeRetainedDeltaUndo,
    ) -> Result<(), Denial> {
        self.stage_text_coverage_fragment(
            delta.affinity(),
            fragment,
            attempt,
            candidates,
            atlas,
            undo,
        )
    }

    pub(in crate::native::presentation) fn stage_text_coverage_fragment_after_unchanged(
        &mut self,
        unchanged: &worth_ui_host_contract::UiMountedPresentationUnchanged,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
        undo: &mut super::UiNativeRetainedUnchangedUndo,
    ) -> Result<(), Denial> {
        let mut staged = Vec::new();
        self.stage_text_coverage_fragment_commands(
            unchanged.affinity(),
            fragment,
            attempt,
            candidates,
            atlas,
            &mut staged,
        )?;
        for command in staged {
            Self::retain_unchanged_appearance(undo, command);
        }
        Ok(())
    }

    fn stage_text_coverage_fragment(
        &mut self,
        affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
        undo: &mut UiNativeRetainedDeltaUndo,
    ) -> Result<(), Denial> {
        let mut staged = Vec::new();
        self.stage_text_coverage_fragment_commands(
            affinity,
            fragment,
            attempt,
            candidates,
            atlas,
            &mut staged,
        )?;
        for text in staged {
            undo.retain_text_coverage(text);
        }
        Ok(())
    }

    fn stage_text_coverage_fragment_commands(
        &mut self,
        affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
        fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
        staged: &mut Vec<crate::native::presentation::appearance::UiNativeTextCoverageUndo>,
    ) -> Result<(), Denial> {
        let changes = validate_replacement_set(fragment, affinity, attempt, &candidates)?;
        for candidate in &candidates {
            if !same_successor_affinity(candidate.affinity(), affinity)
                || candidate.presentation_attempt() != attempt
            {
                return Err(Denial::AffinityMismatch);
            }
            candidate
                .validate_images(atlas)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        staged.extend(stage_text_coverage_changes(
            &mut self
                .staged_appearance
                .as_mut()
                .ok_or(Denial::CommandMismatch)?
                .1,
            fragment,
            &candidates,
            &changes,
            atlas,
        )?);
        Ok(())
    }

    pub(crate) fn prepare_appearance_replay(
        &mut self,
    ) -> Result<
        Box<[crate::native::presentation::appearance::UiNativeAppearanceReplayRegion]>,
        Denial,
    > {
        self.staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?
            .1
            .prepare_replay()
            .map_err(|_| Denial::CommandMismatch)
    }

    pub(super) fn validate_text_coverage(
        &self,
        candidate: &UiNativeFinalizedTextForeground,
    ) -> Result<(), Denial> {
        let binding = candidate.binding();
        let affinity = candidate.affinity();
        if self
            .staged_appearance
            .as_ref()
            .is_some_and(|(current, _)| *current != binding)
        {
            return Err(Denial::AffinityMismatch);
        }
        if binding.semantic_surface() != self.surface
            || binding.binding() != self.binding
            || binding.baseline() != self.baseline
            || affinity.successor() != self.frame
            || affinity.content() != self.content
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
        {
            return Err(Denial::AffinityMismatch);
        }
        for identity in candidate.candidate_commands() {
            let Some(worth_ui_host_contract::UiMountedPaintCommand::SemanticText {
                mechanic, ..
            }) = self.command(identity)
            else {
                return Err(Denial::CommandMismatch);
            };
            candidate
                .validate_command(mechanic)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        Ok(())
    }
}
