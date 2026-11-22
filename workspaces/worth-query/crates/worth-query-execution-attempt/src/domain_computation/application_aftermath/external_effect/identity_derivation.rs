//! Delegates canonical external-effect causal identity derivation to the
//! single canonical owner in `worth-query-execution-lineage`.

use worth_query_execution_lineage::facade::domain_computation::application_aftermath::external_effect::identity_derivation as canonical;

pub(super) use canonical::{
    emission_digest, emission_identity, observation_digest, observation_identity,
    provider_commit_identity, recorded_attempt_digest, DerivedEventIdentity,
};

use super::super::WorthQueryAftermathDerivationFailure;
use super::identity::ExternalEffectPostureIdentity;
use super::ExternalEffectCorrelationIdentity;

pub(super) fn attempt_identity(
    predecessor: &ExternalEffectPostureIdentity,
    correlation: &ExternalEffectCorrelationIdentity,
    coordinate: &super::dispatch_admission::DispatchAttemptCoordinate<'_>,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    match coordinate {
        super::dispatch_admission::DispatchAttemptCoordinate::Memory(ordinal) => {
            canonical::memory_attempt_digest(predecessor.digest(), correlation, ordinal.value())
        }
        super::dispatch_admission::DispatchAttemptCoordinate::Recorded(contact) => {
            canonical::recorded_attempt_digest(
                predecessor.digest(),
                correlation,
                contact.recorded_attempt(),
            )
        }
    }
}
