use worth_ui_host_contract::{
    UiMountedAppearanceMechanicChange, UiMountedAppearanceMechanicIdentity,
};

use super::UiHeadlessAppearanceMechanic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiHeadlessAppearanceMechanicChange {
    Insert(UiHeadlessAppearanceMechanic),
    Replace {
        predecessor: UiMountedAppearanceMechanicIdentity,
        successor: UiHeadlessAppearanceMechanic,
    },
    Remove(UiMountedAppearanceMechanicIdentity),
}

pub(super) fn translate_changes(
    changes: &[UiMountedAppearanceMechanicChange],
) -> Box<[UiHeadlessAppearanceMechanicChange]> {
    changes
        .iter()
        .map(|change| match change {
            UiMountedAppearanceMechanicChange::Insert(mechanic) => {
                UiHeadlessAppearanceMechanicChange::Insert(
                    UiHeadlessAppearanceMechanic::from_mounted(mechanic),
                )
            }
            UiMountedAppearanceMechanicChange::Replace {
                predecessor,
                successor,
            } => UiHeadlessAppearanceMechanicChange::Replace {
                predecessor: predecessor.clone(),
                successor: UiHeadlessAppearanceMechanic::from_mounted(successor),
            },
            UiMountedAppearanceMechanicChange::Remove(identity) => {
                UiHeadlessAppearanceMechanicChange::Remove(identity.clone())
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(crate) fn matches_mounted(
    transcript: &UiHeadlessAppearanceMechanicChange,
    source: &UiMountedAppearanceMechanicChange,
) -> bool {
    match (transcript, source) {
        (
            UiHeadlessAppearanceMechanicChange::Insert(translated),
            UiMountedAppearanceMechanicChange::Insert(source),
        ) => translated.matches_mounted(source),
        (
            UiHeadlessAppearanceMechanicChange::Replace {
                predecessor: translated_predecessor,
                successor: translated_successor,
            },
            UiMountedAppearanceMechanicChange::Replace {
                predecessor: source_predecessor,
                successor: source_successor,
            },
        ) => {
            translated_predecessor == source_predecessor
                && translated_successor.matches_mounted(source_successor)
        }
        (
            UiHeadlessAppearanceMechanicChange::Remove(translated),
            UiMountedAppearanceMechanicChange::Remove(source),
        ) => translated == source,
        _ => false,
    }
}
