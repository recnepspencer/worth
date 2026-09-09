use worth_ui::facade::rebind::{UiRebindExecutionRequest, UiRebindOutcome};
use worth_ui::facade::source::{
    WorthUiSourceEventIngress, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, WorthUiQueryWorkspaceExt,
};

use crate::projection_presentation::collection_query::{
    collection_app, collection_module, collection_plan, collection_registration,
};
use crate::projection_presentation::scalar_query_only::mount_and_allocate;

pub(super) fn run_capacity_boundary() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 320.0,
            height: 128.0,
        },
    );
    let (mut workspace, entities) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace(
            vec![
                ("pulse.alpha".to_owned(), "Alpha".to_owned()),
                ("pulse.bravo".to_owned(), "Bravo".to_owned()),
            ],
            worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    let domain = workspace.worth_ui().expect("Worth UI domain installed");
    let registration = collection_registration(&domain);
    let mut session = collection_app(registration.clone(), recorder)
        .launch()
        .expect("capacity application launches");
    mount_and_allocate(&mut session);
    let binding = match registration.admit(&workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("collection binding admits: {stop:?}")
        }
    };
    let opened = match binding.open(
        UiCollectionProjectionBudget::new(2, 3, 0, 2048).unwrap(),
        &mut workspace,
    ) {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => {
            panic!("collection projection opens: {stop:?}")
        }
    };
    let (mut live, snapshot) = opened.into_parts();
    let snapshot_plan = collection_plan(&mut session, snapshot.into_observation());
    execute_content_plan(&mut session, snapshot_plan, 50_000);

    worth_ui_query_binding::certification::update_projection_status(
        &mut workspace,
        entities[1].clone(),
        "Bravo capacity",
    );
    let patch = refresh_fact(&mut live, &mut workspace);
    let plan = collection_plan(&mut session, patch.into_observation());
    let sets = (0..17)
        .map(|index| {
            let candidate = source_candidate(&session, &format!("phase-313-qp06-capacity-{index}"));
            let mut turn = session.begin_observation_turn().unwrap();
            turn.admit_source(candidate).unwrap();
            turn.seal().unwrap()
        })
        .collect::<Vec<_>>();
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(50_001))
        .expect("capacity predecessor prepares");
    let mut effecting = prepared
        .begin_effecting()
        .unwrap_or_else(|_| panic!("capacity predecessor begins effecting"));
    assert_eq!(effecting.queued_observation_count(), 0);

    let mut sets = sets.into_iter();
    for expected in 1..=15 {
        let receipt = admit_one(&mut effecting, sets.next().unwrap());
        assert_eq!(receipt.total_queued_observations(), expected);
    }
    assert_eq!(effecting.queued_observation_count(), 15);
    let sixteenth = admit_one(&mut effecting, sets.next().unwrap());
    assert_eq!(sixteenth.total_queued_observations(), 16);
    assert_eq!(sixteenth.remaining_capacity(), 0);

    let rejected = sets.next().unwrap();
    let rejected_turn = rejected.turn();
    let stop = match effecting.admit_observations(rejected) {
        Ok(_) => panic!("the seventeenth observation exceeds exact capacity"),
        Err(stop) => stop,
    };
    assert_eq!(stop.configured(), 16);
    assert_eq!(stop.observed(), 16);
    assert_eq!(stop.attempted(), 17);
    let returned = stop.into_observation_set();
    assert_eq!(returned.turn(), rejected_turn);
    let (outcome, queued) = effecting.complete(50_001).into_parts();
    match &outcome {
        UiRebindOutcome::Published(_) => {}
        UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "capacity publication rejected: {:?}",
            denial
                .host_rejections()
                .iter()
                .map(|rejection| rejection.denial())
                .collect::<Vec<_>>()
        ),
        _ => panic!("capacity publication did not reach a terminal publish"),
    }
    assert_eq!(queued.len(), 16);
    drop(outcome);
    drop(queued);
    drop(returned);
    close_live(live, &mut workspace);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
}

fn admit_one<'session>(
    effecting: &mut worth_ui_runtime::facade::rebind::UiEffectingRebind<'session>,
    set: worth_ui::facade::observation::UiAdmittedObservationSet,
) -> worth_ui_runtime::facade::observation::UiEffectingObservationQueueAdmissionReceipt {
    match effecting.admit_observations(set) {
        Ok(receipt) => receipt,
        Err(_) => panic!("the first sixteen observations fit exact capacity"),
    }
}

fn refresh_fact(
    live: &mut worth_ui_query_binding::UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> worth_ui_query_binding::UiCollectionProjectionFactReceipt {
    match live.refresh(workspace).unwrap() {
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::Applied(receipt) => {
            receipt.into_fact()
        }
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("changed collection row produces a patch")
        }
    }
}

fn execute_content_plan(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    plan: worth_ui::facade::rebind::UiRebindPlan,
    request: u64,
) {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(request))
        .expect("content plan prepares");
    assert!(matches!(
        prepared.execute(request),
        UiRebindOutcome::Published(_)
    ));
}

fn source_candidate(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    provider: &str,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    let source = WorthUiSourceProvider::rust_authored(provider).with_rust_authored_input(
        WorthUiRustAuthoredArtifactInput::from_modules([collection_module(false)]),
    );
    WorthUiSourceEventIngress::new(source)
        .start()
        .ingest([WorthUiWatcherEvent::provider_revision(provider)])
        .unwrap()
        .attempt_candidate_for_certification(session.capabilities())
        .expect("real Rust-authored candidate lowers")
}

fn close_live(
    live: worth_ui_query_binding::UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) {
    match live.close(workspace) {
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Closed(_) => {}
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!("live collection closes: {:?}", stop.query_error())
        }
    }
}
