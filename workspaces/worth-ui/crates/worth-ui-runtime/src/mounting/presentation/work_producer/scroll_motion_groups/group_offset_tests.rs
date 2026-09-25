//! The evidence each Scroll group crossing consumes is checked, not carried.
use super::super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use super::group_offset::{
    UiAcceptedCommandTranslation, UiAcceptedGroupOffset, UiBoundGroupStanding,
    UiDisplayedCommandTranslation, UiGroupStanding, UiPublishedGroupOffset,
    UiPublishedToAcceptedTranslation, UiScrollGroupBind,
};
use crate::mounting::presentation::UiAcceptedRect;
use crate::runtime::scroll::{UiScrollOffset, UiScrollPresentationDeviceScale};
use worth_ui_host_contract::*;

fn basis(epoch: u64) -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(epoch),
    )
}

fn content() -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 100.0,
        width: 100.0,
        height: 400.0,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}

fn accepted_on(presentation: UiHostObservationPresentationBasis, y: f32) -> UiAcceptedGroupOffset {
    let sample = UiAcceptedRect::sampled_for_test(
        [0.0, y, 100.0, 400.0],
        UiMountedCoordinateSpace::Viewport,
        presentation,
    )
    .unwrap();
    UiAcceptedGroupOffset::of_sample(content(), sample)
}

fn scale() -> UiScrollPresentationDeviceScale {
    UiScrollPresentationDeviceScale::admit(1000).unwrap()
}

fn published() -> UiPublishedGroupOffset {
    UiPublishedGroupOffset::of(UiScrollOffset::origin())
}

#[test]
fn published_moves_compose_only_on_one_presentation_basis() {
    let (tick, other) = (basis(1), basis(2));
    let here = published().move_to(accepted_on(tick, 80.0), scale());
    let there = published().move_to(accepted_on(other, 60.0), scale());
    assert_eq!(
        here.then(here)
            .map(UiPublishedToAcceptedTranslation::components),
        Ok([0.0, -40.0])
    );
    assert_eq!(here.then(there), Err(Denial::PresentationBasisMismatch));
    assert_eq!(
        UiAcceptedCommandTranslation::from_published(tick, [here]).map(|moved| moved.components()),
        Ok([0.0, -20.0])
    );
    assert_eq!(
        UiAcceptedCommandTranslation::from_published(other, [here]),
        Err(Denial::PresentationBasisMismatch),
        "a command drawn on one basis never takes a move accepted on another"
    );
}

#[test]
fn a_displayed_base_shows_only_the_standings_its_own_bind_bound() {
    let (source, sampled) = (content(), content());
    let bind = UiScrollGroupBind::default().next();
    let base = UiDisplayedCommandTranslation::of_displayed_transform(source, sampled, bind);
    let standing = UiGroupStanding::Published(published());
    let shown = UiBoundGroupStanding::new(standing, bind)
        .shown_by(base)
        .expect("a base shows the standing bound with it");
    let tick = basis(1);
    let step = shown.move_to(accepted_on(tick, 80.0), scale());
    assert_eq!(
        UiAcceptedCommandTranslation::from_displayed(tick, base, [step])
            .map(|moved| moved.components()),
        Ok([0.0, -20.0])
    );
    assert_eq!(
        UiAcceptedCommandTranslation::from_displayed(basis(2), base, [step]),
        Err(Denial::PresentationBasisMismatch)
    );
    assert_eq!(
        UiBoundGroupStanding::new(standing, bind.next()).shown_by(base),
        Err(Denial::DisplayedBaseFromAnotherBind),
        "a base another bind read shows the group somewhere else"
    );
}
