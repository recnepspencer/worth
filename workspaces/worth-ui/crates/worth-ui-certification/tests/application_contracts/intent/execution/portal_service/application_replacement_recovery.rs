use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
use worth_ui_runtime::certification_support::presented_completion;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationOutcome, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiPortalRuntimeCertificationExt;

use super::application_replacement_lifecycle::{open_portal, prepare_owner_removal, safe_boundary};
use super::motion_sampling::scripted_motion_host;
use crate::intent::admission::phase3::world::AdmissionWorld;
use crate::intent::operability::build_open_portal_application_with_host;

#[test]
fn replacement_keeps_lifecycle_bundle_through_host_in_flight_completion() {
    let host = scripted_motion_host();
    for _ in 0..2 {
        host.push_presented();
    }
    let (application, facts) = build_open_portal_application_with_host(host.clone());
    let candidate_facts = facts.clone();
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    open_portal(&mut world);
    let (pending, catalog) = prepare_owner_removal(&mut world, &candidate_facts);
    let boundary = safe_boundary(&mut world);
    let replacement = match world
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("the candidate prepares before the host attempt")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("the removed owner changes application meaning")
        }
    };
    host.push_in_flight(
        vec![presented_completion()],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let in_flight = match replacement.present(UiPresentationDeadline::at_tick(60), 2) {
        WorthUiMountedApplicationReplacementOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("the scripted replacement host must remain in flight"),
    };
    let _application = match in_flight.complete(3) {
        WorthUiMountedApplicationReplacementOutcome::Published { application, .. } => application,
        _ => panic!("the in-flight completion must publish the replacement"),
    };
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        0
    );
    super::exit_retention::assert_retention_census(&world.session);
    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}

#[test]
fn replacement_keeps_portal_authority_through_host_indeterminate_disposal() {
    let host = scripted_motion_host();
    for _ in 0..2 {
        host.push_presented();
    }
    let (application, facts) = build_open_portal_application_with_host(host.clone());
    let candidate_facts = facts.clone();
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    open_portal(&mut world);
    let (pending, catalog) = prepare_owner_removal(&mut world, &candidate_facts);
    let boundary = safe_boundary(&mut world);
    let replacement = match world
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("the candidate prepares before the host attempt")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("the removed owner changes application meaning")
        }
    };
    host.push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
    match replacement.present(UiPresentationDeadline::at_tick(60), 2) {
        WorthUiMountedApplicationReplacementOutcome::PresentationIndeterminate(recovery) => {
            drop(recovery);
        }
        _ => panic!("the scripted replacement host must report indeterminate presentation"),
    }
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        1,
        "indeterminate replacement cannot remove Portal authority"
    );
    super::exit_retention::assert_retention_census(&world.session);
    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}
