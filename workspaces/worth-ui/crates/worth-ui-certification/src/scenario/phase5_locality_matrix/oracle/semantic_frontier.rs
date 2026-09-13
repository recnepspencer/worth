use std::collections::HashSet;

use worth_ui_host_native::{
    UiNativeClientConditionalOutcome, UiNativeClientPresentationSemanticChange as SemanticChange,
    UiNativeClientShutdownObservation, UiNativePresentationObservation,
};

use super::{require, Phase5LocalityAxis, LOCAL_SIGNAL_FRONTIER};
use crate::scenario::phase5_locality_matrix::case::Phase5LocalityCase;

#[path = "semantic_frontier/fixture_identity.rs"]
mod fixture_identity;
#[path = "semantic_frontier/identity_model.rs"]
mod identity_model;
#[path = "semantic_frontier/mechanic_evidence.rs"]
mod mechanic_evidence;

pub(super) fn adjudicate(
    case: Phase5LocalityCase,
    shutdown: &UiNativeClientShutdownObservation,
    presentation: &UiNativePresentationObservation,
    physical: &worth_ui_host_native::UiNativePhysicalSignalTransitionObservation,
) -> Result<&'static [SemanticChange], String> {
    let expected = expected_changes(case.axis());
    for &change in expected {
        let frontier = shutdown
            .presentation_semantic_frontiers()
            .iter()
            .rev()
            .find(|frontier| frontier.change() == change)
            .ok_or_else(|| format!("missing {change:?} semantic frontier"))?;
        require(
            !frontier.performed_counter_rows().is_empty(),
            "semantic frontier omitted performed Signal counters",
        )?;
        let expected_subscribers =
            expected_subscriber_count(case.axis(), change, case.retained_paragraphs());
        let expected_deliveries =
            expected_source_deliveries(case.axis(), change, expected_subscribers);
        require_subscriber_cardinality(change, frontier, expected_subscribers)?;
        require(
            frontier.performed_counter_rows().len() == frontier.subscribers().len()
                && frontier.outcomes().len() == frontier.subscribers().len(),
            "semantic evidence rows are not in one-to-one correspondence with subscribers",
        )?;
        require_exact_subscribers(
            case,
            frontier,
            presentation,
            physical,
            shutdown.authored_mounted_instances(),
            expected_subscribers,
            expected_deliveries,
        )?;
        require(
            frontier
                .outcomes()
                .iter()
                .all(|outcome| *outcome == UiNativeClientConditionalOutcome::ComputedChanged),
            "semantic frontier contains a non-performed or unchanged outcome",
        )?;
        require_scope_rejections(case, change, frontier.scope_rejections())?;
        if frontier.source_deliveries() as usize != expected_deliveries {
            return Err(format!(
                "{change:?} semantic source delivery breadth: expected {expected_deliveries}, observed {}",
                frontier.source_deliveries(),
            ));
        }
        require(
            frontier
                .performed_counter_rows()
                .iter()
                .all(|row| *row == LOCAL_SIGNAL_FRONTIER),
            "performed Signal frontier widened beyond the independent local model",
        )?;
    }
    Ok(expected)
}

fn require_scope_rejections(
    case: Phase5LocalityCase,
    change: SemanticChange,
    observed: [u64; 4],
) -> Result<(), String> {
    let expected =
        expected_scope_rejections(case.axis(), change, case.retained_paragraphs() as u64);
    require(
        observed == expected,
        "WUI aspect/partition/detail/range rejection counters disagree with the exact model",
    )
}

fn require_subscriber_cardinality(
    change: SemanticChange,
    frontier: &worth_ui_host_native::UiNativeClientPresentationSemanticFrontierObservation,
    expected: usize,
) -> Result<(), String> {
    if frontier.subscribers().len() == expected {
        return Ok(());
    }
    Err(format!(
        "{change:?} semantic frontier subscriber cardinality: expected {expected}, observed {} (active={}, removal={}, source_deliveries={}, frames={:?}, unique={})",
        frontier.subscribers().len(),
        frontier
            .subscribers()
            .iter()
            .filter(|subscriber| !subscriber.removal())
            .count(),
        frontier
            .subscribers()
            .iter()
            .filter(|subscriber| subscriber.removal())
            .count(),
        frontier.source_deliveries(),
        frontier
            .subscribers()
            .iter()
            .map(|subscriber| subscriber.mounted_frame())
            .collect::<HashSet<_>>(),
        subscriber_identities(frontier).len(),
    ))
}

