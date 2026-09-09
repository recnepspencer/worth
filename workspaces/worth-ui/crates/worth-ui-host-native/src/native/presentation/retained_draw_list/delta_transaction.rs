use worth_ui_host_contract::{
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPresentationDelta,
};

use super::mutation::visible_bounds;
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial, UiNativeRetainedReplayPlan};
use crate::native::presentation::retained_order::UiNativeRetainedOrderSnapshot;

pub(crate) struct UiNativeRetainedDeltaUndo {
    pub(super) physical_coverage: Vec<(
        UiMountedPaintCommandIdentity,
        Option<super::physical_coverage::UiNativeCommandImageCoverage>,
    )>,
    text_coverage: Vec<crate::native::presentation::appearance::UiNativeTextCoverageUndo>,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    content: worth_ui_host_contract::UiMountedContentGeneration,
    commands: Vec<(UiMountedPaintCommandIdentity, Option<UiMountedPaintCommand>)>,
    glyph_runs: Vec<(
        UiMountedPaintCommandIdentity,
        Option<Box<[worth_ui_host_contract::UiGlyphRunView]>>,
    )>,
    order: UiNativeRetainedOrderSnapshot<worth_ui_host_contract::UiMountedPaintOrderIdentity>,
    order_integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity,
    regions: super::super::retained_regions::UiNativeRetainedRegions,
    identity_overlay: super::super::identity_overlay::UiNativeRetainedIdentityOverlay,
    last_paint_attribution: Option<(usize, super::UiNativeRetainedPresentationAttribution)>,
    sample_overrides: Vec<(
        UiMountedPaintCommandIdentity,
        Option<worth_ui_host_contract::UiMountedPresentationSampleChange>,
    )>,
}

impl UiNativeRetainedDrawList {
    pub(crate) fn stage_delta(
        &mut self,
        delta: &UiMountedPresentationDelta,
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<
        (UiNativeRetainedReplayPlan, UiNativeRetainedDeltaUndo),
        UiNativeRetainedDrawListDenial,
    > {
        self.order.take_cost();
        self.validate_affinity(delta)?;
        let membership = self.validate_changes(delta.changes())?;
        self.validate_order_edits(delta.order(), &membership)?;
        self.validate_damage(delta.changes(), delta.damage())?;
        let changed_identities = changed_identities(delta.changes());
        let undo = UiNativeRetainedDeltaUndo {
            physical_coverage: Vec::new(),
            text_coverage: Vec::new(),
            frame: self.frame,
            content: self.content,
            commands: changed_identities
                .iter()
                .copied()
                .map(|identity| (identity, self.commands.get(&identity).cloned()))
                .collect(),
            glyph_runs: changed_identities
                .iter()
                .copied()
                .map(|identity| (identity, self.glyph_runs.get(&identity).cloned()))
                .collect(),
            order: self
                .order
                .snapshot(delta.order().iter().map(|edit| edit.identity())),
            order_integrity: self.order_integrity,
            regions: self.regions.clone(),
            identity_overlay: self.identity_overlay,
            last_paint_attribution: self.last_paint_attribution,
            sample_overrides: changed_identities
                .iter()
                .copied()
                .map(|identity| (identity, self.sample_overrides.get(&identity).copied()))
                .collect(),
        };
        self.retire_sample_overrides_for_semantic_delta(&changed_identities)?;
        if let Err(error) = self
            .apply_changes(delta.changes(), glyph_runs)
            .and_then(|_| self.apply_order_edits(delta.order()))
        {
            self.rollback_delta(undo)
                .expect("a prevalidated retained delta must roll back exactly");
            return Err(error);
        }
        if self.order_integrity != delta.order_integrity() {
            self.rollback_delta(undo)
                .expect("an invalid order receipt must preserve retained state");
            return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
        }
        self.regions.apply_paint_changes(delta.changes());
        if self.regions.apply_node_changes(delta).is_err() {
            self.rollback_delta(undo)
                .expect("an invalid realized-region delta must roll back exactly");
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        let predecessor_identity_overlay = self.identity_overlay;
        let identity_overlay_effect = match self.identity_overlay.apply_delta(delta) {
            Ok(changed) => changed,
            Err(_) => {
                self.rollback_delta(undo)
                    .expect("an invalid identity-overlay delta must roll back exactly");
                return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
            }
        };
        let region_result = delta.auxiliary().map_or(Ok(()), |auxiliary| {
            let projection = auxiliary
                .reconstruct(self.commands.as_map())
                .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)?;
            self.regions
                .replace_hit_tests(&projection)
                .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)?;
            Ok(())
        });
        if let Err(error) = region_result {
            self.rollback_delta(undo)
                .expect("an invalid realized-region delta must roll back exactly");
            return Err(error);
        }
        self.regions
            .rebind_receipt_affinity(delta.affinity().receipt_affinity());
        self.frame = delta.affinity().successor();
        self.content = delta.affinity().content();
        self.retain_current_paint_attribution();
        let mut replay_damage = delta.damage().to_vec();
        let overlay_damage =
            match super::super::identity_overlay::UiNativeRetainedIdentityOverlay::transition_damage(
                predecessor_identity_overlay,
                self.identity_overlay,
            ) {
                Ok(damage) => damage,
                Err(_) => {
                    self.rollback_delta(undo)
                        .expect("invalid overlay damage must roll back exactly");
                    return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
                }
            };
        replay_damage.extend(overlay_damage);
        match self.replay_plan(&replay_damage, delta.changes().len(), delta.order().len()) {
            Ok(mut plan) => {
                plan.identity_overlay_effect = identity_overlay_effect;
                Ok((plan, undo))
            }
            Err(error) => {
                self.rollback_delta(undo)
                    .expect("a prevalidated retained delta must roll back exactly");
                Err(error)
            }
        }
    }

