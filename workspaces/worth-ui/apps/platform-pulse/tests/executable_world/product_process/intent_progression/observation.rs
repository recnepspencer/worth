use std::fmt;
use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentAttemptObservationReference, PlatformPulseIntentExecutorGateObservation,
    PlatformPulseIntentOperabilityObservation, PlatformPulseIntentPostureObservation,
    PlatformPulseLifecycleObservation, PlatformPulseQueryActionObservation,
    PlatformPulseQueryProjectionPosture,
};

use crate::adjudication::IntentControlPointFailure;
use crate::external_observation::{NativeClientPixelPoint, NativeInputProbeKind};
use crate::native_platform::{NativePlatformContract, NativePlatformFailure};

use super::super::{
    await_next_observation, await_watched_observation, NativeBoundExecutableWorld,
    WatchedPulseObservationFailure, WatchedPulseTransition,
};
use super::states::ConfirmationChallenge;

mod causal_trace;
mod visual;

pub(super) use visual::{
    await_visual_refresh, capture_visible_change, capture_visible_confirmation,
};

const TRANSITION_DEADLINE: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
pub(super) struct Sequenced<T> {
    pub(super) value: T,
    pub(super) sequence: u64,
}

#[derive(Clone, Copy)]
pub(super) enum ExpectedTerminalPosture {
    Denied,
    StaleConfirmation,
}

#[derive(Debug)]
pub(crate) enum IntentObservationFailure {
    Native(NativePlatformFailure),
    Watched(WatchedPulseObservationFailure),
    Unexpected {
        expected: &'static str,
        observed: String,
    },
    NativeDelivery(&'static str),
    IntentInput(&'static str),
    ExecutorStart(&'static str),
    QueryCompletion(String),
    Visible(IntentControlPointFailure),
}

pub(super) fn activate_native_control(
    world: &mut NativeBoundExecutableWorld,
    point: NativeClientPixelPoint,
) -> Result<(), IntentObservationFailure> {
    let client = world
        .platform
        .observe_bound_client_area(&world.native_client)
        .map_err(IntentObservationFailure::Native)?;
    let delivery = world
        .platform
        .deliver_pointer_activation(&world.native_client, point)
        .map_err(IntentObservationFailure::Native)?;
    if delivery.process_id() != world.process.id()
        || delivery.window() != client.window()
        || delivery.kind() != NativeInputProbeKind::Pointer
        || delivery.delivered_event_count() != 2
    {
        return Err(IntentObservationFailure::NativeDelivery(
            "pointer delivery did not remain one two-event activation on the bound child window",
        ));
    }
    let (client_x, client_y) = point.coordinates();
    let expected_x = client.bounds().left().saturating_add_unsigned(client_x);
    let expected_y = client.bounds().top().saturating_add_unsigned(client_y);
    let (actual_x, actual_y) = delivery.qualified_screen_point();
    if actual_x.abs_diff(expected_x) > point.landing_tolerance()
        || actual_y.abs_diff(expected_y) > point.landing_tolerance()
    {
        return Err(IntentObservationFailure::NativeDelivery(
            "pre-delivery cursor qualification disagreed with the pixel-derived control point",
        ));
    }

    Ok(())
}

pub(super) fn await_intent_input(
    world: &mut NativeBoundExecutableWorld,
    revision: u64,
    operability: PlatformPulseIntentOperabilityObservation,
    gate: PlatformPulseIntentExecutorGateObservation,
) -> Result<u64, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentInputAdmitted)?;
    let PlatformPulseLifecycleObservation::IntentInputAdmitted(observed) = envelope.outcome()
    else {
        return Err(unexpected("intent input admission", envelope.outcome()));
    };
    if observed.revision() != revision
        || observed.operability() != operability
        || observed.executor_gate() != gate
    {
        return Err(IntentObservationFailure::IntentInput(
            "admitted product input disagreed with its typed revision delta",
        ));
    }
    Ok(envelope.sequence().value())
}

pub(super) fn await_admitted(
    world: &mut NativeBoundExecutableWorld,
) -> Result<Sequenced<PlatformPulseIntentAttemptObservationReference>, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
    let PlatformPulseLifecycleObservation::IntentPosturePublished(published) = envelope.outcome()
    else {
        return Err(unexpected("admitted intent posture", envelope.outcome()));
    };
    let PlatformPulseIntentPostureObservation::Admitted { reference } = published.posture() else {
        return Err(unexpected("admitted intent posture", envelope.outcome()));
    };
    if published.frame().diagnostic_value() == 0 {
        return Err(IntentObservationFailure::IntentInput(
            "admitted posture omitted mounted frame identity",
        ));
    }
    Ok(Sequenced {
        value: *reference,
        sequence: envelope.sequence().value(),
    })
}

