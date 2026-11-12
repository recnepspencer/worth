//! Recorded causal names are descriptions, never physical contact or live delivery.
use super::identity_derivation::{
    emission_digest, observation_digest, provider_commit_identity, recorded_attempt_digest,
};
use crate::domain_computation::application_aftermath::WorthQueryAftermathDerivationFailure;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use worth_foundational::facade::{CanonicalDigestId, DispatchAttemptIdentity};

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct RecordedCompletionCausality {
    attempt: CanonicalDigestId,
    observation: CanonicalDigestId,
}
impl RecordedCompletionCausality {
    /// Pure description from actual original meaning and recorded coordinates.
    /// The journal owner separately admits the recorded completion fact.
    pub(in crate::domain_computation) fn describe(
        runtime: WorthQueryRuntimeAuthorityIdentity,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        attempt: DispatchAttemptIdentity,
    ) -> Result<Self, WorthQueryAftermathDerivationFailure> {
        let home = owner
            .committed_product_publication()
            .product_branch()
            .owner_identity()
            .durable()
            .ok_or(WorthQueryAftermathDerivationFailure::BasisRejected)?
            .home();
        if home != attempt.home() {
            return Err(WorthQueryAftermathDerivationFailure::BasisRejected);
        }
        let provider = provider_commit_identity(runtime, owner)?;
        let emission = emission_digest(
            &provider.digest,
            owner.record().correlation(),
            owner.record().outcome_identity(),
        )?;
        let attempted =
            recorded_attempt_digest(&emission.digest, owner.record().correlation(), attempt)?;
        let observed =
            observation_digest(&attempted.digest, owner.record().correlation(), "completed")?;
        Ok(Self {
            attempt: attempted.digest,
            observation: observed.digest,
        })
    }
    pub(in crate::domain_computation) fn attempt(&self) -> &CanonicalDigestId {
        &self.attempt
    }
    pub(in crate::domain_computation) fn observation(&self) -> &CanonicalDigestId {
        &self.observation
    }
}
