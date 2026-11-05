//! Compiler-enforced progression of the external-effect causal ladder.

use std::sync::Arc;

use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::super::WorthQueryAftermathDerivationFailure;
use super::identity::{ExternalEffectCausalLink, ExternalEffectPostureIdentity};
use super::identity_derivation::{
    attempt_identity, emission_identity, observation_identity, provider_commit_identity,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use crate::domain_computation::vocabulary::primary_graph::external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal;
use {
    super::ExternalEffectPostureKind,
    crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord,
};

/// Read-only projection of one causal stage. Only the typed transitions in this module can seal it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEffectPosture {
    kind: ExternalEffectPostureKind,
    identity: ExternalEffectPostureIdentity,
    predecessor: Option<ExternalEffectCausalLink>,
}

impl ExternalEffectPosture {
    fn root(kind: ExternalEffectPostureKind, identity: ExternalEffectPostureIdentity) -> Self {
        Self {
            kind,
            identity,
            predecessor: None,
        }
    }

    fn successor(
        kind: ExternalEffectPostureKind,
        identity: ExternalEffectPostureIdentity,
        predecessor: &Self,
    ) -> Self {
        Self {
            kind,
            identity,
            predecessor: Some(ExternalEffectCausalLink::to(predecessor.identity())),
        }
    }

    pub const fn kind(&self) -> ExternalEffectPostureKind {
        self.kind
    }

    pub const fn identity(&self) -> &ExternalEffectPostureIdentity {
        &self.identity
    }

    pub const fn predecessor(&self) -> Option<&ExternalEffectCausalLink> {
        self.predecessor.as_ref()
    }
}

/// Private identity-minting capability used only by the exact stage transitions below.
pub(super) struct CausalConstructionAuthority {
    _private: (),
}

const CAUSAL_CONSTRUCTION: CausalConstructionAuthority =
    CausalConstructionAuthority { _private: () };

struct ProviderCommitEvent {
    posture: ExternalEffectPosture,
    committed: WorthQueryCommittedDispatchOutboxObservation,
}

pub(super) struct CoCommittedApplicationEmission {
    provider_commit: ProviderCommitEvent,
    posture: ExternalEffectPosture,
}

pub(super) struct DispatchAttemptEvent<'emission> {
    emission: &'emission CoCommittedApplicationEmission,
    posture: ExternalEffectPosture,
}

pub(super) struct ExternalCompletionEvent {
    posture: ExternalEffectPosture,
}

pub(super) struct ExternalAcknowledgementEvent {
    posture: ExternalEffectPosture,
}

/// Move-only authority for one exact runtime-admitted physical attempt.
///
/// Only the runtime can mint the opaque ordinal required by `seal`, and only
/// this causal owner can decompose the completed admission.
pub struct WorthQueryAdmittedExternalDispatchAttempt {
    committed: WorthQueryCommittedDispatchOutboxObservation,
    query_runtime: crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity,
    ordinal: WorthQueryExternalDispatchAttemptOrdinal,
    clock: Arc<WorthQueryRuntimeClock>,
}

impl WorthQueryAdmittedExternalDispatchAttempt {
    pub(in crate::domain_computation) fn seal(
        committed: WorthQueryCommittedDispatchOutboxObservation,
        query_runtime: crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity,
        ordinal: WorthQueryExternalDispatchAttemptOrdinal,
        clock: Arc<WorthQueryRuntimeClock>,
    ) -> Self {
        Self {
            committed,
            query_runtime,
            ordinal,
            clock,
        }
    }

    pub const fn ordinal_for_test(&self) -> u64 {
        self.ordinal.value_for_test()
    }
}

pub(super) fn with_admitted_dispatch<Output>(
    admitted: WorthQueryAdmittedExternalDispatchAttempt,
    continue_with: impl FnOnce(
        &DispatchAttemptEvent<'_>,
        &WorthQueryRuntimeClock,
        WorthQueryCanonicalWorkEvidence,
    ) -> Result<Output, WorthQueryAftermathDerivationFailure>,
) -> Result<Output, WorthQueryAftermathDerivationFailure> {
    let WorthQueryAdmittedExternalDispatchAttempt {
        committed,
        query_runtime,
        ordinal,
        clock,
    } = admitted;
    let (emission, emission_work) = admit_co_committed_emission(query_runtime, committed)?;
    let (attempt, attempt_work) = begin_dispatch_attempt(&emission, ordinal.into_value())?;
    continue_with(&attempt, &clock, emission_work.combine(attempt_work))
}

fn admit_co_committed_emission(
    runtime: crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity,
    committed: WorthQueryCommittedDispatchOutboxObservation,
) -> Result<
    (
        CoCommittedApplicationEmission,
        WorthQueryCanonicalWorkEvidence,
    ),
    WorthQueryAftermathDerivationFailure,
