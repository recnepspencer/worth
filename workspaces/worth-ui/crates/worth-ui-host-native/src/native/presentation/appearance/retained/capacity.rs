use std::collections::BTreeMap;

use super::super::command::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandFamily, UiNativeAppearanceCommandKey,
};

pub(super) fn family_slot(family: UiNativeAppearanceCommandFamily) -> usize {
    match family {
        UiNativeAppearanceCommandFamily::Surface
        | UiNativeAppearanceCommandFamily::PortalSurface => 0,
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
        | UiNativeAppearanceCommandFamily::PortalSurface => usize::from(profile.surface_commands),
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
