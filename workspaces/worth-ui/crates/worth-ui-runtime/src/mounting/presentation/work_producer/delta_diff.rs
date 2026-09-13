use std::collections::HashSet;

use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedLogicalDamage, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPaintOrderEdit,
};

use super::{
    command_same_presentation_meaning, command_visible_bounds, UiMountedPresentationState,
};

pub(super) fn affected_commands(
    predecessor: &UiMountedPresentationState,
    successor: &UiMountedPresentationState,
    changed_instances: &[UiMountedInstanceIdentity],
) -> Vec<UiMountedPaintCommandIdentity> {
    let mut seen = HashSet::new();
    let mut affected = Vec::new();
    for instance in changed_instances {
        for identity in predecessor
            .command_identities_for_instance(*instance)
            .chain(successor.command_identities_for_instance(*instance))
        {
            if seen.insert(identity) {
                affected.push(identity);
            }
        }
    }
    affected.sort_unstable_by_key(|identity| {
        successor
            .order_key(*identity)
            .or_else(|| predecessor.order_key(*identity))
            .unwrap_or((u32::MAX, u64::MAX, usize::MAX))
    });
    affected
}

pub(super) fn command_changes(
    predecessor: &UiMountedPresentationState,
    successor: &UiMountedPresentationState,
    affected: &[UiMountedPaintCommandIdentity],
) -> (
    Vec<UiMountedPaintCommandChange>,
    Vec<UiMountedLogicalDamage>,
) {
    let mut changes = Vec::new();
    let mut damage = Vec::new();
    for identity in affected.iter().copied() {
        push_change(
            identity,
            predecessor.command_option(identity),
            successor.command_option(identity),
            &mut changes,
            &mut damage,
        );
    }
    (changes, damage)
}

fn push_change(
    identity: UiMountedPaintCommandIdentity,
    before: Option<&worth_ui_host_contract::UiMountedPaintCommand>,
    after: Option<&worth_ui_host_contract::UiMountedPaintCommand>,
    changes: &mut Vec<UiMountedPaintCommandChange>,
    damage: &mut Vec<UiMountedLogicalDamage>,
) {
    match (before, after) {
        (None, Some(command)) => {
            changes.push(UiMountedPaintCommandChange::Insert(command.clone()));
            append_damage(command, damage);
        }
        (Some(command), None) => {
            changes.push(UiMountedPaintCommandChange::Remove(identity));
            append_damage(command, damage);
        }
        (Some(old), Some(new)) if !command_same_presentation_meaning(old, new) => {
            changes.push(UiMountedPaintCommandChange::replacement(
                old.identity(),
                new.clone(),
            ));
            append_damage(old, damage);
            append_damage(new, damage);
        }
        (Some(_), Some(_)) => {}
        (None, None) => {}
    }
}

fn append_damage(
    command: &worth_ui_host_contract::UiMountedPaintCommand,
    damage: &mut Vec<UiMountedLogicalDamage>,
) {
    damage
        .extend(command_visible_bounds(command).map(UiMountedLogicalDamage::from_runtime_mounting));
}

pub(super) fn order_edits(
    predecessor: &UiMountedPresentationState,
    successor: &UiMountedPresentationState,
    affected: &[UiMountedPaintCommandIdentity],
) -> Vec<UiMountedPaintOrderEdit> {
    let affected_set = affected.iter().copied().collect::<HashSet<_>>();
    let mut removals = affected
        .iter()
        .filter(|identity| {
            predecessor.command_option(**identity).is_some()
                && successor.command_option(**identity).is_none()
        })
        .map(|identity| {
            UiMountedPaintOrderEdit::remove(
                worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(*identity),
            )
        })
        .collect::<Vec<_>>();
    removals.sort_unstable_by_key(|edit| {
        predecessor
            .order_key(edit.identity().command())
            .unwrap_or((u32::MAX, u64::MAX, usize::MAX))
    });
    let mut placement_identities = affected_set
        .iter()
        .filter(|identity| successor.command_option(**identity).is_some())
        .copied()
        .collect::<Vec<_>>();
    placement_identities.sort_unstable_by_key(|identity| {
        successor
            .order_key(*identity)
            .unwrap_or((u32::MAX, u64::MAX, usize::MAX))
    });
    let mut placements = placement_identities
        .into_iter()
        .filter_map(|identity| {
            let expected = successor.previous_order(identity);
            let retained = predecessor
                .command_option(identity)
                .map(|_| predecessor.previous_order(identity));
            (retained != Some(expected)).then(|| {
                UiMountedPaintOrderEdit::place_after(
                    worth_ui_host_contract::UiMountedPaintOrderIdentity::for_command(identity),
                    expected,
                )
            })
        })
        .collect::<Vec<_>>();
    removals.append(&mut placements);
    removals
}