fn require_exact_subscribers(
    case: Phase5LocalityCase,
    frontier: &worth_ui_host_native::UiNativeClientPresentationSemanticFrontierObservation,
    presentation: &UiNativePresentationObservation,
    physical: &worth_ui_host_native::UiNativePhysicalSignalTransitionObservation,
    authored_mounted_instances: &[worth_ui_host_native::UiNativeClientAuthoredMountedInstanceObservation],
    expected: usize,
    expected_deliveries: usize,
) -> Result<(), String> {
    require(
        subscriber_identities(frontier).len() == expected,
        "semantic frontier repeated or collapsed a subscriber identity",
    )?;
    require(
        frontier
            .subscribers()
            .iter()
            .all(|subscriber| subscriber.mounted_frame() == presentation.presented_frame()),
        "semantic subscriber is not bound to the presented successor frame",
    )?;
    require(
        frontier.subscribers().iter().all(|subscriber| {
            subscriber.removal() == (case.axis() == Phase5LocalityAxis::PinRelease)
        }),
        "semantic subscriber posture disagrees with the requested axis",
    )?;
    require_exact_request_and_dependency_identity(
        case,
        frontier,
        presentation,
        physical,
        expected,
        expected_deliveries,
    )?;
    mechanic_evidence::require_exact(case, frontier, authored_mounted_instances)
}

fn require_exact_request_and_dependency_identity(
    case: Phase5LocalityCase,
    frontier: &worth_ui_host_native::UiNativeClientPresentationSemanticFrontierObservation,
    presentation: &UiNativePresentationObservation,
    physical: &worth_ui_host_native::UiNativePhysicalSignalTransitionObservation,
    expected_subscribers: usize,
    expected_deliveries: usize,
) -> Result<(), String> {
    let subscribers = frontier.subscribers();
    if let Some(subscriber) = subscribers.iter().find(|subscriber| {
        subscriber.attempt() != presentation.presentation_attempt()
            || subscriber.binding() != presentation.binding_generation()
            || subscriber.semantic_surface() != presentation.semantic_surface()
            || subscriber.host_surface() != presentation.host_surface()
            || subscriber.host_surface() != physical.host_surface()
            || subscriber.host_lineage() != physical.host_session()
    }) {
        return Err(format!(
            "semantic source identity mismatch: subscriber=(attempt={},semantic_surface={},host_surface={},binding={},lineage={}) expected=(attempt={},semantic_surface={},host_surface={},binding={},physical_host={})",
            subscriber.attempt(),
            subscriber.semantic_surface(),
            subscriber.host_surface(),
            subscriber.binding(),
            subscriber.host_lineage(),
            presentation.presentation_attempt(),
            presentation.semantic_surface(),
            presentation.host_surface(),
            presentation.binding_generation(),
            physical.host_session(),
        ));
    }
    for subscriber in subscribers {
        identity_model::require_exact(*subscriber, frontier.change())?;
        fixture_identity::require_exact(case, *subscriber)?;
    }
    let sources = subscribers
        .iter()
        .map(|subscriber| subscriber.source_digest())
        .collect::<HashSet<_>>();
    require(
        sources.len() == expected_subscribers && !sources.contains(&[0; 32]),
        "semantic source identity set disagrees with the independent subscriber model",
    )?;
    let dependencies = subscribers
        .iter()
        .map(|subscriber| subscriber.immediate_dependency_digest())
        .collect::<HashSet<_>>();
    require(
        dependencies.len() == expected_deliveries && !dependencies.contains(&[0; 32]),
        "immediate dependency identity set disagrees with modeled source deliveries",
    )?;
    require(
        subscribers.iter().all(|subscriber| {
            subscriber.layout_digest() != [0; 32] && subscriber.raster_key_set_digest() != [0; 32]
        }),
        "semantic subscriber omitted its layout or raster-key-set identity",
    )
}

