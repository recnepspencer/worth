use super::*;
use crate::mounting::presentation::work_producer::command_motion_layers::tests::{
    boxed, command, layers, portal, scroll, RESTING,
};

/// How a tick shows each command it moves.
type Shown = HashMap<
    UiMountedPaintCommandIdentity,
    (UiCommandMotionLayers, UiMountedPresentationSampleChange),
>;

/// How the tick shows `command`: scrolled 6 points up in its viewport, and
/// presented by a Portal entrance halving it and moving it down 20.
fn scrolled_through_a_portal(command: UiMountedPaintCommandIdentity) -> Shown {
    let shown = layers(&[
        (UiCommandMotionLayerKind::Scroll, scroll()),
        (UiCommandMotionLayerKind::Portal, portal()),
    ]);
    HashMap::from([(command, (shown, shown.change(command, RESTING).unwrap()))])
}

fn resolved(
    damage: UiMotionTickDamage,
    shown: &Shown,
) -> Result<Vec<UiMountedCanonicalBox>, Denial> {
    let mut resolved = Vec::new();
    damage.resolve(shown, &mut resolved)?;
    Ok(resolved.into_iter().map(|damage| damage.bounds()).collect())
}

#[test]
fn a_scrolled_command_repaints_its_viewport_where_the_portal_shows_it() {
    let command = command();
    assert_eq!(
        resolved(
            UiMotionTickDamage::Scrolled(command),
            &scrolled_through_a_portal(command)
        ),
        Ok(vec![boxed([100.0, 120.0, 100.0, 50.0])])
    );
}

#[test]
fn a_faded_command_repaints_where_it_shows_within_its_viewport() {
    let command = command();
    // Laid out over the whole viewport, the command shows 3 points above
    // where the Portal shows the viewport, and the viewport clips that.
    assert_eq!(
        resolved(
            UiMotionTickDamage::Faded(command, boxed([100.0, 100.0, 200.0, 100.0])),
            &scrolled_through_a_portal(command)
        ),
        Ok(vec![boxed([100.0, 120.0, 100.0, 47.0])])
    );
}

#[test]
fn moved_damage_is_carried_by_every_layer_outside_the_one_that_moved() {
    let command = command();
    let shown = scrolled_through_a_portal(command);
    let drawn = boxed([140.0, 152.0, 10.0, 10.0]);
    assert_eq!(
        resolved(
            UiMotionTickDamage::Moved(command, UiCommandMotionLayerKind::Own, vec![drawn]),
            &shown
        ),
        Ok(vec![boxed([120.0, 143.0, 5.0, 5.0])])
    );
    assert_eq!(
        resolved(
            UiMotionTickDamage::Moved(command, UiCommandMotionLayerKind::Portal, vec![drawn]),
            &shown
        ),
        Ok(vec![drawn])
    );
    assert_eq!(
        resolved(UiMotionTickDamage::Shown(vec![drawn]), &shown),
        Ok(vec![drawn])
    );
}

#[test]
fn damage_for_a_command_the_tick_does_not_show_is_refused() {
    let shown = scrolled_through_a_portal(command());
    // Each command minted is a new one, so the tick shows none of this one.
    let unshown = command();
    assert_eq!(
        resolved(UiMotionTickDamage::Scrolled(unshown), &shown),
        Err(Denial::UnknownTargetCommands)
    );
}
