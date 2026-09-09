use worth_ui::facade::observation::{UiChangeClassificationOutcome, UiObservationFamily};
use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationReportOutcome,
    WorthUiHostObservationSessionExt,
};
use worth_ui::facade::rebind::{
    UiAffectedScopeCost, UiProducedFactFamily, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui::facade::source::{
    WorthUiSourceEventIngress, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiProjectionObservation, WorthUiQueryWorkspaceExt,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use crate::host_observation_fixture::{batch, report, source};
use crate::mounted_application_lifecycle::published_mounted_world::{
    presented_epoch, PresentedObservationBasis,
};
use crate::projection_presentation::collection_query::{
    collection_app, collection_module, collection_plan, collection_registration,
};
use crate::projection_presentation::scalar_query_only::mount_and_allocate;

#[derive(Clone, Copy)]
pub(super) enum MixedCause {
    Query,
    Source,
    Viewport,
}

pub(super) fn run_mixed_permutation(order: [MixedCause; 3], run: usize) -> UiAffectedScopeCost {
    let recorder = recorder();
    let (mut workspace, entities) = seeded_workspace();
    let domain = workspace.worth_ui().expect("Worth UI domain installed");
    let registration = collection_registration(&domain);
    let mut session = collection_app(registration.clone(), recorder.clone())
        .launch()
        .expect("mixed Query observation application launches");
    mount_and_allocate(&mut session);
    let binding = match registration.admit(&workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("collection binding admits: {stop:?}")
        }
    };
    let opened = match binding.open(
        UiCollectionProjectionBudget::new(2, 4, 0, 2048).unwrap(),
        &mut workspace,
    ) {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => {
            panic!("collection projection opens: {stop:?}")
        }
    };
    let (mut live, snapshot) = opened.into_parts();
    let snapshot_plan = collection_plan(&mut session, snapshot.into_observation());
    execute_content_plan(&mut session, snapshot_plan, 40_000 + run as u64);

    worth_ui_query_binding::certification::update_projection_status(
        &mut workspace,
        entities[1].clone(),
        "Bravo effecting",
    );
    let effecting_fact = refresh_fact(&mut live, &mut workspace);
    let effecting_plan = collection_plan(&mut session, effecting_fact.into_observation());

    worth_ui_query_binding::certification::update_projection_status(
        &mut workspace,
        entities[1].clone(),
        "Bravo mixed",
    );
    let mixed_fact = refresh_fact(&mut live, &mut workspace);
    let viewport = validated_viewport(&mut session, &recorder);
    let candidate = source_candidate(&session, &format!("phase-313-qp06-mixed-{run}"), true);
    let mut query = Some(mixed_fact.into_observation());
    let mut source_candidate = Some(candidate);
    let mut viewport = Some(viewport);
    let mut turn = session.begin_observation_turn().unwrap();
    for cause in order {
        match cause {
            MixedCause::Query => turn
                .admit_projection_query(UiProjectionObservation::Collection(query.take().unwrap()))
                .map(|_| ())
                .unwrap(),
            MixedCause::Source => turn
                .admit_source(source_candidate.take().unwrap())
                .map(|_| ())
                .unwrap(),
            MixedCause::Viewport => turn
                .admit_host(viewport.take().unwrap())
                .map(|_| ())
                .unwrap(),
        }
    }
    let mixed = turn.seal().unwrap();
    assert_eq!(
        mixed.summary().families(),
        &[
            UiObservationFamily::AuthoredSource,
            UiObservationFamily::HostViewport,
            UiObservationFamily::Query,
        ]
    );

    let prepared = session
        .prepare_rebind(
            effecting_plan,
            UiRebindExecutionRequest::new(41_000 + run as u64),
        )
        .expect("the predecessor collection patch prepares");
    let mut effecting = prepared
        .begin_effecting()
        .unwrap_or_else(|_| panic!("the prepared content rebind owns effecting authority"));
    assert_eq!(effecting.queued_observation_count(), 0);
    let queued = match effecting.admit_observations(mixed) {
        Ok(receipt) => receipt,
        Err(_) => panic!("the three-cause turn fits the bounded effecting queue"),
    };
    assert_eq!(queued.admitted_observations(), 3);
    assert_eq!(queued.total_queued_observations(), 3);
    let (effecting_outcome, queued) = effecting.complete(41_000 + run as u64).into_parts();
    assert!(matches!(effecting_outcome, UiRebindOutcome::Published(_)));
    drop(effecting_outcome);
    let mut queued = queued.into_vec();
    assert_eq!(queued.len(), 1);
    let mixed = queued.pop().unwrap();

    let changed = match session.classify_observations(mixed).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("the queued source, viewport, and Query facts change presentation"),
    };
    assert_eq!(
        changed
            .facts()
            .iter()
            .map(|fact| fact.family())
            .collect::<Vec<_>>(),
        [
            UiProducedFactFamily::AuthoredSource,
            UiProducedFactFamily::HostViewport,
            UiProducedFactFamily::Query,
        ]
    );
    let scope = session.resolve_affected_scope(changed).unwrap();
    let cost = scope.cost();
    assert_eq!(cost.observations(), 3);
    assert_eq!(cost.changed_facts(), 3);
    assert_eq!(cost.lookup_receipts(), 6);
    assert_eq!(cost.index_probes(), 6);
    let lifecycle = scope.resolve_identity_lifecycle().unwrap();
    let plan = session
        .compile_rebind_plan(
            lifecycle,
            worth_ui::facade::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let candidate_generation = plan.basis().candidate_generation().clone();
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(42_000 + run as u64))
        .expect("queued mixed successor prepares once");
    let receipt = match prepared.execute(42_000 + run as u64) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("queued mixed successor publishes atomically"),
    };
    assert!(receipt.application_publication().is_some());
    assert!(receipt.mounted_publication().is_some());
    assert!(receipt.active_generation() == &candidate_generation);
    assert!(session.generation_identity() == &candidate_generation);
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 3);
    assert!(transcripts[2]
        .semantic_text()
        .iter()
        .any(|row| row.text() == "Bravo mixed"));
    drop(receipt);

    close_live(live, &mut workspace);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
    cost
}

