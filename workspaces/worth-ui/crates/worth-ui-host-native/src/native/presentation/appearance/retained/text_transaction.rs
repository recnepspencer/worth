//! Reversible text coverage mutations consumed by presentation transaction undo.
use super::*;
use crate::native::presentation::appearance::text_foreground::UiNativeFinalizedTextForeground;
use crate::native::presentation::retained_order::UiNativeRetainedOrderSnapshot;
use crate::native::text_atlas::UiNativeTextAtlas;

pub(crate) struct UiNativeAppearanceCommandUndo {
    key: UiNativeAppearanceCommandKey,
    previous: Option<UiNativeAppearanceCommand>,
    bounds: Option<UiNativeAppearanceDamageRect>,
    order: UiNativeRetainedOrderSnapshot<UiNativeAppearanceCommandKey>,
    damage: UiNativeAppearanceDamage,
}

pub(crate) type UiNativeTextCoverageUndo = UiNativeAppearanceCommandUndo;

impl UiNativeAppearanceRetained {
    pub(crate) fn stage_text_insert(
        &mut self,
        successor: UiNativeFinalizedTextForeground,
        atlas: &UiNativeTextAtlas,
        predecessor: Option<UiNativeAppearanceCommandKey>,
    ) -> Result<
        (UiNativeAppearanceCommandKey, UiNativeTextCoverageUndo),
        UiNativeAppearanceRetainedDenial,
    > {
        successor
            .validate_images(atlas)
            .map_err(|_| UiNativeAppearanceRetainedDenial::StaleTextImages)?;
        let key = UiNativeAppearanceCommandKey::new(self.next_key);
        let undo = UiNativeAppearanceCommandUndo {
            key,
            previous: None,
            bounds: None,
            order: self.order.snapshot([key]),
            damage: self.pending_damage.clone(),
        };
        self.insert(
            UiNativeAppearanceCommand::TextForeground(successor),
            predecessor,
        )?;
        Ok((key, undo))
    }

    pub(crate) fn stage_text_replace(
        &mut self,
        successor: UiNativeFinalizedTextForeground,
        atlas: &UiNativeTextAtlas,
    ) -> Result<UiNativeTextCoverageUndo, UiNativeAppearanceRetainedDenial> {
        successor
            .validate_images(atlas)
            .map_err(|_| UiNativeAppearanceRetainedDenial::StaleTextImages)?;
        let command = UiNativeAppearanceCommand::TextForeground(successor);
        let key = *self
            .identities
            .get(&command.identity())
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        let undo = self.command_undo(key)?;
        self.replace(command)?;
        Ok(undo)
    }

    pub(crate) fn stage_text_remove(
        &mut self,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<UiNativeTextCoverageUndo, UiNativeAppearanceRetainedDenial> {
        let undo = self.command_undo(key)?;
        self.remove(key)?;
        Ok(undo)
    }

    /// Restore carried predecessor evidence even if its atlas images were evicted.
    /// Presentation owns reverse-order consumption of the move-only undo values.
    pub(crate) fn rollback_text(
        &mut self,
        undo: UiNativeTextCoverageUndo,
    ) -> Result<(), UiNativeAppearanceRetainedDenial> {
        let key = undo.key;
        let current_bounds = self.damage_bounds.get(&key).copied();
        self.replace_damage_index(key, current_bounds, undo.bounds)?;
        self.order
            .restore(undo.order)
            .map_err(UiNativeAppearanceRetainedDenial::Order)?;
        if let Some(current) = self.commands.remove(&key) {
            let text_commands = super::text_paint_commands(&current);
            self.remove_text_key(key, &text_commands);
            self.identities.remove(&current.identity());
            self.family_counts[family_slot(current.family())] -= 1;
        }
        if let Some(previous) = undo.previous {
            let text_commands = super::text_paint_commands(&previous);
            self.family_counts[family_slot(previous.family())] += 1;
            self.identities.insert(previous.identity(), key);
            self.commands.insert(key, previous);
            for identity in text_commands {
                self.text_keys_by_paint_command
                    .entry(identity)
                    .or_default()
                    .insert(key);
            }
        }
        match undo.bounds {
            Some(bounds) => {
                self.damage_bounds.insert(key, bounds);
            }
            None => {
                self.damage_bounds.remove(&key);
            }
        }
        self.pending_damage = undo.damage;
        Ok(())
    }

    pub(crate) fn stage_command_insert(
        &mut self,
        command: UiNativeAppearanceCommand,
        predecessor: Option<UiNativeAppearanceCommandKey>,
    ) -> Result<
        (UiNativeAppearanceCommandKey, UiNativeAppearanceCommandUndo),
        UiNativeAppearanceRetainedDenial,
    > {
        let key = UiNativeAppearanceCommandKey::new(self.next_key);
        let undo = UiNativeAppearanceCommandUndo {
            key,
            previous: None,
            bounds: None,
            order: self.order.snapshot([key]),
            damage: self.pending_damage.clone(),
        };
        self.insert(command, predecessor)?;
        Ok((key, undo))
    }

    pub(crate) fn stage_command_replace(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        command: UiNativeAppearanceCommand,
    ) -> Result<UiNativeAppearanceCommandUndo, UiNativeAppearanceRetainedDenial> {
        let undo = self.command_undo(key)?;
        self.replace_key(key, command)?;
        Ok(undo)
    }

    pub(crate) fn stage_command_remove(
        &mut self,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<UiNativeAppearanceCommandUndo, UiNativeAppearanceRetainedDenial> {
        let undo = self.command_undo(key)?;
        self.remove(key)?;
        Ok(undo)
    }

    fn command_undo(
        &self,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<UiNativeTextCoverageUndo, UiNativeAppearanceRetainedDenial> {
        let previous = self
            .commands
            .get(&key)
            .ok_or(UiNativeAppearanceRetainedDenial::MissingIdentity)?;
        Ok(UiNativeAppearanceCommandUndo {
            key,
            previous: Some(previous.clone()),
            bounds: self.damage_bounds.get(&key).copied(),
            order: self.order.snapshot([key]),
            damage: self.pending_damage.clone(),
        })
    }
}