> {
    let provider = provider_commit_identity(runtime, &committed)?;
    let provider_commit = ProviderCommitEvent {
        posture: ExternalEffectPosture::root(
            ExternalEffectPostureKind::ProviderCommit,
            ExternalEffectPostureIdentity::from_digest(&CAUSAL_CONSTRUCTION, provider.digest),
        ),
        committed,
    };
    let emitted = emission_identity(
        provider_commit.posture.identity(),
        provider_commit.committed.record().correlation(),
        provider_commit.committed.record().outcome_identity(),
    )?;
    let posture = ExternalEffectPosture::successor(
        ExternalEffectPostureKind::EmittedApplicationCausality,
        ExternalEffectPostureIdentity::from_digest(&CAUSAL_CONSTRUCTION, emitted.digest),
        &provider_commit.posture,
    );
    Ok((
        CoCommittedApplicationEmission {
            provider_commit,
            posture,
        },
        provider.work.combine(emitted.work),
    ))
}

fn begin_dispatch_attempt(
    emission: &CoCommittedApplicationEmission,
    attempt_ordinal: u64,
) -> Result<
    (DispatchAttemptEvent<'_>, WorthQueryCanonicalWorkEvidence),
    WorthQueryAftermathDerivationFailure,
> {
    let derived = attempt_identity(
        emission.posture.identity(),
        emission.record().correlation(),
        attempt_ordinal,
    )?;
    let posture = ExternalEffectPosture::successor(
        ExternalEffectPostureKind::DispatchAttempt,
        ExternalEffectPostureIdentity::from_digest(&CAUSAL_CONSTRUCTION, derived.digest),
        &emission.posture,
    );
    Ok((DispatchAttemptEvent { emission, posture }, derived.work))
}

pub(super) fn observe_completion(
    attempt: &DispatchAttemptEvent<'_>,
) -> Result<
    (ExternalCompletionEvent, WorthQueryCanonicalWorkEvidence),
    WorthQueryAftermathDerivationFailure,
> {
    let (posture, work) = observe_attempt(
        attempt,
        ExternalEffectPostureKind::ExternalCompletion,
        "completed",
    )?;
    Ok((ExternalCompletionEvent { posture }, work))
}

pub(super) fn observe_acknowledgement(
    attempt: &DispatchAttemptEvent<'_>,
) -> Result<
    (
        ExternalAcknowledgementEvent,
        WorthQueryCanonicalWorkEvidence,
    ),
    WorthQueryAftermathDerivationFailure,
> {
    let (posture, work) = observe_attempt(
        attempt,
        ExternalEffectPostureKind::ExternalAcknowledgement,
        "acknowledged",
    )?;
    Ok((ExternalAcknowledgementEvent { posture }, work))
}

fn observe_attempt(
    attempt: &DispatchAttemptEvent<'_>,
    kind: ExternalEffectPostureKind,
    observation: &'static str,
) -> Result<
    (ExternalEffectPosture, WorthQueryCanonicalWorkEvidence),
    WorthQueryAftermathDerivationFailure,
> {
    let derived = observation_identity(
        attempt.posture.identity(),
        attempt.emission.record().correlation(),
        observation,
    )?;
    Ok((
        ExternalEffectPosture::successor(
            kind,
            ExternalEffectPostureIdentity::from_digest(&CAUSAL_CONSTRUCTION, derived.digest),
            &attempt.posture,
        ),
        derived.work,
    ))
}

impl CoCommittedApplicationEmission {
    pub(super) const fn provider_commit(&self) -> &ExternalEffectPosture {
        &self.provider_commit.posture
    }

    pub(super) const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub(super) const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        self.provider_commit.committed.record()
    }
}

impl DispatchAttemptEvent<'_> {
    pub(super) const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        self.emission.record()
    }

    pub(super) const fn provider_commit(&self) -> &ExternalEffectPosture {
        self.emission.provider_commit()
    }

    pub(super) const fn emission(&self) -> &ExternalEffectPosture {
        self.emission.posture()
    }

    pub(super) const fn attempt(&self) -> &ExternalEffectPosture {
        &self.posture
    }
}

impl ExternalCompletionEvent {
    pub(super) const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub(super) fn into_posture(self) -> ExternalEffectPosture {
        self.posture
    }
}

impl ExternalAcknowledgementEvent {
    pub(super) const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub(super) fn into_posture(self) -> ExternalEffectPosture {
        self.posture
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::CanonicalDigestId;

    use super::*;

    #[test]
    fn owner_observation_identity_binds_its_exact_attempt_predecessor() {
        let correlation = crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity::from_digest(
            CanonicalDigestId::new([9; 32]),
        );
        let first = ExternalEffectPostureIdentity::from_digest(
            &CAUSAL_CONSTRUCTION,
            CanonicalDigestId::new([1; 32]),
        );
        let second = ExternalEffectPostureIdentity::from_digest(
            &CAUSAL_CONSTRUCTION,
            CanonicalDigestId::new([2; 32]),
        );

        let first_observation = observation_identity(&first, &correlation, "completed").unwrap();
        let second_observation = observation_identity(&second, &correlation, "completed").unwrap();

        assert_ne!(first_observation.digest, second_observation.digest);
    }
}
