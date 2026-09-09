mod application;
mod fixture_profile;

pub(super) use fixture_profile::{MixedCarrierFixtureProfile, CLOSURE, SMOKE};

use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiProjectionConsumptionBudget, UiProjectionFieldRequirement,
    UiProjectionObservation, UiScalarProjectionBatchOutcome, UiScalarProjectionBindingAdmission,
    UiScalarProjectionRegistration, WorthUiQueryWorkspaceExt,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use crate::projection_lifecycle::async_fixture::{
    admitted_async_request_and_completion, authoritative_async_basis, projection_bridge,
    scalar_async_view_named,
};
use crate::projection_presentation::collection_query::collection_registration;

const REMOVAL_INDEX: usize = 2;
const SCALAR_PROJECTION: &str = "host.platform.mixed.scalar.view";

pub(super) struct MixedCarrierProduction {
    pub initial: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub text_replacement: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub rectangle_removal: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub rectangle_insertion: worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    pub costs: [worth_ui_host_contract::UiMountedPresentationProductionCost; 5],
    pub adapter_costs: [worth_ui_host_contract::UiHostPresentationCostReport; 5],
}

struct MountedMixedRows {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    rows: Vec<MountedMixedRow>,
}

struct MountedMixedRow {
    node: worth_ui_runtime::facade::mounted::UiMountedGraphNodeHandle,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
}

pub(super) fn produce(
    recorder: worth_ui_host_headless::WorthUiHeadlessRecorder,
    profile: MixedCarrierFixtureProfile,
) -> MixedCarrierProduction {
    let (mut workspace, entities) = seeded_workspace(profile);
    let collection = collection_registration(&workspace.worth_ui().unwrap());
    let (scalar, scalar_observation) = scalar_observation(&mut workspace);
    let mut session = application::build(profile, collection.clone(), scalar, recorder.clone())
        .launch()
        .expect("mixed carrier application launches");
    let mut mounted = application::mount(profile, &mut session);
    super::world::establish_allocations(&mut session, profile.rectangle_component_count);
    let opened = open_collection(profile, collection, &mut workspace);
    let (mut live, snapshot) = opened.into_parts();

    let _staging_adapter = publish(
        &mut session,
        vec![UiProjectionObservation::Collection(
            snapshot.into_observation(),
        )],
        3_140,
    );
    let staging = one_transcript(&recorder, "mixed collection staging");
    assert_eq!(staging.filled_rects().len(), profile.rectangle_count);
    assert_eq!(staging.semantic_text().len(), profile.collection_rows + 1);
    let initial_adapter = publish(
        &mut session,
        vec![UiProjectionObservation::Scalar(scalar_observation)],
        3_141,
    );
    let initial = one_transcript(&recorder, "mixed initial");
    let initial_cost = latest_cost(&recorder);
    assert_initial_ceiling(profile, &initial);

    worth_ui_query_binding::certification::update_projection_status(
        &mut workspace,
        entities[profile.collection_rows - 1].clone(),
        &replacement_value(profile, profile.collection_rows - 1),
    );
    let text_adapter = refresh(&mut live, &mut workspace, &mut session, 3_142);
    let text_replacement = one_transcript(&recorder, "mixed text replacement");
    let text_cost = latest_cost(&recorder);

    session
        .unmount_instance(mounted.rows[REMOVAL_INDEX].instance)
        .unwrap();
    let removal_adapter = super::world::execute_frame(&mut session, 3_143);
    let rectangle_removal = one_transcript(&recorder, "mixed rectangle removal");
    let removal_cost = latest_cost(&recorder);

    let removed = &mut mounted.rows[REMOVAL_INDEX];
    removed.instance = session
        .mount_instance(removed.node, mounted.surface)
        .unwrap();
    let insertion_adapter = super::world::execute_frame(&mut session, 3_144);
    let rectangle_insertion = one_transcript(&recorder, "mixed rectangle insertion");
    let insertion_cost = latest_cost(&recorder);

    let unchanged_adapter = super::world::execute_frame(&mut session, 3_145);
    assert!(recorder.drain_transcripts().is_empty());
    let unchanged_cost = latest_cost(&recorder);
    close(live, &mut workspace);
    let shutdown = session.shutdown();
    assert!(shutdown.mounted_presentation().is_empty());
    MixedCarrierProduction {
        initial,
        text_replacement,
        rectangle_removal,
        rectangle_insertion,
        costs: [
            initial_cost,
            text_cost,
            removal_cost,
            insertion_cost,
            unchanged_cost,
        ],
        adapter_costs: [
            initial_adapter,
            text_adapter,
            removal_adapter,
            insertion_adapter,
            unchanged_adapter,
        ],
    }
}