pub(super) fn await_confirmation_required(
    world: &mut NativeBoundExecutableWorld,
) -> Result<Sequenced<ConfirmationChallenge>, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
    let PlatformPulseLifecycleObservation::IntentPosturePublished(published) = envelope.outcome()
    else {
        return Err(unexpected(
            "confirmation-required posture",
            envelope.outcome(),
        ));
    };
    let PlatformPulseIntentPostureObservation::ConfirmationRequired {
        slot,
        generation,
        lineage,
        expires_at_millis,
    } = published.posture()
    else {
        return Err(unexpected(
            "confirmation-required posture",
            envelope.outcome(),
        ));
    };
    Ok(Sequenced {
        value: ConfirmationChallenge {
            slot: *slot,
            generation: *generation,
            lineage: *lineage,
            expires_at_millis: *expires_at_millis,
        },
        sequence: envelope.sequence().value(),
    })
}

pub(super) fn await_terminal_posture(
    world: &mut NativeBoundExecutableWorld,
    expected: ExpectedTerminalPosture,
) -> Result<u64, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
    let PlatformPulseLifecycleObservation::IntentPosturePublished(published) = envelope.outcome()
    else {
        return Err(unexpected("terminal intent posture", envelope.outcome()));
    };
    let matches = matches!(
        (expected, published.posture()),
        (
            ExpectedTerminalPosture::Denied,
            PlatformPulseIntentPostureObservation::Denied
        ) | (
            ExpectedTerminalPosture::StaleConfirmation,
            PlatformPulseIntentPostureObservation::StaleConfirmation
        )
    );
    if !matches {
        return Err(unexpected("terminal intent posture", envelope.outcome()));
    }
    Ok(envelope.sequence().value())
}

pub(super) fn await_executor_started(
    world: &mut NativeBoundExecutableWorld,
    expected: PlatformPulseIntentAttemptObservationReference,
) -> Result<u64, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentExecutorStarted)?;
    let PlatformPulseLifecycleObservation::IntentExecutorStarted(started) = envelope.outcome()
    else {
        return Err(unexpected("intent executor start", envelope.outcome()));
    };
    if started.reference() != expected
        || started.transition_count() != 1
        || started.active_slots_visited() != 1
        || started.provider_calls() != 1
        || started.provider_polls() != 0
        || started.cancellation_calls() != 0
        || started.settlements() != 0
    {
        return Err(IntentObservationFailure::ExecutorStart(
            "executor start did not prove one new live attempt with zero settlement",
        ));
    }
    Ok(envelope.sequence().value())
}

pub(super) fn await_query_completion(
    world: &mut NativeBoundExecutableWorld,
    expected_attempt: PlatformPulseIntentAttemptObservationReference,
    action_input_revision: u64,
    query_source_revision: u64,
    status: &str,
) -> Result<
    Sequenced<
        worth_ui_platform_pulse::observation_contract::PlatformPulseIntentCausalTraceObservation,
    >,
    IntentObservationFailure,
