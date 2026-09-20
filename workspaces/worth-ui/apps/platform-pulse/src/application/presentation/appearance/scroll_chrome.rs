//! The appearance roles that paint the dashboard's scroll track and thumb.
//!
//! The chrome contract on a Mosaic region names these two identities; this file
//! is where the names are defined and where the states they paint are decided.
//! Both roles partition on Hover and Pressed, so a resting bar, a bar under the
//! pointer and a thumb being dragged are three distinct declared appearances.
//! `PressedCapturedOutside` is the drag that has left the gutter, which is why
//! it resolves to the same held tone as `PressedArmedInside`.
use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearanceCellValue, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleAuthoringDenial, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
    UiAppearanceStateAxis, UiThemeSlotIdentity, UiThemeValueKind,
};

/// The registered role that paints a scroll track.
pub(in crate::application) const PLATFORM_PULSE_SCROLL_TRACK_ROLE: &str =
    "platform.pulse.appearance.scroll_track";
/// The registered role that paints a scroll thumb.
pub(in crate::application) const PLATFORM_PULSE_SCROLL_THUMB_ROLE: &str =
    "platform.pulse.appearance.scroll_thumb";

/// The identity a chrome contract names for the track.
pub(in crate::application) fn track_role_identity() -> UiAppearanceRoleIdentity {
    role_identity(PLATFORM_PULSE_SCROLL_TRACK_ROLE)
}

/// The identity a chrome contract names for the thumb.
pub(in crate::application) fn thumb_role_identity() -> UiAppearanceRoleIdentity {
    role_identity(PLATFORM_PULSE_SCROLL_THUMB_ROLE)
}

/// The two chrome roles, in track-then-thumb order.
pub(in crate::application) fn roles(
) -> Result<[UiAppearanceRoleDeclaration; 2], UiAppearanceRoleAuthoringDenial> {
    Ok([track_role()?, thumb_role()?])
}

/// The track: a subtle reserved gutter that firms up under the pointer and
/// again while the thumb it holds is being dragged.
fn track_role() -> Result<UiAppearanceRoleDeclaration, UiAppearanceRoleAuthoringDenial> {
    UiAppearanceRole::authoring(track_role_identity())
        .cover(
            UiAppearanceAspect::Background,
            interaction_partition(
                "theme.platform_pulse.scroll_track",
                "theme.platform_pulse.scroll_track_hover",
                "theme.platform_pulse.scroll_track_drag",
            ),
        )?
        .build()
}

/// The thumb: a rounded bar whose three tones say resting, hovered and held.
fn thumb_role() -> Result<UiAppearanceRoleDeclaration, UiAppearanceRoleAuthoringDenial> {
    UiAppearanceRole::authoring(thumb_role_identity())
        .cover(
            UiAppearanceAspect::Background,
            interaction_partition(
                "theme.platform_pulse.scroll_thumb",
                "theme.platform_pulse.scroll_thumb_hover",
                "theme.platform_pulse.scroll_thumb_drag",
            ),
        )?
        .cover(UiAppearanceAspect::Radius, rounded())?
        .build()
}

/// Hover x Pressed, resolved to three declared tones.
///
/// A held thumb is held whether the pointer is still inside the gutter or has
/// been dragged off it, so both pressed classes reach one cell. What the three
/// cells leave uncovered — not hovered, not pressed — is the resting tone.
fn interaction_partition(
    resting: &'static str,
    hovered: &'static str,
    held: &'static str,
) -> UiAppearancePartitionAuthoring {
    UiAppearancePartitionAuthoring::new([
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover),
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Pressed),
    ])
    .with_cell(
        UiAppearanceCell::named("held")
            .when([UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::PressedArmedInside,
            )])
            .uses_slot(theme_slot(held), UiThemeValueKind::Color),
    )
    .with_cell(
        UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(
            UiAppearanceAxisClass::PressedCapturedOutside,
        )])
        .same_as("held"),
    )
    .with_cell(
        UiAppearanceCell::when([
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::Hovered),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::PressedIdle),
        ])
        .uses_slot(theme_slot(hovered), UiThemeValueKind::Color),
    )
    .with_otherwise(UiAppearanceCellValue::theme_slot(
        theme_slot(resting),
        UiThemeValueKind::Color,
    ))
}

/// The thumb's corner radii. A six-point bar takes the largest declared radius,
/// which is what makes it read as a capsule rather than a rectangle.
fn rounded() -> UiAppearancePartitionAuthoring {
    UiAppearancePartitionAuthoring::new([]).with_cell(UiAppearanceCell::when([]).uses_slot(
        theme_slot("theme.platform_pulse.radius.r32"),
        UiThemeValueKind::CornerRadii,
    ))
}

fn role_identity(value: &'static str) -> UiAppearanceRoleIdentity {
    UiAppearanceRoleIdentity::new(value)
        .expect("a Pulse scroll chrome role identity is well formed")
}

fn theme_slot(value: &'static str) -> UiThemeSlotIdentity {
    UiThemeSlotIdentity::new(value).expect("a Pulse scroll chrome theme slot is well formed")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both chrome roles admit, and they are two different roles. One role
    /// painting both parts is exactly what the chrome contract refuses.
    #[test]
    fn the_track_and_the_thumb_are_two_admitted_roles() {
        let [track, thumb] = roles().expect("the authored scroll chrome roles are admitted");
        assert_eq!(track.role(), &track_role_identity());
        assert_eq!(thumb.role(), &thumb_role_identity());
        assert_ne!(track_role_identity(), thumb_role_identity());
    }

    /// Hover x Pressed is six states, and every one of them resolves. A hole
    /// would leave a bar with no declared appearance in some state it reaches.
    #[test]
    fn every_hover_and_pressed_state_resolves_to_a_declared_tone() {
        let [track, thumb] = roles().expect("the authored scroll chrome roles are admitted");
        for role in [&track, &thumb] {
            let (_, partition) = role
                .partitions()
                .iter()
                .find(|(aspect, _)| *aspect == UiAppearanceAspect::Background)
                .expect("each chrome role covers its background");
            assert_eq!(partition.cells().len(), 6);
            assert_eq!(partition.axes().len(), 2);
        }
    }

    /// The held tone is the one the drag keeps when the pointer leaves the
    /// gutter, so the two pressed classes must resolve to the same slot.
    #[test]
    fn a_drag_that_leaves_the_gutter_keeps_the_held_tone() {
        let [_, thumb] = roles().expect("the authored scroll chrome roles are admitted");
        let (_, partition) = thumb
            .partitions()
            .iter()
            .find(|(aspect, _)| *aspect == UiAppearanceAspect::Background)
            .expect("the thumb covers its background");
        let slot_for = |class| {
            partition
                .cells()
                .iter()
                .find(|cell| cell.classes().contains(&class))
                .and_then(|cell| cell.result().slot().cloned())
                .expect("every pressed class resolves")
        };
        assert_eq!(
            slot_for(UiAppearanceAxisClass::PressedCapturedOutside),
            slot_for(UiAppearanceAxisClass::PressedArmedInside),
        );
        assert_ne!(
            slot_for(UiAppearanceAxisClass::PressedCapturedOutside),
            theme_slot("theme.platform_pulse.scroll_thumb"),
        );
    }
}
