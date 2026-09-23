use std::collections::BTreeMap;

use super::super::command::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandFamily, UiNativeAppearanceCommandKey,
};

pub(super) fn family_slot(family: UiNativeAppearanceCommandFamily) -> usize {
    match family {
        // Chrome is a surface fill with a reserved paint order; it is counted
        // and budgeted with the surfaces rather than against a family of its
        // own.
        UiNativeAppearanceCommandFamily::Surface
        | UiNativeAppearanceCommandFamily::PortalSurface
        | UiNativeAppearanceCommandFamily::ScrollChrome => 0,
        UiNativeAppearanceCommandFamily::Outline => 1,
        UiNativeAppearanceCommandFamily::TextForeground => 2,
        UiNativeAppearanceCommandFamily::Backdrop => 3,
        UiNativeAppearanceCommandFamily::OverlayOrder => 4,
        UiNativeAppearanceCommandFamily::PointerAffordance => 5,
    }
}

pub(super) fn family_capacity(
    family: UiNativeAppearanceCommandFamily,
    profile: crate::native_profile::UiNativeAppearanceProfile,
) -> usize {
    match family {
        UiNativeAppearanceCommandFamily::Surface
        | UiNativeAppearanceCommandFamily::PortalSurface
        | UiNativeAppearanceCommandFamily::ScrollChrome => usize::from(profile.surface_commands),
        UiNativeAppearanceCommandFamily::Outline => usize::from(profile.outline_commands),
        UiNativeAppearanceCommandFamily::TextForeground => {
            usize::from(profile.text_foreground_commands)
        }
        UiNativeAppearanceCommandFamily::Backdrop => usize::from(profile.backdrop_commands),
        UiNativeAppearanceCommandFamily::OverlayOrder => {
            usize::from(profile.overlay_order_commands)
        }
        UiNativeAppearanceCommandFamily::PointerAffordance => {
            usize::from(profile.pointer_affordance_commands)
        }
    }
}

pub(super) fn command_family(
    commands: &BTreeMap<UiNativeAppearanceCommandKey, UiNativeAppearanceCommand>,
    key: UiNativeAppearanceCommandKey,
) -> UiNativeAppearanceCommandFamily {
    commands
        .get(&key)
        .expect("a committed staged command has a retained family")
        .family()
}