> {
    let issued_envelope = next(world, WatchedPulseTransition::QueryProjectionIssued)?;
    let PlatformPulseLifecycleObservation::QueryProjectionIssued(issued) =
        issued_envelope.outcome()
    else {
        return Err(unexpected(
            "Query projection issue",
            issued_envelope.outcome(),
        ));
    };
    if issued.posture() != PlatformPulseQueryProjectionPosture::Current
        || issued.native_value() != Some(status)
    {
        return Err(IntentObservationFailure::QueryCompletion(
            "issued Query consequence was not the expected current product value".to_owned(),
        ));
    }
    let issued = issued.clone();

    let action_envelope = next(world, WatchedPulseTransition::IntentQueryAction)?;
    let PlatformPulseLifecycleObservation::QueryAction(action) = action_envelope.outcome() else {
        return Err(unexpected("Query action", action_envelope.outcome()));
    };
    let PlatformPulseQueryActionObservation::Executed {
        reference,
        action_input_revision: observed_action_input_revision,
        query_source_revision: observed_query_source_revision,
        status: observed_status,
        query_receipt_digest,
        affected_live_view_ids,
    } = action
    else {
        return Err(unexpected(
            "executed Query action",
            action_envelope.outcome(),
        ));
    };
    if *reference != expected_attempt
        || *observed_action_input_revision != action_input_revision
        || *observed_query_source_revision != query_source_revision
        || observed_status != status
        || query_receipt_digest.is_empty()
        || affected_live_view_ids.is_empty()
    {
        return Err(IntentObservationFailure::QueryCompletion(format!(
            "Query action mismatch: expected reference={expected_attempt:?}, \
                 action_revision={action_input_revision}, query_revision={query_source_revision}, \
                 status={status:?}; observed reference={reference:?}, \
                 action_revision={observed_action_input_revision}, \
                 query_revision={observed_query_source_revision}, status={observed_status:?}, \
                 receipt_empty={}, affected_views={affected_live_view_ids:?}",
            query_receipt_digest.is_empty()
        )));
    }

    let published_envelope = next(world, WatchedPulseTransition::QueryProjectionPublished)?;
    let PlatformPulseLifecycleObservation::QueryProjectionPublished(published) =
        published_envelope.outcome()
    else {
        return Err(unexpected(
            "Query projection publication",
            published_envelope.outcome(),
        ));
    };
    if published.projection() != &issued || published.frame().diagnostic_value() == 0 {
        return Err(IntentObservationFailure::QueryCompletion(
            "3.12 publication did not carry the exact owner-issued Query projection".to_owned(),
        ));
    }

    let posture_envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
    let PlatformPulseLifecycleObservation::IntentPosturePublished(posture) =
        posture_envelope.outcome()
    else {
        return Err(unexpected(
            "completed intent posture",
            posture_envelope.outcome(),
        ));
    };
    if !matches!(
        posture.posture(),
        PlatformPulseIntentPostureObservation::Completed { reference }
            if *reference == expected_attempt
    ) {
        return Err(IntentObservationFailure::QueryCompletion(
            "completed posture did not preserve the admitted attempt lineage".to_owned(),
        ));
    }
    if posture.frame() != published.frame() {
        return Err(IntentObservationFailure::QueryCompletion(
            "completed posture and Query consequence named different mounted frames".to_owned(),
        ));
    }
    causal_trace::await_completed_causal_trace(world, expected_attempt, &issued, published.frame())
}

pub(super) fn next(
    world: &mut NativeBoundExecutableWorld,
    expected: WatchedPulseTransition,
) -> Result<
    worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope,
    IntentObservationFailure,
> {
    await_watched_observation(
        &mut world.process,
        &mut world.lifecycle,
        expected,
        Instant::now() + TRANSITION_DEADLINE,
    )
    .map_err(IntentObservationFailure::Watched)
}

pub(super) fn next_unfiltered(
    world: &mut NativeBoundExecutableWorld,
    expected: WatchedPulseTransition,
) -> Result<
    worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope,
    IntentObservationFailure,
> {
    await_next_observation(
        &mut world.process,
        &mut world.lifecycle,
        expected,
        Instant::now() + TRANSITION_DEADLINE,
    )
    .map_err(IntentObservationFailure::Watched)
}

fn unexpected(
    expected: &'static str,
    observed: &PlatformPulseLifecycleObservation,
) -> IntentObservationFailure {
    IntentObservationFailure::Unexpected {
        expected,
        observed: format!("{observed:?}"),
    }
}

impl fmt::Display for IntentObservationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(failure) => write!(formatter, "native observation: {failure}"),
            Self::Watched(failure) => write!(formatter, "lifecycle observation: {failure}"),
            Self::Unexpected { expected, observed } => {
                write!(formatter, "expected {expected}, observed {observed}")
            }
            Self::NativeDelivery(detail) => write!(formatter, "native delivery: {detail}"),
            Self::IntentInput(detail) => write!(formatter, "intent input: {detail}"),
            Self::ExecutorStart(detail) => write!(formatter, "executor start: {detail}"),
            Self::QueryCompletion(detail) => write!(formatter, "Query completion: {detail}"),
            Self::Visible(failure) => write!(formatter, "visible posture: {failure}"),
        }
    }
}
