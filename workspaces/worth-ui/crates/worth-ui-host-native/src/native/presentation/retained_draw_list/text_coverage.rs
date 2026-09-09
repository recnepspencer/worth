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

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn apply_text_paint(
        &self,
        command: &worth_ui_host_contract::UiMountedSemanticTextMechanic,
        glyphs: &mut [crate::native::presentation::text::UiNativeGlyphCommand],
        atlas: &UiNativeTextAtlas,
    ) -> Result<(), Denial> {
        if let Some((_, appearance)) = &self.staged_appearance {
            appearance
                .apply_text_paint(command, glyphs, atlas)
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
        let changes = validate_replacement_set(fragment, delta, attempt, &candidates)?;
        for candidate in &candidates {
            if candidate.affinity() != delta.affinity()
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

    fn validate_text_coverage(
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

#[derive(Default)]
struct TextCoverageChanges {
    inserts: std::collections::HashSet<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
    replacements: Vec<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
    removals: Vec<worth_ui_host_contract::UiMountedAppearanceMechanicIdentity>,
}

fn validate_replacement_set(
    fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
    delta: &UiMountedPresentationDelta,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    candidates: &[UiNativeFinalizedTextForeground],
) -> Result<TextCoverageChanges, Denial> {
    use worth_ui_host_contract::{
        UiMountedAppearanceMechanic as Mechanic, UiMountedAppearanceMechanicChange as Change,
        UiMountedAppearanceMechanicIdentity as Identity,
    };
    if fragment.presentation_affinity() != delta.affinity()
        || fragment.work().successor().overlay_order().presentation() != attempt
    {
        return Err(Denial::AffinityMismatch);
    }
    let mut expected = std::collections::HashSet::new();
    let mut text_changes = TextCoverageChanges::default();
    for change in fragment.work().changes() {
        match change {
            Change::Insert(Mechanic::TextForeground(value)) => {
                let identity = text_identity(value);
                if !expected.insert(identity.clone()) || !text_changes.inserts.insert(identity) {
                    return Err(Denial::CommandMismatch);
                }
            }
            Change::Replace {
                predecessor,
                successor: Mechanic::TextForeground(value),
            } => {
                let identity = text_identity(value);
                if predecessor != &identity
                    || !expected.insert(identity.clone())
                    || text_changes.replacements.contains(&identity)
                {
                    return Err(Denial::CommandMismatch);
                }
                text_changes.replacements.push(identity);
            }
            Change::Remove(identity @ Identity::TextForeground { .. }) => {
                text_changes.removals.push(identity.clone());
            }
            _ => {}
        }
    }
    if expected.is_empty() && text_changes.removals.is_empty() {
        return Err(Denial::CommandMismatch);
    }
    for candidate in candidates {
        if candidate.binding() != fragment.surface_binding() {
            return Err(Denial::AffinityMismatch);
        }
        if !candidate.matches_candidates(fragment) {
            return Err(Denial::CommandMismatch);
        }
        if !expected.remove(&text_identity(candidate.mechanic())) {
            return Err(Denial::CommandMismatch);
        }
    }
    if !expected.is_empty() {
        return Err(Denial::CommandMismatch);
    }
    Ok(text_changes)
}

fn stage_text_coverage_changes(
    appearance: &mut UiNativeAppearanceRetained,
    fragment: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
    candidates: &[UiNativeFinalizedTextForeground],
    changes: &TextCoverageChanges,
    atlas: &UiNativeTextAtlas,
) -> Result<Vec<crate::native::presentation::appearance::UiNativeTextCoverageUndo>, Denial> {
    use worth_ui_host_contract::UiMountedAppearanceMechanic as Mechanic;
    let mut undo = Vec::new();
    let staging = (|| {
        for identity in &changes.removals {
            let native = native_text_identity(identity).ok_or(Denial::CommandMismatch)?;
            let key = appearance
                .key_for_identity(&native)
                .ok_or(Denial::CommandMismatch)?;
            undo.push(
                appearance
                    .stage_text_remove(key)
                    .map_err(|_| Denial::CommandMismatch)?,
            );
        }
        for identity in &changes.replacements {
            let candidate = candidate_for(candidates, identity)?;
            undo.push(
                appearance
                    .stage_text_replace(candidate.clone(), atlas)
                    .map_err(|_| Denial::CommandMismatch)?,
            );
        }
        let mut predecessor = None;
        for mechanic in fragment.work().successor().mechanics() {
            let Mechanic::TextForeground(value) = mechanic else {
                continue;
            };
            let identity = text_identity(value);
            if changes.inserts.contains(&identity) {
                let candidate = candidate_for(candidates, &identity)?;
                let (key, insertion) = appearance
                    .stage_text_insert(candidate.clone(), atlas, predecessor)
                    .map_err(|_| Denial::CommandMismatch)?;
                undo.push(insertion);
                predecessor = Some(key);
            } else {
                predecessor = appearance.key_for_identity(
                    &native_text_identity(&identity).ok_or(Denial::CommandMismatch)?,
                );
                if predecessor.is_none() {
                    return Err(Denial::CommandMismatch);
                }
            }
        }
        Ok(())
    })();
    if let Err(denial) = staging {
        for staged in undo.drain(..).rev() {
            appearance
                .rollback_text(staged)
                .expect("partial text coverage staging must restore its predecessor");
        }
        return Err(denial);
    }
    Ok(undo)
}

fn candidate_for<'a>(
    candidates: &'a [UiNativeFinalizedTextForeground],
    identity: &worth_ui_host_contract::UiMountedAppearanceMechanicIdentity,
) -> Result<&'a UiNativeFinalizedTextForeground, Denial> {
    candidates
        .iter()
        .find(|candidate| text_identity(candidate.mechanic()) == *identity)
        .ok_or(Denial::CommandMismatch)
}

fn text_identity(
    value: &worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic,
) -> worth_ui_host_contract::UiMountedAppearanceMechanicIdentity {
    worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::TextForeground {
        target: value.node_receipt().mounted_instance(),
        command: value.command(),
        span: value.paint_span(),
    }
}

fn native_text_identity(
    identity: &worth_ui_host_contract::UiMountedAppearanceMechanicIdentity,
) -> Option<UiNativeAppearanceCommandIdentity> {
    let worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::TextForeground {
        target,
        command,
        span,
    } = identity
    else {
        return None;
    };
    Some(UiNativeAppearanceCommandIdentity::TextForeground {
        target: *target,
        command: command.semantic_text_identity_parts()?,
        span_digest: span.digest(),
    })
}