fn scalar_observation(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> (
    UiScalarProjectionRegistration,
    worth_ui_query_binding::UiScalarProjectionObservation,
) {
    let bridge = projection_bridge();
    let (request, completion) = admitted_async_request_and_completion(
        &bridge,
        NodeId::new(31_410, 0),
        authoritative_async_basis("mixed-commit", "mixed-snapshot"),
        64,
    );
    let view = scalar_async_view_named(workspace, &request, SCALAR_PROJECTION);
    let registration = UiScalarProjectionRegistration::text(
        workspace
            .worth_ui()
            .unwrap()
            .projection_view(SCALAR_PROJECTION)
            .unwrap(),
        UiProjectionFieldRequirement::declared("status").unwrap(),
    );
    let mut binding = match registration.clone().admit(&*workspace) {
        UiScalarProjectionBindingAdmission::Ready(binding) => binding,
        other => panic!("mixed scalar binding did not admit: {other:?}"),
    };
    let pending = binding
        .consume_initial_async_result(
            workspace,
            &view,
            UiProjectionConsumptionBudget::platform_pulse(),
        )
        .unwrap()
        .into_fact_and_predecessor()
        .0;
    let ordering = bridge.order_mixed_causes(
        &worth_runtime_bridge::facade::BridgeMixedCauseOrderingRequest::new(
            worth_runtime_bridge::facade::BridgeMixedCauseOrderingLaneKind::Authoritative,
            vec![BridgeMixedCauseOrderingInput::AsyncCompletion(completion)],
        ),
    );
    let batch = workspace
        .admit_bridge_async_result_transitions(&view, &ordering)
        .unwrap();
    let receipt = match binding.consume_async_result_batch(
        workspace,
        batch,
        Some(pending),
        UiProjectionConsumptionBudget::platform_pulse(),
    ) {
        UiScalarProjectionBatchOutcome::Advanced(receipt) => receipt,
        UiScalarProjectionBatchOutcome::Unchanged(_) => panic!("mixed scalar must advance"),
    };
    (
        registration,
        receipt.into_fact_and_predecessor().0.into_observation(),
    )
}

fn seeded_workspace(
    profile: MixedCarrierFixtureProfile,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    let rows = (0..profile.collection_rows)
        .map(|index| (format!("mixed.{index:04}"), initial_value(profile, index)))
        .collect();
    worth_ui_query_binding::certification::seeded_mixed_projection_workspace(rows)
}

pub(super) fn initial_value(profile: MixedCarrierFixtureProfile, index: usize) -> String {
    if index == 0 {
        return "Ready".to_owned();
    }
    let len = if index + 1 == profile.collection_rows {
        profile.final_text_bytes
    } else {
        profile.ordinary_text_bytes
    };
    format!("A{index:04}{}", "x".repeat(len - 5))
}

pub(super) fn replacement_value(profile: MixedCarrierFixtureProfile, index: usize) -> String {
    let mut value = initial_value(profile, index).into_bytes();
    value[0] = b'B';
    String::from_utf8(value).unwrap()
}

fn open_collection(
    profile: MixedCarrierFixtureProfile,
    registration: worth_ui_query_binding::UiCollectionProjectionRegistration,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> worth_ui_query_binding::UiCollectionProjectionOpenReceipt {
    let binding = match registration.admit(&*workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("binding stopped: {stop:?}")
        }
    };
    match binding.open(
        UiCollectionProjectionBudget::new(
            profile.collection_rows as u32,
            profile.text_count,
            0,
            profile.text_bytes * 2,
        )
        .unwrap(),
        workspace,
    ) {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => panic!("open stopped: {stop:?}"),
    }
}

fn refresh(
    live: &mut worth_ui_query_binding::UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: u64,
) -> worth_ui_host_contract::UiHostPresentationCostReport {
    let fact = match live.refresh(workspace).unwrap() {
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::Applied(receipt) => {
            receipt.into_fact()
        }
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("mixed replacement must produce a Query patch")
        }
    };
    publish(
        session,
        vec![UiProjectionObservation::Collection(fact.into_observation())],
        request,
    )
}

fn publish(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    observations: Vec<UiProjectionObservation>,
    request: u64,
) -> worth_ui_host_contract::UiHostPresentationCostReport {
    let mut turn = session.begin_observation_turn().unwrap();
    for observation in observations {
        turn.admit_projection_query(observation).unwrap();
    }
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("mixed observation must change the mounted projection"),
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(request))
        .unwrap();
    match prepared.execute(request) {
        UiRebindOutcome::Published(receipt) => receipt
            .realized_mount_cost()
            .expect("mixed rebind publishes mounted work")
            .adapter(),
        UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "mixed request {request} rejected: {:?}",
            denial
                .host_rejections()
                .iter()
                .map(|rejection| rejection.denial())
                .collect::<Vec<_>>()
        ),
        _ => panic!("mixed request {request} did not publish"),
    }
}

fn assert_initial_ceiling(
    profile: MixedCarrierFixtureProfile,
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) {
    assert_eq!(transcript.filled_rects().len(), profile.rectangle_count);
    assert_eq!(transcript.semantic_text().len(), profile.text_count);
    let bytes = transcript
        .semantic_text()
        .iter()
        .map(|row| row.text().len() + usize::from(row.collection_row().is_some()) * 32)
        .sum::<usize>();
    assert_eq!(bytes, profile.text_bytes);
}

pub(super) fn text_bytes(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) -> usize {
    transcript
        .semantic_text()
        .iter()
        .map(|row| row.text().len() + usize::from(row.collection_row().is_some()) * 32)
        .sum()
}

pub(super) fn collection_value_count(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    value: &str,
) -> usize {
    transcript
        .semantic_text()
        .iter()
        .filter(|row| row.collection_row().is_some() && row.text() == value)
        .count()
}

fn one_transcript(
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
    context: &str,
) -> worth_ui_host_headless::UiHeadlessMountedFrameTranscript {
    let mut transcripts = recorder.drain_transcripts().into_vec();
    assert_eq!(transcripts.len(), 1, "{context}");
    transcripts.pop().unwrap()
}

fn latest_cost(
    recorder: &worth_ui_host_headless::WorthUiHeadlessRecorder,
) -> worth_ui_host_contract::UiMountedPresentationProductionCost {
    recorder
        .latest_production_cost()
        .expect("mixed presentation exposes exact production cost")
}

fn close(
    live: worth_ui_query_binding::UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) {
    match live.close(workspace) {
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Closed(_) => {}
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!("mixed collection close stopped: {:?}", stop.query_error())
        }
    }
}