    pub(crate) fn rollback_delta(
        &mut self,
        undo: UiNativeRetainedDeltaUndo,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        self.restore_physical_coverage(undo.physical_coverage)?;
        if !undo.text_coverage.is_empty() {
            let appearance = self
                .staged_appearance
                .as_mut()
                .map(|(_, commands)| commands)
                .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
            for text in undo.text_coverage.into_iter().rev() {
                appearance
                    .rollback_text(text)
                    .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)?;
            }
        }
        for (identity, _) in &undo.commands {
            if let Some(current) = self.commands.remove(identity) {
                if visible_bounds(&current).is_some() {
                    self.damage.remove(*identity)?;
                }
            }
        }
        for (identity, _) in &undo.glyph_runs {
            self.glyph_runs.remove(identity);
        }
        for (identity, runs) in undo.glyph_runs {
            if let Some(runs) = runs {
                self.glyph_runs.insert(identity, runs);
            }
        }
        for (identity, command) in undo.commands {
            let Some(command) = command else {
                continue;
            };
            if let Some(bounds) = visible_bounds(&command) {
                self.damage.insert(identity, bounds)?;
            }
            self.commands.insert(identity, command);
        }
        self.order.restore(undo.order)?;
        self.order_integrity = undo.order_integrity;
        self.regions = undo.regions;
        self.identity_overlay = undo.identity_overlay;
        self.last_paint_attribution = undo.last_paint_attribution;
        self.frame = undo.frame;
        self.content = undo.content;
        for (identity, override_change) in undo.sample_overrides {
            self.sample_overrides.remove(&identity);
            let Some(override_change) = override_change else {
                continue;
            };
            let command = self
                .commands
                .get(&identity)
                .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
            let semantic = visible_bounds(command);
            let sampled =
                super::sample_transaction::sampled_visible_bounds(command, Some(override_change))?;
            super::mutation::update_damage(&mut self.damage, identity, semantic, sampled)?;
            self.sample_overrides.insert(identity, override_change);
        }
        Ok(())
    }
}

pub(super) fn changed_identities(
    changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
) -> Vec<UiMountedPaintCommandIdentity> {
    let mut seen = std::collections::HashSet::new();
    let mut identities = Vec::new();
    let mut include = |identity| {
        if seen.insert(identity) {
            identities.push(identity);
        }
    };
    for change in changes {
        match change {
            worth_ui_host_contract::UiMountedPaintCommandChange::Insert(command) => {
                include(command.identity());
            }
            worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
                predecessor,
                successor,
            } => {
                include(*predecessor);
                include(successor.identity());
            }
            worth_ui_host_contract::UiMountedPaintCommandChange::Remove(identity) => {
                include(*identity);
            }
        }
    }
    identities
}

impl UiNativeRetainedDeltaUndo {
    pub(super) fn retain_text_coverage(
        &mut self,
        undo: crate::native::presentation::appearance::UiNativeTextCoverageUndo,
    ) {
        self.text_coverage.push(undo);
    }
}
