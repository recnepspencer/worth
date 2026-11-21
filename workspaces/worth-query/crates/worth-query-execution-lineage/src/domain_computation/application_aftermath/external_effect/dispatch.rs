//! Pure dispatch posture and causal-ladder models.

use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::causal_event::ExternalEffectPosture;
use super::classification::{ExternalEffectClassification, ExternalRailTransportFault};
use super::correlation::ExternalEffectCorrelationIdentity;
use super::proven_no_effect::ExternalEffectProvenNoEffect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchPostureKind {
    Completed,
    Acknowledged,
    ProvenNoEffect,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalDispatchPosture {
    state: DispatchPostureState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DispatchPostureState {
    Completed(ExternalEffectPosture),
    Acknowledged(ExternalEffectPosture),
    ProvenNoEffect(ExternalEffectProvenNoEffect),
    Unresolved(ExternalEffectClassification),
}

impl WorthQueryExternalDispatchPosture {
    pub const fn completed(observation: ExternalEffectPosture) -> Self {
        Self {
            state: DispatchPostureState::Completed(observation),
        }
    }

    pub const fn acknowledged(observation: ExternalEffectPosture) -> Self {
        Self {
            state: DispatchPostureState::Acknowledged(observation),
        }
    }

    pub const fn from_proven_no_effect(proof: ExternalEffectProvenNoEffect) -> Self {
        Self {
            state: DispatchPostureState::ProvenNoEffect(proof),
        }
    }

    pub const fn unresolved(classification: ExternalEffectClassification) -> Self {
        Self {
            state: DispatchPostureState::Unresolved(classification),
        }
    }

    pub const fn kind(&self) -> WorthQueryExternalDispatchPostureKind {
        match self.state {
            DispatchPostureState::Completed(_) => WorthQueryExternalDispatchPostureKind::Completed,
            DispatchPostureState::Acknowledged(_) => {
                WorthQueryExternalDispatchPostureKind::Acknowledged
            }
            DispatchPostureState::ProvenNoEffect(_) => {
                WorthQueryExternalDispatchPostureKind::ProvenNoEffect
            }
            DispatchPostureState::Unresolved(_) => {
                WorthQueryExternalDispatchPostureKind::Unresolved
            }
        }
    }

    pub const fn observation(&self) -> Option<&ExternalEffectPosture> {
        match &self.state {
            DispatchPostureState::Completed(observation)
            | DispatchPostureState::Acknowledged(observation) => Some(observation),
            DispatchPostureState::ProvenNoEffect(_) | DispatchPostureState::Unresolved(_) => None,
        }
    }

    pub const fn classification(&self) -> Option<&ExternalEffectClassification> {
        match &self.state {
            DispatchPostureState::Unresolved(classification) => Some(classification),
            DispatchPostureState::Completed(_)
            | DispatchPostureState::Acknowledged(_)
            | DispatchPostureState::ProvenNoEffect(_) => None,
        }
    }

    pub const fn proven_no_effect(&self) -> Option<&ExternalEffectProvenNoEffect> {
        match &self.state {
            DispatchPostureState::ProvenNoEffect(proof) => Some(proof),
            DispatchPostureState::Completed(_)
            | DispatchPostureState::Acknowledged(_)
            | DispatchPostureState::Unresolved(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalEffectCausalLadder {
    provider_commit: ExternalEffectPosture,
    emission: ExternalEffectPosture,
    attempt: ExternalEffectPosture,
    observation: Option<ExternalEffectPosture>,
}

impl WorthQueryExternalEffectCausalLadder {
    pub const fn from_parts(
        provider_commit: ExternalEffectPosture,
        emission: ExternalEffectPosture,
        attempt: ExternalEffectPosture,
        observation: Option<ExternalEffectPosture>,
    ) -> Self {
        Self {
            provider_commit,
            emission,
            attempt,
            observation,
        }
    }

    pub const fn provider_commit(&self) -> &ExternalEffectPosture {
        &self.provider_commit
    }

    pub const fn emission(&self) -> &ExternalEffectPosture {
        &self.emission
    }

    pub const fn attempt(&self) -> &ExternalEffectPosture {
        &self.attempt
    }

    pub const fn observation(&self) -> Option<&ExternalEffectPosture> {
        self.observation.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalEffectDispatch {
    live_runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    correlation: ExternalEffectCorrelationIdentity,
    posture: WorthQueryExternalDispatchPosture,
    causal_ladder: WorthQueryExternalEffectCausalLadder,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchCausalRelation {
    SameAttempt,
    FreshAttemptForSameEmission,
    Unrelated,
}

impl WorthQueryExternalEffectDispatch {
    pub const fn from_parts(
        live_runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        correlation: ExternalEffectCorrelationIdentity,
        posture: WorthQueryExternalDispatchPosture,
        causal_ladder: WorthQueryExternalEffectCausalLadder,
        canonical_work: WorthQueryCanonicalWorkEvidence,
    ) -> Self {
        Self {
            live_runtime,
            correlation,
            posture,
            causal_ladder,
            canonical_work,
        }
    }

    pub(crate) const fn live_runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.live_runtime
    }

    pub const fn correlation(&self) -> &ExternalEffectCorrelationIdentity {
        &self.correlation
    }

    pub const fn posture(&self) -> &WorthQueryExternalDispatchPosture {
        &self.posture
    }

    pub const fn causal_ladder(&self) -> &WorthQueryExternalEffectCausalLadder {
        &self.causal_ladder
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }

    pub const fn is_external_completion(&self) -> bool {
        matches!(
            self.posture.kind(),
            WorthQueryExternalDispatchPostureKind::Completed
        )
    }

    pub const fn fault(&self) -> Option<ExternalRailTransportFault> {
        match self.posture.classification() {
            Some(classification) => Some(classification.fault()),
            None => None,
        }
    }

    pub fn matches_committed_owner(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        committed: &crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
    ) -> bool {
        if self.live_runtime != runtime {
            return false;
        }
        let ladder = &self.causal_ladder;
        let Some(observation) = ladder.observation() else {
            return false;
        };
        let expected =
            match super::identity_derivation::provider_commit_identity(runtime, committed) {
                Ok(expected) => expected,
                Err(_) => return false,
            };
        self.is_external_completion()
            && self.correlation == *committed.record().correlation()
            && ladder.provider_commit().kind() == super::ExternalEffectPostureKind::ProviderCommit
            && ladder.provider_commit().identity().digest() == &expected.digest
            && ladder.emission().kind()
                == super::ExternalEffectPostureKind::EmittedApplicationCausality
            && ladder
                .emission()
                .predecessor()
                .is_some_and(|link| link.predecessor() == ladder.provider_commit().identity())
            && ladder.attempt().kind() == super::ExternalEffectPostureKind::DispatchAttempt
            && ladder
                .attempt()
                .predecessor()
                .is_some_and(|link| link.predecessor() == ladder.emission().identity())
            && observation.kind() == super::ExternalEffectPostureKind::ExternalCompletion
            && observation
                .predecessor()
                .is_some_and(|link| link.predecessor() == ladder.attempt().identity())
            && self.posture.observation() == Some(observation)
    }

    pub fn causal_relation_to(&self, later: &Self) -> WorthQueryExternalDispatchCausalRelation {
        let same_emission = self.correlation == later.correlation
            && self.causal_ladder.provider_commit.identity()
                == later.causal_ladder.provider_commit.identity()
            && self.causal_ladder.emission.identity() == later.causal_ladder.emission.identity();
        if !same_emission {
            WorthQueryExternalDispatchCausalRelation::Unrelated
        } else if self.causal_ladder.attempt.identity() == later.causal_ladder.attempt.identity() {
            WorthQueryExternalDispatchCausalRelation::SameAttempt
        } else {
            WorthQueryExternalDispatchCausalRelation::FreshAttemptForSameEmission
        }
    }
}

mod court_doors_for_test;
