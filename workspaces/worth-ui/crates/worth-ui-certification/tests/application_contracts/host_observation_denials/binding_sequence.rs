use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationReportDenial,
    UiHostObservationReportOutcome,
};

use crate::host_observation_fixture::{batch, report, source, window_focus};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_application_lifecycle::published_mounted_world::published_observation_world;

#[test]
fn observation_sequence_remains_session_scoped_across_a_binding_successor() {
    let mut world = published_observation_world("observation-binding-successor-sequence");
    let first = batch(
        source(&world.session, world.binding, &world.current),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            window_focus(&world.current, true),
            &world.current,
        )],
    );
    assert!(matches!(
        world.session.validate_host_observation_batch(first),
        UiHostObservationReportOutcome::Validated(_)
    ));

    let successor = world.rebind_surface(profile(2));
    let successor_basis = world.current;

    let next = batch(
        source(&world.session, successor, &successor_basis),
        (2, 2),
        UiHostObservationLoss::Complete,
        vec![report(
            2,
            window_focus(&successor_basis, false),
            &successor_basis,
        )],
    );
    assert!(matches!(
        world.session.validate_host_observation_batch(next),
        UiHostObservationReportOutcome::Validated(_)
    ));

    let reset = batch(
        source(&world.session, successor, &successor_basis),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            UiHostObservationPayload::Tick { tick: 1 },
            &successor_basis,
        )],
    );
    assert_eq!(
        world.session.validate_host_observation_batch(reset),
        UiHostObservationReportOutcome::Denied(UiHostObservationReportDenial::SequenceReordered)
    );
}
