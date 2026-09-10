use std::collections::HashSet;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedLogicalDamage, UiMountedPaintCommand,
    UiMountedPaintCommandChange, UiMountedPaintCommandIdentity, UiMountedPaintOrderEdit,
    UiMountedPaintOrderIdentity, UiMountedPresentationDelta,
};

use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial};
use crate::native::presentation::damage_index::UiNativeDamageIndex;

#[derive(Default)]
pub(super) struct DeltaMembership {
    inserted: HashSet<UiMountedPaintCommandIdentity>,
    removed: HashSet<UiMountedPaintCommandIdentity>,
}

impl UiNativeRetainedDrawList {
    pub(super) fn validate_affinity(
        &self,
        delta: &UiMountedPresentationDelta,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let affinity = delta.affinity();
        if affinity.predecessor() != Some(self.frame)
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
            || affinity.baseline() != self.baseline
        {
            return Err(UiNativeRetainedDrawListDenial::AffinityMismatch);
        }
        Ok(())
    }

    pub(super) fn validate_changes(
        &self,
        changes: &[UiMountedPaintCommandChange],
    ) -> Result<DeltaMembership, UiNativeRetainedDrawListDenial> {
        let mut membership = DeltaMembership::default();
        let mut seen = HashSet::new();
        for change in changes {
            match change {
                UiMountedPaintCommandChange::Insert(command)
                    if seen.insert(command.identity())
                        && !self.commands.contains(&command.identity()) =>
                {
                    membership.inserted.insert(command.identity());
                }
                UiMountedPaintCommandChange::Replace {
                    predecessor,
                    successor,
                } if seen.insert(*predecessor)
                    && (*predecessor == successor.identity()
                        || seen.insert(successor.identity()))
                    && self.commands.contains(predecessor)
                    && (*predecessor == successor.identity()
                        || !self.commands.contains(&successor.identity())) =>
                {
                    if *predecessor != successor.identity() {
                        membership.removed.insert(*predecessor);
                        membership.inserted.insert(successor.identity());
                    }
                }
                UiMountedPaintCommandChange::Remove(identity)
                    if seen.insert(*identity) && self.commands.contains(identity) =>
                {
                    membership.removed.insert(*identity);
                }
                _ => return Err(UiNativeRetainedDrawListDenial::CommandMismatch),
            }
        }
        Ok(membership)
    }

    pub(super) fn validate_damage(
        &self,
        changes: &[UiMountedPaintCommandChange],
        regions: &[UiMountedLogicalDamage],
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        for change in changes {
            if let UiMountedPaintCommandChange::Insert(command)
            | UiMountedPaintCommandChange::Replace {
                successor: command, ..
            } = change
            {
                if let Some(bounds) = visible_bounds(command) {
                    self.damage.validate_bounds(bounds)?;
                }
            }
        }
        for region in regions {
            self.damage.validate_bounds(region.bounds())?;
        }
        Ok(())
    }

    pub(super) fn validate_order_edits(
        &self,
        edits: &[UiMountedPaintOrderEdit],
        membership: &DeltaMembership,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let mut removals = HashSet::new();
        let mut placements = HashSet::new();
        for edit in edits {
            let identity = edit.identity();
            if edit.is_removal() {
                if !self.order.contains(identity)
                    || !membership.removed.contains(&identity.command())
                    || !removals.insert(identity.command())
                {
                    return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
                }
            } else {
                if !self.final_command_exists(identity.command(), membership)
                    || !placements.insert(identity.command())
                {
                    return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
                }
                if edit.predecessor().is_some_and(|predecessor| {
                    predecessor == identity
                        || !self.final_command_exists(predecessor.command(), membership)
                }) {
                    return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
                }
            }
        }
        if removals != membership.removed
            || !membership
                .inserted
                .iter()
                .all(|identity| placements.contains(identity))
        {
            return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
        }
        Ok(())
    }

    fn final_command_exists(
        &self,
        identity: UiMountedPaintCommandIdentity,
        membership: &DeltaMembership,
    ) -> bool {
        !membership.removed.contains(&identity)
            && (membership.inserted.contains(&identity) || self.commands.contains(&identity))
    }

    pub(super) fn apply_changes(
        &mut self,
        changes: &[UiMountedPaintCommandChange],
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        for change in changes {
            if let UiMountedPaintCommandChange::Remove(identity) = change {
                self.remove(*identity)?;
            }
        }
        for change in changes {
            match change {
                UiMountedPaintCommandChange::Insert(command) => self.insert(command.clone())?,
                UiMountedPaintCommandChange::Replace {
                    predecessor,
                    successor,
                } => self.replace(*predecessor, successor.clone())?,
                UiMountedPaintCommandChange::Remove(_) => {}
            }
        }
        self.apply_glyph_changes(changes, glyph_runs)
    }

