//! Descriptive references derived only after the exact owner accepts the source.
use super::super::{
    compose_bridge_causal_envelope_evidence_identity as compose,
    digest_basis::BridgeCausalEnvelopeDigestArtifact as Artifact, evidence_part, shape_part,
    value_part, BridgeCausalEnvelopeDenial, BridgeCausalEvidenceFamily as Family,
    BridgeCausalEvidenceOwner as Owner, BridgeCausalEvidenceReference as Reference,
    BridgeCausalEvidenceReferenceIdentity as Identity,
};
use crate::conditional_execution::BridgeConditionalCausalObservation;
use crate::identity::BridgeIdentityEvidence;

pub(super) struct NativeEvidence<'a> {
    // Resolution always has the genuine immutable source, not only its digests.
    pub(super) observation: &'a BridgeConditionalCausalObservation,
    references: Vec<Reference>,
}

impl<'a> NativeEvidence<'a> {
    pub(super) fn from_verified(
        observation: &'a BridgeConditionalCausalObservation,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        let attempt = observation.attempt().to_string();
        let source = observation
            .bridge_snapshot_identity()
            .map(|source| source.as_str());
        let native = compose(
            Artifact::NativeObservation,
            &[
                value_part(observation.lowering_projection().label()),
                value_part(observation.signal_projection().label()),
                value_part(&attempt),
                shape_part(if source.is_some() {
                    "bound_source"
                } else {
                    "no_source"
                }),
                value_part(source.unwrap_or("")),
            ],
        );
        let evaluation = compose(
            Artifact::NativeSignalEvaluation,
            &[
                evidence_part(&native),
                value_part(observation.signal_projection().label()),
            ],
        );
        let mut references = vec![
            reference(
                Owner::RuntimeBridge,
                Family::BridgeConditionalObservation,
                native.clone(),
            )?,
            reference(Owner::Signal, Family::SignalEvaluation, evaluation)?,
        ];
        if observation.performed_signal_invalidation().is_some() {
            let invalidation = compose(
                Artifact::NativeSignalInvalidation,
                &[
                    evidence_part(&native),
                    shape_part("actual_performed_invalidation"),
                ],
            );
            references.push(reference(
                Owner::Signal,
                Family::SignalInvalidation,
                invalidation,
            )?);
        }
        Ok(Self {
            observation,
            references,
        })
    }

    pub(super) fn references_with_query(
        &self,
        query: BridgeIdentityEvidence,
    ) -> Result<Vec<Reference>, BridgeCausalEnvelopeDenial> {
        let mut references = Vec::with_capacity(self.references.len() + 1);
        references.push(reference(Owner::Query, Family::QueryObservation, query)?);
        references.extend(self.references.iter().cloned());
        Ok(references)
    }

    pub(super) fn contains(&self, reference: &Reference) -> bool {
        // Presence is checked against this actual source's derived references.
        // No digest from outside this context can substitute the source itself.
        self.references.iter().any(|issued| issued == reference)
            && (reference.family() != Family::SignalInvalidation
                || self.observation.performed_signal_invalidation().is_some())
    }
}

fn reference(
    owner: Owner,
    family: Family,
    identity: BridgeIdentityEvidence,
) -> Result<Reference, BridgeCausalEnvelopeDenial> {
    let identity = match owner {
        Owner::Query => Identity::query_observation(identity),
        Owner::RuntimeBridge => Identity::runtime_bridge(family, identity),
        Owner::Signal => Identity::signal(family, identity),
        Owner::Relational => Identity::relational_authority(identity),
    }?;
    Reference::new(owner, family, identity)
}