fn seeded_workspace() -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    worth_ui_query_binding::certification::seeded_collection_projection_workspace(
        vec![
            ("pulse.alpha".to_owned(), "Alpha".to_owned()),
            ("pulse.bravo".to_owned(), "Bravo".to_owned()),
        ],
        worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
    )
}

fn recorder() -> WorthUiHeadlessRecorder {
    WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 320.0,
            height: 128.0,
        },
    )
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

fn validated_viewport(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &WorthUiHeadlessRecorder,
) -> worth_ui::facade::observation_report::UiValidatedHostObservationBatch {
    let sequence = 1;
    let current = session
        .current_mounted_publication()
        .expect("snapshot content is currently published");
    let frame = current.frame();
    let binding = current.bindings()[0];
    let transcripts = recorder.observed_transcripts();
    let row = transcripts
        .last()
        .expect("snapshot transcript exists")
        .semantic_text()
        .first()
        .expect("snapshot contains semantic text");
    let basis = PresentedObservationBasis {
        host_surface: session.inspect_mounted_identity().surface_bindings()[0]
            .host_surface_identity(),
        frame,
        epoch: presented_epoch(session, frame, binding),
        instance: row.mounted_instance(),
        receipt: row.node_receipt(),
    };
    let raw = batch(
        source(session, binding, &basis),
        (sequence, sequence),
        UiHostObservationLoss::Complete,
        vec![report(
            sequence,
            UiHostObservationPayload::Viewport {
                width_subpixels: 320_000,
                height_subpixels: 128_000,
            },
            &basis,
        )],
    );
    match session.validate_host_observation_batch(raw) {
        UiHostObservationReportOutcome::Validated(batch) => batch,
        other => panic!("current viewport report validates: {other:?}"),
    }
}

fn source_candidate(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    provider: &str,
    with_region: bool,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    let source = WorthUiSourceProvider::rust_authored(provider).with_rust_authored_input(
        WorthUiRustAuthoredArtifactInput::from_modules([collection_module(with_region)]),
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
