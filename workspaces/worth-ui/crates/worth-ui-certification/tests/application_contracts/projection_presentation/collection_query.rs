use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiProjectionCollectionPolicy,
    WorthUiProjectionCollectionSelection, WorthUiProjectionLifecycle,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_host_contract::UiSemanticTextSlot;
use worth_ui_host_headless::{
    UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect, WorthUiHeadlessRecorder,
};
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiCollectionProjectionRegistration,
    UiProjectionFieldRequirement, UiProjectionObservation, WorthUiQueryWorkspaceExt,
};

use super::scalar_query_only::{
    component_descriptor, mount_and_allocate, status_region_descriptor, text_token_descriptor,
    ACTIVE_COMPONENT, PROJECTION, STATUS_REGION, TEXT_COLOR,
};

#[path = "collection_query/locality.rs"]
mod locality;

#[test]
fn real_query_collection_snapshot_and_patch_publish_keyed_semantic_text() {
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
    let mut session = collection_app(registration.clone(), recorder.clone())
        .launch()
        .expect("collection projection application launches");
    let mounted_instances = mount_and_allocate(&mut session);
    let generation = session.generation_identity().clone();
    let opened = open_live_collection(registration, &mut workspace);
    let (mut live, snapshot) = opened.into_parts();
    publish_collection_fact(&mut session, snapshot.into_observation(), 3131);
    let (inserted, analyzed_bytes) = exercise_membership_shifts(
        &mut workspace,
        &entities,
        &mut live,
        &mut session,
        &recorder,
    );
    assert_membership_shift_transcripts(
        &recorder.observed_transcripts(),
        &mounted_instances,
        &entities,
        inserted,
    );
    let production = recorder
        .latest_production_cost()
        .expect("the public headless boundary observes the final successor cost");
    assert_eq!(production.retained_command_scans(), 0);
    assert_eq!(production.retained_command_clones(), 0);
    assert_eq!(production.projection_rows_materialized(), 0);
    assert_eq!(analyzed_bytes, "Bravo updated".len());
    assert!(session.generation_identity() == &generation);
    close_collection(live, &mut workspace);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
}

fn exercise_membership_shifts(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entities: &[worth_query::facade::foundation::WorthQueryEntityIdentity],
    live: &mut worth_ui_query_binding::UiLiveCollectionProjection,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &WorthUiHeadlessRecorder,
) -> (
    worth_query::facade::foundation::WorthQueryEntityIdentity,
    usize,
) {
    worth_ui_query_binding::certification::remove_projection_entity(workspace, entities[0].clone());
    refresh_and_publish(live, workspace, session, 3132, "removed Query row");
    let before_content_update = recorder
        .observed_transcripts()
        .last()
        .cloned()
        .expect("removal transcript");
    worth_ui_query_binding::certification::update_projection_status(
        workspace,
        entities[1].clone(),
        "Bravo updated",
    );
    refresh_and_publish(live, workspace, session, 3133, "changed Query row");
    let after_content_update = recorder
        .observed_transcripts()
        .last()
        .cloned()
        .expect("content transcript");
    locality::assert_content_update_is_local(&before_content_update, &after_content_update);
    let inserted = worth_ui_query_binding::certification::insert_projection_status(
        workspace,
        "pulse.aaron",
        "Aaron",
    );
    refresh_and_publish(live, workspace, session, 3134, "inserted Query row");
    worth_ui_query_binding::certification::update_projection_status(
        workspace,
        entities[1].clone(),
        "Bravo final",
    );
    refresh_and_publish(live, workspace, session, 3135, "shifted Query row");
    (inserted, "Bravo updated".len())
}

fn refresh_and_publish(
    live: &mut worth_ui_query_binding::UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: u64,
    expectation: &str,
) {
    let fact = match live.refresh(workspace).unwrap() {
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::Applied(receipt) => {
            receipt.into_fact()
        }
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("the {expectation} produces one exact patch")
        }
    };
    publish_collection_fact(session, fact.into_observation(), request);
}

fn assert_membership_shift_transcripts(
    transcripts: &[worth_ui_host_headless::UiHeadlessMountedFrameTranscript],
    mounted_instances: &[worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity],
    entities: &[worth_query::facade::foundation::WorthQueryEntityIdentity],
    inserted: worth_query::facade::foundation::WorthQueryEntityIdentity,
) {
    assert_eq!(transcripts.len(), 5);
    assert_collection_transcript(
        &transcripts[0],
        mounted_instances,
        entities,
        &["Alpha", "Bravo"],
    );
    assert_collection_transcript(
        &transcripts[1],
        mounted_instances,
        &entities[1..],
        &["Bravo"],
    );
    assert_collection_transcript(
        &transcripts[2],
        mounted_instances,
        &entities[1..],
        &["Bravo updated"],
    );
    let inserted_entities = [inserted, entities[1].clone()];
    assert_collection_transcript(
        &transcripts[3],
        mounted_instances,
        &inserted_entities,
        &["Aaron", "Bravo updated"],
    );
    assert_collection_transcript(
        &transcripts[4],
        mounted_instances,
        &inserted_entities,
        &["Aaron", "Bravo final"],
    );
    assert_ne!(transcripts[3].frame(), transcripts[4].frame());
}

