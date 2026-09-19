use std::fmt;
use std::time::Instant;

use crate::adjudication::{adjudicate_action_control_point, IntentControlPointFailure};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::native_platform::{NativePlatformContract, NativePlatformFailure};
use crate::source_delta::{IntentRouteRemovalSourceDelta, PulseCausalActionCursor};

use super::{FinalRecovered, Published, PulseExecutableWorld};

mod evidence;
mod observation;
mod states;
mod transitions;

use evidence::PlatformPulseIntentJourneyEvidenceBuilder;
pub(crate) use evidence::{
    PlatformPulseIntentCausalPulseEvidence, PlatformPulseIntentJourneyEvidence,
};

#[derive(Debug)]
pub(crate) enum PlatformPulseIntentJourneyFailure {
    Native(NativePlatformFailure),
    ControlPoint(IntentControlPointFailure),
    Observation(observation::IntentObservationFailure),
    SourceAction(crate::source_delta::PulseSourceActionFailure),
    Cancellation(String),
    EvidenceOrder(&'static str),
    CausalManifest(crate::source_delta::PulseCausalActionManifestFailure),
    EvidenceIncomplete,
}

pub(crate) struct CompletedPlatformPulseIntentJourney {
    recovered: PulseExecutableWorld<Published<FinalRecovered>>,
    evidence: PlatformPulseIntentJourneyEvidence,
}

pub(crate) struct CompletedPlatformPulseIntentCausalPulse {
    recovered: PulseExecutableWorld<Published<FinalRecovered>>,
    evidence: PlatformPulseIntentCausalPulseEvidence,
}

impl PulseExecutableWorld<Published<FinalRecovered>> {
    pub(crate) fn complete_intent_causal_pulse(
        self,
    ) -> Result<CompletedPlatformPulseIntentCausalPulse, PulseExecutableWorldFailureReport> {
        let Published { mut world, stage } = self.state;
        let result = (|| {
            let baseline = world
                .platform
                .capture_client_area(&world.native_client)
                .map_err(PlatformPulseIntentJourneyFailure::Native)?;
            let action = adjudicate_action_control_point(&baseline)
                .map_err(PlatformPulseIntentJourneyFailure::ControlPoint)?;
            let mut evidence = PlatformPulseIntentJourneyEvidenceBuilder::default();
            transitions::run_causal_pulse(&mut world, &baseline, action, &mut evidence)?;
            evidence
                .finish_causal_pulse()
                .ok_or(PlatformPulseIntentJourneyFailure::EvidenceIncomplete)
        })();
        match result {
            Ok(evidence) => Ok(CompletedPlatformPulseIntentCausalPulse {
                recovered: PulseExecutableWorld {
                    state: Published { world, stage },
                },
                evidence,
            }),
            Err(failure) => Err(teardown_native_bound_world(
                PulseExecutableWorldFailure::IntentJourney(failure),
                world.into_failure_resources(),
            )),
        }
    }

    pub(crate) fn complete_intent_journey(
        self,
        route_removal: IntentRouteRemovalSourceDelta,
    ) -> Result<CompletedPlatformPulseIntentJourney, PulseExecutableWorldFailureReport> {
        let mut actions = transitions::UntrackedIntentCausalActions;
        self.complete_intent_journey_with_actions(route_removal, &mut actions)
    }

    pub(crate) fn complete_intent_journey_for_manifest(
        self,
        route_removal: IntentRouteRemovalSourceDelta,
        cursor: &mut PulseCausalActionCursor<'_>,
    ) -> Result<CompletedPlatformPulseIntentJourney, PulseExecutableWorldFailureReport> {
        self.complete_intent_journey_with_actions(route_removal, cursor)
    }

    fn complete_intent_journey_with_actions(
        self,
        route_removal: IntentRouteRemovalSourceDelta,
        actions: &mut dyn transitions::IntentCausalActionAuthority,
    ) -> Result<CompletedPlatformPulseIntentJourney, PulseExecutableWorldFailureReport> {
        let Published { mut world, stage } = self.state;
        let result = (|| {
            actions
                .advance("capture-intent-baseline")
                .map_err(PlatformPulseIntentJourneyFailure::CausalManifest)?;
            let baseline = world
                .platform
                .capture_client_area(&world.native_client)
                .map_err(PlatformPulseIntentJourneyFailure::Native)?;
            let action = adjudicate_action_control_point(&baseline)
                .map_err(PlatformPulseIntentJourneyFailure::ControlPoint)?;
            let mut evidence = PlatformPulseIntentJourneyEvidenceBuilder::default();
            transitions::run(
                &mut world,
                &baseline,
                action,
                &mut evidence,
                route_removal,
                actions,
            )?;
            evidence
                .finish()
                .ok_or(PlatformPulseIntentJourneyFailure::EvidenceIncomplete)
        })();
        match result {
            Ok(evidence) => Ok(CompletedPlatformPulseIntentJourney {
                recovered: PulseExecutableWorld {
                    state: Published { world, stage },
                },
                evidence,
            }),
            Err(failure) => Err(teardown_native_bound_world(
                PulseExecutableWorldFailure::IntentJourney(failure),
                world.into_failure_resources(),
            )),
        }
    }
}

impl CompletedPlatformPulseIntentCausalPulse {
    pub(crate) fn evidence(&self) -> &PlatformPulseIntentCausalPulseEvidence {
        &self.evidence
    }

    pub(crate) fn into_recovered(self) -> PulseExecutableWorld<Published<FinalRecovered>> {
        self.recovered
    }

    pub(crate) fn native_journey_started(&self) -> Instant {
        self.recovered.native_journey_started()
    }
}

impl CompletedPlatformPulseIntentJourney {
    pub(crate) fn evidence(&self) -> &PlatformPulseIntentJourneyEvidence {
        &self.evidence
    }

    pub(crate) fn into_recovered(self) -> PulseExecutableWorld<Published<FinalRecovered>> {
        self.recovered
    }

    pub(crate) fn native_journey_started(&self) -> Instant {
        self.recovered.native_journey_started()
    }
}

impl fmt::Display for PlatformPulseIntentJourneyFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(failure) => write!(formatter, "native capture: {failure}"),
            Self::ControlPoint(failure) => write!(formatter, "control point: {failure}"),
            Self::Observation(failure) => write!(formatter, "observation: {failure}"),
            Self::SourceAction(failure) => write!(formatter, "product source action: {failure}"),
            Self::Cancellation(detail) => write!(formatter, "rebind cancellation: {detail}"),
            Self::EvidenceOrder(detail) => write!(formatter, "causal evidence order: {detail}"),
            Self::CausalManifest(failure) => write!(formatter, "causal manifest: {failure}"),
            Self::EvidenceIncomplete => {
                formatter.write_str("intent journey did not produce its complete evidence set")
            }
        }
    }
}

impl From<observation::IntentObservationFailure> for PlatformPulseIntentJourneyFailure {
    fn from(failure: observation::IntentObservationFailure) -> Self {
        Self::Observation(failure)
    }
}