fn expected_scope_rejections(
    axis: Phase5LocalityAxis,
    change: SemanticChange,
    paragraphs: u64,
) -> [u64; 4] {
    match (axis, change) {
        (Phase5LocalityAxis::PaintBoundary, SemanticChange::PaintBoundary) => {
            [28 * paragraphs, 4 * paragraphs - 2, 0, 1]
        }
        (Phase5LocalityAxis::Dpi, SemanticChange::Dpi) => {
            [42 * paragraphs, 2 * paragraphs, 2 * paragraphs, 0]
        }
        (Phase5LocalityAxis::UploadCompletion, SemanticChange::Content) => [
            56 * paragraphs * paragraphs,
            8 * paragraphs * paragraphs - 4 * paragraphs,
            2 * paragraphs,
            0,
        ],
        (Phase5LocalityAxis::UploadCompletion, SemanticChange::UploadCompletion) => {
            [56 * paragraphs, 0, 6 * paragraphs, 0]
        }
        (Phase5LocalityAxis::PinRelease, SemanticChange::PinRelease) if paragraphs == 1 => {
            [70, 0, 8, 0]
        }
        (Phase5LocalityAxis::PinRelease, SemanticChange::PinRelease) => {
            [56 * paragraphs, 0, 8 * paragraphs - 2, 0]
        }
        _ => [56 * paragraphs, 8 * paragraphs - 4, 2, 0],
    }
}

fn subscriber_identities(
    frontier: &worth_ui_host_native::UiNativeClientPresentationSemanticFrontierObservation,
) -> HashSet<(Option<u64>, Option<u16>, Option<[u8; 32]>, u64, bool)> {
    frontier
        .subscribers()
        .iter()
        .map(|subscriber| {
            (
                subscriber.mounted_instance(),
                subscriber.semantic_slot(),
                subscriber.collection_row(),
                subscriber.mounted_frame(),
                subscriber.removal(),
            )
        })
        .collect()
}

fn expected_subscriber_count(
    axis: Phase5LocalityAxis,
    change: SemanticChange,
    retained_size: usize,
) -> usize {
    match (axis, change) {
        (Phase5LocalityAxis::Dpi, SemanticChange::Dpi)
        | (Phase5LocalityAxis::UploadCompletion, SemanticChange::Content)
        | (Phase5LocalityAxis::UploadCompletion, SemanticChange::UploadCompletion) => {
            retained_size * 2
        }
        (Phase5LocalityAxis::Content, SemanticChange::Content)
        | (Phase5LocalityAxis::Width, SemanticChange::Width)
        | (Phase5LocalityAxis::AtlasMiss, SemanticChange::Content)
        | (Phase5LocalityAxis::PinRelease, SemanticChange::PinRelease) => 2,
        _ => 1,
    }
}

fn expected_source_deliveries(
    axis: Phase5LocalityAxis,
    change: SemanticChange,
    subscribers: usize,
) -> usize {
    match (axis, change) {
        (Phase5LocalityAxis::Content, SemanticChange::Content)
        | (Phase5LocalityAxis::Width, SemanticChange::Width)
        | (Phase5LocalityAxis::AtlasMiss, SemanticChange::Content)
        | (Phase5LocalityAxis::PinRelease, SemanticChange::PinRelease)
        | (Phase5LocalityAxis::UploadCompletion, SemanticChange::Content) => subscribers,
        (Phase5LocalityAxis::UploadCompletion, SemanticChange::UploadCompletion) => 2,
        _ => 1,
    }
}

fn expected_changes(axis: Phase5LocalityAxis) -> &'static [SemanticChange] {
    match axis {
        Phase5LocalityAxis::Content | Phase5LocalityAxis::AtlasMiss => &[SemanticChange::Content],
        Phase5LocalityAxis::Width => &[SemanticChange::Width],
        Phase5LocalityAxis::PaintBoundary => &[SemanticChange::PaintBoundary],
        Phase5LocalityAxis::Dpi => &[SemanticChange::Dpi],
        Phase5LocalityAxis::UploadCompletion => {
            &[SemanticChange::Content, SemanticChange::UploadCompletion]
        }
        Phase5LocalityAxis::PinRelease => &[SemanticChange::PinRelease],
    }
}
