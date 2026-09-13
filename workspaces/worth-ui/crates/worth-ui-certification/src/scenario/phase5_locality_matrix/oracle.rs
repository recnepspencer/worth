//! Certification-owned joins and independent locality adjudication.

use serde_json::Value;
use worth_ui_host_native::{
    UiNativePhysicalSignalExternalStatusClass, UiNativePhysicalSignalObservationOriginClass,
    UiNativePhysicalSignalSettlementClass, UiNativePhysicalSignalTransitionObservation,
    UiNativePhysicalSignalWorkClass,
};
use worth_ui_native_platform::{
    UiNativeClientPresentationTransitionKind as TransitionKind, UiNativePresentationWorkKind,
};

use super::case::Phase5LocalityAxis;
use super::dependency_model;
use super::execution::Phase5LocalityEvidence;

mod evidence_row;

pub(super) fn adjudicate(evidence: Phase5LocalityEvidence) -> Result<Value, String> {
    let case = evidence.case();
    let receipt = evidence.receipt();
    let shutdown = receipt
        .client_shutdown()
        .ok_or_else(|| "matrix world omitted Query shutdown observation".to_owned())?;
    require_complete_world(receipt, shutdown)?;
    require_unchanged_zero_work(receipt, shutdown)?;
    let completed = completed_resource_count(receipt, shutdown)?;
    let retained_successor = receipt
        .retained_frames()
        .iter()
        .rev()
        .find(|frame| frame.kind() != UiNativePresentationWorkKind::Unchanged)
        .ok_or_else(|| "native owner retained no physically presented successor".to_owned())?;
    let presentation = retained_successor
        .presentation()
        .ok_or_else(|| "retained successor omitted its owner-issued presentation".to_owned())?;
    let work = shutdown
        .text_presentation_work()
        .iter()
        .rev()
        .find(|work| {
            work.attempt() == presentation.presentation_attempt()
                && work.binding() == presentation.binding_generation()
        })
        .ok_or_else(|| "last native presentation has no exact runtime text-work join".to_owned())?;
    let physical = receipt
        .physical_signal_transition_observations()
        .iter()
        .rev()
        .find(|transition| {
            transition.attempt() == presentation.presentation_attempt()
                && transition.binding() == presentation.binding_generation()
        })
        .ok_or_else(|| "last presentation has no exact physical-Signal join".to_owned())?;
    require_exact_physical_signal(case.axis(), physical)?;
    require_successor_topology(case, retained_successor, presentation)?;
    dependency_model::adjudicate(
        case,
        shutdown.text_presentation_work(),
        receipt.text_atlas_plan_observations(),
    )?;
    super::presentation_cost_model::adjudicate(
        case,
        presentation.production_cost(),
        presentation.cost(),
    )
    .map_err(|denial| {
        format!(
            "{denial}; presentation scale={} extent={:?}",
            presentation.scale_factor_milli(),
            presentation.client_physical_size()
        )
    })?;
    Ok(evidence_row::assemble(
        &evidence,
        presentation,
        completed,
        work,
        physical,
    ))
}

fn require_exact_physical_signal(
    axis: Phase5LocalityAxis,
    observation: &UiNativePhysicalSignalTransitionObservation,
) -> Result<(), String> {
    let expected_revision = match axis {
        Phase5LocalityAxis::Dpi => 21,
        Phase5LocalityAxis::AtlasMiss | Phase5LocalityAxis::UploadCompletion => 18,
        _ => 14,
    };
    require(
        observation.host_session() == 1
            && observation.surface() == 1
            && observation.request_sequence() == 2
            && observation.work() == UiNativePhysicalSignalWorkClass::Presentation
            && observation.origin()
                == UiNativePhysicalSignalObservationOriginClass::NativeExternalPort
            && observation.external_status()
                == UiNativePhysicalSignalExternalStatusClass::Completed
            && observation.settlement() == UiNativePhysicalSignalSettlementClass::Completed
            && observation.performed_transitions() == 1
            && observation.performed_nodes() == 4
            && observation.fact_revision() == expected_revision
            && observation.read_scopes() == 4,
        "physical-Signal identity, class, settlement, or performed counters disagree with the exact model",
    )
}

