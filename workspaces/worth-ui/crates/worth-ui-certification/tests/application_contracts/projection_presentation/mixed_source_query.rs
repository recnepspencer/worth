use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::observation::{UiChangeClassificationOutcome, UiObservationFamily};
use worth_ui::facade::rebind::{
    UiProducedFactFamily, UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui::facade::source::{
    WorthUiSourceEventIngress, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_host_headless::{
    UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect, WorthUiHeadlessRecorder,
};
use worth_ui_query_binding::UiProjectionObservation;
use worth_ui_runtime::facade::mounted::UiMountedRgba8;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use crate::projection_lifecycle::support::ScalarLifecycleWorld;

use super::scalar_query_only::{
    mount_and_allocate, projection_app, projection_module_with_region, scalar_registration,
    ACTIVE_COMPONENT, STATUS_REGION,
};

#[test]
fn real_source_and_query_turn_publishes_one_semantic_application_successor() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 320.0,
            height: 96.0,
        },
    );
    let (mut query, completion) = ScalarLifecycleWorld::standard(NodeId::new(31361, 0), "Ready");
    let registration = scalar_registration(&query);
    let mut session = projection_app(registration, recorder.clone())
        .launch()
        .expect("active projection application launches");
    let predecessor_mounted_instances = mount_and_allocate(&mut session);
    let predecessor = session.generation_identity().clone();

    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    let current_fact = current.into_fact_and_predecessor().0;
    let source = candidate_source(&session);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(UiProjectionObservation::Scalar(
        current_fact.into_observation(),
    ))
    .unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    assert_eq!(
        admitted.summary().families(),
        &[
            UiObservationFamily::AuthoredSource,
            UiObservationFamily::Query,
        ]
    );

    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("source replacement and Query value change application presentation"),
    };
    assert_eq!(
        changed
            .facts()
            .iter()
            .map(|fact| fact.family())
            .collect::<Vec<_>>(),
        [
            UiProducedFactFamily::AuthoredSource,
            UiProducedFactFamily::Query,
        ]
    );
    let scope = session.resolve_affected_scope(changed).unwrap();
    assert!(scope.basis().has_distinct_candidate_generation());
    let query_lookup = scope
        .lookups()
        .iter()
        .find(|lookup| scope.facts()[lookup.fact_ordinal()].query().is_some())
        .expect("the Query fact has an indexed dual-generation lookup");
    assert!(!query_lookup.predecessor().entries().is_empty());
    assert!(!query_lookup.candidate().entries().is_empty());
    let lifecycle = scope.resolve_identity_lifecycle().unwrap();
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    let candidate = plan.basis().candidate_generation().clone();
    assert!(
        candidate != predecessor,
        "a real source semantic change must own a successor generation"
    );
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(314))
        .expect("mixed source and Query successor prepares");
    assert!(prepared.candidate_generation() == &candidate);
    let receipt = match prepared.execute(1) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("mixed source and Query rebind must publish atomically"),
    };

    let application = receipt
        .application_publication()
        .expect("the source change publishes an application successor");
    let mounted = receipt
        .mounted_publication()
        .expect("the same transaction publishes mounted content");
    assert!(application.prior_generation() == &predecessor);
    assert!(application.active_generation() == &candidate);
    assert!(mounted.generation() == &candidate);
    assert!(receipt.prior_generation() == &predecessor);
    assert!(receipt.active_generation() == &candidate);
    assert!(session.generation_identity() == &candidate);

    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    let transcript = &transcripts[0];
    assert_eq!(transcript.frame(), mounted.frame());
    assert_eq!(
        transcript
            .semantic_text()
            .iter()
            .map(|row| row.text())
            .collect::<Vec<_>>(),
        ["Ready", "CURRENT"]
    );
    assert!(transcript.semantic_text().iter().all(|row| {
        row.foregrounds().len() == 1
            && row.foregrounds()[0].color() == UiMountedRgba8::new(255, 255, 255, 255)
    }));
    let mounted_identity = transcript.semantic_text()[0].mounted_instance();
    assert!(predecessor_mounted_instances.contains(&mounted_identity));
    assert!(transcript
        .semantic_text()
        .iter()
        .all(|row| row.mounted_instance() == mounted_identity));
    assert!(session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .any(|row| row.identity() == mounted_identity));
    assert_eq!(
        transcript.unperformed_effects(),
        &[UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 1,
            portal_overlay_count: 0,
            semantic_text_count: 2,
            preview_node_count: 0,
        }]
    );

    drop(receipt);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
}

fn candidate_source(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    const PROVIDER: &str = "phase-313-mixed-source-query";
    let provider = WorthUiSourceProvider::rust_authored(PROVIDER).with_rust_authored_input(
        WorthUiRustAuthoredArtifactInput::from_modules([projection_module_with_region(
            ACTIVE_COMPONENT,
            STATUS_REGION,
        )]),
    );
    WorthUiSourceEventIngress::new(provider)
        .start()
        .ingest([WorthUiWatcherEvent::provider_revision(PROVIDER)])
        .unwrap()
        .attempt_candidate_for_certification(session.capabilities())
        .expect("the real Rust-authored source candidate lowers")
}