    fn apply_glyph_changes(
        &mut self,
        changes: &[UiMountedPaintCommandChange],
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let changed = changes.iter().map(change_identity).collect::<HashSet<_>>();
        if glyph_runs
            .iter()
            .any(|run| !changed.contains(&run.mechanic()))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        for change in changes {
            let identity = change_identity(change);
            match change {
                UiMountedPaintCommandChange::Insert(UiMountedPaintCommand::SemanticText {
                    ..
                })
                | UiMountedPaintCommandChange::Replace {
                    successor: UiMountedPaintCommand::SemanticText { .. },
                    ..
                } => {
                    if let UiMountedPaintCommandChange::Replace { predecessor, .. } = change {
                        if *predecessor != identity {
                            self.glyph_runs.remove(predecessor);
                        }
                    }
                    self.glyph_runs.insert(
                        identity,
                        glyph_runs
                            .iter()
                            .copied()
                            .filter(|run| run.mechanic() == identity)
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    );
                }
                UiMountedPaintCommandChange::Insert(_)
                | UiMountedPaintCommandChange::Replace { .. }
                | UiMountedPaintCommandChange::Remove(_) => {
                    if let UiMountedPaintCommandChange::Replace { predecessor, .. } = change {
                        self.glyph_runs.remove(predecessor);
                    }
                    self.glyph_runs.remove(&identity);
                }
            }
        }
        Ok(())
    }

    fn insert(
        &mut self,
        command: UiMountedPaintCommand,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        if let Some(bounds) = visible_bounds(&command) {
            self.damage.insert(command.identity(), bounds)?;
        }
        self.commands.insert(command.identity(), command);
        Ok(())
    }

    fn replace(
        &mut self,
        predecessor: UiMountedPaintCommandIdentity,
        command: UiMountedPaintCommand,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let identity = command.identity();
        if predecessor != identity {
            self.remove(predecessor)?;
            return self.insert(command);
        }
        let old = self
            .commands
            .get(&identity)
            .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
        update_damage(
            &mut self.damage,
            identity,
            visible_bounds(old),
            visible_bounds(&command),
        )?;
        let semantic_order = command.layer_semantic_order();
        self.commands.insert(identity, command);
        if !self.order.update_weight(
            UiMountedPaintOrderIdentity::for_command(identity),
            semantic_order,
        ) {
            return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
        }
        Ok(())
    }

    fn remove(
        &mut self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let command = self
            .commands
            .remove(&identity)
            .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
        if visible_bounds(&command).is_some() {
            self.damage.remove(identity)?;
        }
        Ok(())
    }

    pub(super) fn apply_order_edits(
        &mut self,
        edits: &[UiMountedPaintOrderEdit],
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        for edit in edits.iter().filter(|edit| edit.is_removal()) {
            let (predecessor, successor) = self.order.neighbors(edit.identity())?;
            self.order_integrity = self
                .order_integrity
                .remove_edge(predecessor, edit.identity(), successor)
                .ok_or(UiNativeRetainedDrawListDenial::OrderMismatch)?;
            self.order.remove(edit.identity())?;
        }
        for edit in edits {
            if !edit.is_removal() {
                if self.order.contains(edit.identity()) {
                    let (predecessor, successor) = self.order.neighbors(edit.identity())?;
                    self.order_integrity = self
                        .order_integrity
                        .remove_edge(predecessor, edit.identity(), successor)
                        .ok_or(UiNativeRetainedDrawListDenial::OrderMismatch)?;
                    self.order.remove(edit.identity())?;
                }
                let successor = self.order.successor_after(edit.predecessor())?;
                self.order_integrity = self
                    .order_integrity
                    .insert_edge(edit.predecessor(), edit.identity(), successor)
                    .ok_or(UiNativeRetainedDrawListDenial::OrderMismatch)?;
                let command = self
                    .commands
                    .get(&edit.identity().command())
                    .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
                self.order.place_after_weighted(
                    edit.identity(),
                    edit.predecessor(),
                    command.layer_semantic_order(),
                )?;
            }
        }
        Ok(())
    }
}

pub(super) fn change_identity(
    change: &UiMountedPaintCommandChange,
) -> UiMountedPaintCommandIdentity {
    match change {
        UiMountedPaintCommandChange::Insert(command)
        | UiMountedPaintCommandChange::Replace {
            successor: command, ..
        } => command.identity(),
        UiMountedPaintCommandChange::Remove(identity) => *identity,
    }
}

pub(super) fn visible_bounds(command: &UiMountedPaintCommand) -> Option<UiMountedCanonicalBox> {
    let bounds = command.bounds();
    let clip = command.clip_bounds();
    if bounds.coordinate_space() != clip.coordinate_space() {
        return None;
    }
    let x = bounds.x().max(clip.x());
    let y = bounds.y().max(clip.y());
    let right = (bounds.x() + bounds.width()).min(clip.x() + clip.width());
    let bottom = (bounds.y() + bounds.height()).min(clip.y() + clip.height());
    UiMountedCanonicalBox::canonicalize(worth_ui_host_contract::UiMountedCanonicalBoxInput {
        x,
        y,
        width: right - x,
        height: bottom - y,
        coordinate_space: bounds.coordinate_space(),
    })
    .ok()
}

pub(super) fn update_damage(
    index: &mut UiNativeDamageIndex<UiMountedPaintCommandIdentity>,
    identity: UiMountedPaintCommandIdentity,
    old: Option<UiMountedCanonicalBox>,
    new: Option<UiMountedCanonicalBox>,
) -> Result<(), UiNativeRetainedDrawListDenial> {
    match (old, new) {
        (Some(_), Some(bounds)) => index.replace(identity, bounds)?,
        (Some(_), None) => index.remove(identity)?,
        (None, Some(bounds)) => index.insert(identity, bounds)?,
        (None, None) => {}
    }
    Ok(())
}