fn require_complete_world(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
    shutdown: &worth_ui_native_platform::UiNativeClientShutdownObservation,
) -> Result<(), String> {
    require(
        receipt.event_loop_thread_posture()
            == worth_ui_host_native::UiNativeEventLoopThreadPosture::CertificationWorker,
        "matrix world did not disclose certification-worker event-loop posture",
    )?;
    require(
        receipt.event_loop_thread_matches_launch(),
        "matrix event-loop callback diverged from its qualified launch thread",
    )?;
    require(
        shutdown.managed_semantic_resources_complete(),
        "Query resources did not close",
    )?;
    require(
        shutdown.presentation_transition_trace_complete(),
        "Query transition trace overflowed",
    )?;
    require(
        shutdown.text_presentation_work_trace_complete(),
        "text work trace overflowed",
    )?;
    require(
        receipt.observation_history_complete(),
        "native observation history overflowed",
    )?;
    require(
        receipt.terminal_census().is_zero(),
        "native census is nonzero",
    )
}

fn require_unchanged_zero_work(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
    shutdown: &worth_ui_native_platform::UiNativeClientShutdownObservation,
) -> Result<(), String> {
    let unchanged = receipt
        .retained_frames()
        .iter()
        .filter(|frame| frame.kind() == UiNativePresentationWorkKind::Unchanged)
        .collect::<Vec<_>>();
    require(
        unchanged.len() == 1,
        "matrix world must retain exactly one unchanged turn",
    )?;
    let unchanged = unchanged[0];
    require(
        unchanged.cost() == Default::default(),
        "unchanged turn performed native presentation work",
    )?;
    require(
        unchanged
            .presentation()
            .is_some_and(|presentation| presentation.cost() == Default::default()),
        "unchanged turn omitted or widened its owner-issued physical observation",
    )?;
    require(
        shutdown
            .text_presentation_work()
            .iter()
            .all(|work| work.mounted_frame() != unchanged.frame()),
        "unchanged turn performed text qualification, raster, atlas, or pin work",
    )
}

fn completed_resource_count(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
    shutdown: &worth_ui_native_platform::UiNativeClientShutdownObservation,
) -> Result<usize, String> {
    let completed = shutdown
        .presentation_transitions()
        .iter()
        .filter(|transition| transition.kind() == TransitionKind::Completed)
        .count();
    if completed < 2 {
        return Err(format!(
            "initial and successor Query resources did not complete: transitions={:?} text-work={:?} retained-kinds={:?}",
            shutdown
                .presentation_transitions()
                .iter()
                .map(|transition| transition.kind())
                .collect::<Vec<_>>(),
            shutdown
                .text_presentation_work()
                .iter()
                .map(|work| (
                    work.mounted_frame(),
                    work.layout_count(),
                    work.demand_records()
                ))
                .collect::<Vec<_>>(),
            receipt
                .retained_frames()
                .iter()
                .map(|frame| frame.kind())
                .collect::<Vec<_>>(),
        ));
    }
    Ok(completed)
}

fn require_successor_topology(
    case: super::case::Phase5LocalityCase,
    retained: &worth_ui_host_native::UiNativeRetainedFrameObservation,
    presentation: &worth_ui_host_native::UiNativePresentationObservation,
) -> Result<(), String> {
    require(
        retained.frame() == presentation.presented_frame(),
        "retained frame and presentation observation disagree",
    )?;
    let expected_work_kind = match case.axis() {
        Phase5LocalityAxis::Dpi => UiNativePresentationWorkKind::Reconstruction,
        _ => UiNativePresentationWorkKind::Delta,
    };
    require(
        retained.kind() == expected_work_kind,
        "matrix successor used the wrong native presentation topology",
    )
}

fn require(condition: bool, message: &str) -> Result<(), String> {
    condition.then_some(()).ok_or_else(|| message.to_owned())
}
