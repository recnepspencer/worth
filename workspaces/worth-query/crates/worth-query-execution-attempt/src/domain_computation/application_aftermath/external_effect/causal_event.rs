//! Compiler-enforced progression of the external-effect causal ladder.

use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::super::WorthQueryAftermathDerivationFailure;
use super::dispatch_admission::{
    DispatchAttemptCoordinate, WorthQueryAdmittedExternalDispatchAttempt,
};
use super::identity::{ExternalEffectCausalLink, ExternalEffectPostureIdentity};
use super::identity_derivation::{
    attempt_identity, emission_identity, observation_identity, provider_commit_identity,
};
use super::{ExternalEffectPostureKind, WorthQueryDispatchOutboxRecord};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

pub use worth_query_execution_lineage::facade::application_aftermath::external_effect::causal_event::{
    CausalConstructionAuthority, ExternalEffectPosture, CAUSAL_CONSTRUCTION,
};

struct ProviderCommitEvent {
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    posture: ExternalEffectPosture,
    committed: WorthQueryCommittedDispatchOutboxObservation,
}

pub struct CoCommittedApplicationEmission {
    provider_commit: ProviderCommitEvent,
    posture: ExternalEffectPosture,
}

pub struct DispatchAttemptEvent<'emission> {
    emission: &'emission CoCommittedApplicationEmission,
    posture: ExternalEffectPosture,
}

pub struct ExternalCompletionEvent {
    posture: ExternalEffectPosture,
}

pub struct ExternalAcknowledgementEvent {
    posture: ExternalEffectPosture,
}

pub fn with_admitted_dispatch<Output>(
    admitted: WorthQueryAdmittedExternalDispatchAttempt<'_>,
    continue_with: impl FnOnce(
        &DispatchAttemptEvent<'_>,
        &WorthQueryRuntimeClock,
        WorthQueryCanonicalWorkEvidence,
    ) -> Result<Output, WorthQueryAftermathDerivationFailure>,
) -> Result<Output, WorthQueryAftermathDerivationFailure> {
    let WorthQueryAdmittedExternalDispatchAttempt {
        committed,
        query_runtime,
        coordinate,
        clock,
    } = admitted;
    let (emission, emission_work) = admit_co_committed_emission(query_runtime, committed)?;
    let (attempt, attempt_work) = begin_dispatch_attempt(&emission, &coordinate)?;
    continue_with(&attempt, &clock, emission_work.combine(attempt_work))
}

fn admit_co_committed_emission(
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
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
        runtime,
        posture: ExternalEffectPosture::initial(
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

fn begin_dispatch_attempt<'emission>(
    emission: &'emission CoCommittedApplicationEmission,
    coordinate: &DispatchAttemptCoordinate<'_>,
) -> Result<
    (
        DispatchAttemptEvent<'emission>,
        WorthQueryCanonicalWorkEvidence,
    ),
    WorthQueryAftermathDerivationFailure,
> {
    let derived = attempt_identity(
        emission.posture.identity(),
        emission.record().correlation(),
        coordinate,
    )?;
    let posture = ExternalEffectPosture::successor(
        ExternalEffectPostureKind::DispatchAttempt,
        ExternalEffectPostureIdentity::from_digest(&CAUSAL_CONSTRUCTION, derived.digest),
        &emission.posture,
    );
    Ok((DispatchAttemptEvent { emission, posture }, derived.work))
}

pub fn observe_completion(
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

pub fn observe_acknowledgement(
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
    pub const fn provider_commit(&self) -> &ExternalEffectPosture {
        &self.provider_commit.posture
    }

    pub const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        self.provider_commit.committed.record()
    }
}

impl DispatchAttemptEvent<'_> {
    pub fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.emission.provider_commit.runtime
    }

    pub const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        self.emission.record()
    }

    pub const fn provider_commit(&self) -> &ExternalEffectPosture {
        self.emission.provider_commit()
    }

    pub const fn emission(&self) -> &ExternalEffectPosture {
        self.emission.posture()
    }

    pub const fn attempt(&self) -> &ExternalEffectPosture {
        &self.posture
    }
}

impl ExternalCompletionEvent {
    pub const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub fn into_posture(self) -> ExternalEffectPosture {
        self.posture
    }
}

impl ExternalAcknowledgementEvent {
    pub const fn posture(&self) -> &ExternalEffectPosture {
        &self.posture
    }

    pub fn into_posture(self) -> ExternalEffectPosture {
        self.posture
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::CanonicalDigestId;

    use super::*;

    #[test]
    fn owner_observation_identity_binds_its_exact_attempt_predecessor() {
        let correlation = super::super::ExternalEffectCorrelationIdentity::from_digest(
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