fn close_collection(
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

fn open_live_collection(
    registration: UiCollectionProjectionRegistration,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> worth_ui_query_binding::UiCollectionProjectionOpenReceipt {
    let binding = match registration.admit(&*workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("real collection binding admits: {stop:?}")
        }
    };
    match binding.open(
        UiCollectionProjectionBudget::new(2, 2, 0, 1024).unwrap(),
        workspace,
    ) {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => {
            panic!("real collection projection opens: {stop:?}")
        }
    }
}

fn publish_collection_fact(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    observation: worth_ui_query_binding::UiCollectionProjectionObservation,
    request: u64,
) {
    let plan = collection_plan(session, observation);
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(request))
        .unwrap_or_else(|error| {
            panic!("collection request {request} prepares against mounted authority: {error:?}")
        });
    match prepared.execute(request) {
        UiRebindOutcome::Published(receipt) => {
            assert!(receipt.application_publication().is_none());
            assert!(receipt.mounted_publication().is_some());
        }
        _ => panic!("collection content publishes atomically"),
    }
}

pub(crate) fn collection_plan(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    observation: worth_ui_query_binding::UiCollectionProjectionObservation,
) -> worth_ui::facade::rebind::UiRebindPlan {
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(UiProjectionObservation::Collection(observation))
        .unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("the real collection fact changes mounted presentation"),
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap()
}

fn assert_collection_transcript(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    mounted_instances: &[worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity],
    entities: &[worth_query::facade::foundation::WorthQueryEntityIdentity],
    expected_values: &[&str],
) {
    assert_eq!(transcript.semantic_text().len(), expected_values.len() + 1);
    let values = transcript
        .semantic_text()
        .iter()
        .filter(|row| matches!(row.slot(), UiSemanticTextSlot::CollectionValue { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        values.iter().map(|row| row.text()).collect::<Vec<_>>(),
        expected_values
    );
    assert_eq!(
        values
            .iter()
            .map(|row| {
                row.collection_row()
                    .expect("collection value retains row correlation")
                    .correlation_digest()
            })
            .collect::<Vec<_>>(),
        entities
            .iter()
            .map(|entity| entity
                .evidence_identity()
                .operational_key()
                .correlation_digest())
            .collect::<Vec<_>>()
    );
    assert!(values
        .iter()
        .all(|row| mounted_instances.contains(&row.mounted_instance())));
    let posture = transcript
        .semantic_text()
        .iter()
        .find(|row| row.slot() == UiSemanticTextSlot::Posture)
        .expect("collection posture is independently mounted");
    assert_eq!(posture.text(), "CURRENT · COMPLETE");
    assert!(posture.collection_row().is_none());
    assert_eq!(
        transcript.unperformed_effects(),
        &[UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 1,
            portal_overlay_count: 0,
            semantic_text_count: u32::try_from(expected_values.len() + 1)
                .expect("certification row count fits the host contract"),
            preview_node_count: 0,
        }]
    );
}

pub(crate) fn collection_app(
    registration: UiCollectionProjectionRegistration,
    recorder: WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    worth_ui::facade::app::WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor(ACTIVE_COMPONENT))
        .register_mosaic_region_kind(status_region_descriptor())
        .register_theme_token(text_token_descriptor())
        .register_collection_projection(registration)
        .expect("product collection projection registers")
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([
            collection_module(false),
        ]))
        .freeze()
        .map(|application| {
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
                application,
                recorder,
            )
        })
        .expect("collection content application freezes")
}

pub(crate) fn collection_module(with_region: bool) -> WorthUiRustAuthoredArtifactInputModule {
    let mut body = vec![
        WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier(PROJECTION.to_owned()),
    ];
    if with_region {
        body.extend([
            WorthUiArtifactInputBodyAtom::Identifier("region".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(STATUS_REGION.to_owned()),
            WorthUiArtifactInputBodyAtom::LeftBrace,
            WorthUiArtifactInputBodyAtom::RightBrace,
        ]);
    }
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component_body_atoms_and_authored_identity(
            ACTIVE_COMPONENT,
            "platform-pulse-projected-collection-component",
            body,
        )
        .with_token(TEXT_COLOR, "#ffffff")
        .try_with_query_collection_text(
            PROJECTION,
            PROJECTION,
            "identity.id",
            WorthUiProjectionCollectionSelection::new(
                ["status"],
                WorthUiProjectionLifecycle::Live,
                WorthUiProjectionCollectionPolicy::new(false, false),
            ),
        )
        .unwrap()
}

pub(crate) fn collection_registration(
    domain: &worth_ui_query_binding::WorthUiInstalledQueryDomain,
) -> UiCollectionProjectionRegistration {
    UiCollectionProjectionRegistration::text(
        domain.projection_view(PROJECTION).unwrap(),
        UiProjectionFieldRequirement::identity_id(),
        [UiProjectionFieldRequirement::query_text_status()],
        false,
        false,
    )
    .unwrap()
}
