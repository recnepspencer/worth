use std::fmt;
use std::rc::Rc;

use super::WorthUiPreparedApplicationGenerationIdentity;
use crate::facade::registry::snapshot::CapabilitySnapshot;
use crate::runtime::{
    WorthUiAdmittedReplacementCandidate, WorthUiExecutionLaneSupport,
    WorthUiReplacementCandidateBasis,
};

struct WorthUiPreparedApplicationLoweringFacts {
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    generation_witness: super::WorthUiPreparedApplicationGenerationWitness,
    source_candidate_basis: WorthUiReplacementCandidateBasis,
    source_artifact_authority: Rc<crate::source::WorthUiArtifact>,
    graph_authority_identity: crate::graph::UiGraphAuthorityIdentity,
    capability_snapshot: Rc<CapabilitySnapshot>,
    query_binding_plan: worth_ui_query_binding::WorthUiQueryBindingPlan,
    execution_lane_support: WorthUiExecutionLaneSupport,
}

pub(super) struct WorthUiPreparedApplicationLoweringInput {
    pub(super) generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    pub(super) source_candidate_basis: WorthUiReplacementCandidateBasis,
    pub(super) source_artifact_authority: Rc<crate::source::WorthUiArtifact>,
    pub(super) graph_authority_identity: crate::graph::UiGraphAuthorityIdentity,
    pub(super) capability_snapshot: Rc<CapabilitySnapshot>,
    pub(super) query_binding_plan: worth_ui_query_binding::WorthUiQueryBindingPlan,
}

/// Retained exact authority for every prepared constituent that can affect
/// execution-plan lowering. Cloning this value shares one sealed authority; it
/// does not recreate authority from comparison-safe identities or digests.
#[derive(Clone)]
pub(crate) struct WorthUiPreparedApplicationLoweringAuthority {
    facts: Rc<WorthUiPreparedApplicationLoweringFacts>,
}

impl WorthUiPreparedApplicationLoweringAuthority {
    pub(super) fn seal(input: WorthUiPreparedApplicationLoweringInput) -> Self {
        let WorthUiPreparedApplicationLoweringInput {
            generation_identity,
            source_candidate_basis,
            source_artifact_authority,
            graph_authority_identity,
            capability_snapshot,
            query_binding_plan,
        } = input;
        let execution_lane_support = WorthUiExecutionLaneSupport::for_prepared_application(
            !query_binding_plan.is_query_free(),
        );
        Self {
            facts: Rc::new(WorthUiPreparedApplicationLoweringFacts {
                generation_identity,
                generation_witness: super::WorthUiPreparedApplicationGenerationWitness::issue(),
                source_candidate_basis,
                source_artifact_authority,
                graph_authority_identity,
                capability_snapshot,
                query_binding_plan,
                execution_lane_support,
            }),
        }
    }

    pub(crate) fn shares_authority_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.facts, &other.facts)
    }

    pub(crate) fn admits_candidate(&self, admitted: &WorthUiAdmittedReplacementCandidate) -> bool {
        self.facts.source_candidate_basis == admitted.candidate().basis()
            && admitted
                .artifact_bundle()
                .shares_artifact_authority_with(&self.facts.source_artifact_authority)
            && self.facts.capability_snapshot.digest().as_u64()
                == admitted.candidate().lowering_basis().snapshot_digest()
    }

    pub(crate) fn admits_launch_artifact(
        &self,
        artifact: &crate::source::WorthUiArtifact,
        artifact_digest: crate::source::WorthUiArtifactDigest,
    ) -> bool {
        self.facts.source_candidate_basis.artifact_digest() == artifact_digest
            && std::ptr::eq(self.facts.source_artifact_authority.as_ref(), artifact)
    }

    pub(crate) fn generation_identity(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.facts.generation_identity
    }

    pub(crate) fn capability_snapshot_digest(&self) -> crate::capability::CapabilitySnapshotDigest {
        self.facts.capability_snapshot.digest()
    }

    pub(crate) fn generation_witness(&self) -> super::WorthUiPreparedApplicationGenerationWitness {
        self.facts.generation_witness.clone()
    }

    pub(crate) fn graph_authority_identity(&self) -> crate::graph::UiGraphAuthorityIdentity {
        self.facts.graph_authority_identity
    }

    pub(crate) fn query_binding_plan(&self) -> &worth_ui_query_binding::WorthUiQueryBindingPlan {
        &self.facts.query_binding_plan
    }

    pub(crate) fn execution_lane_support(&self) -> &WorthUiExecutionLaneSupport {
        &self.facts.execution_lane_support
    }

    pub(crate) fn mosaic_state_capabilities(
        &self,
    ) -> &crate::capability::FrozenMosaicStateCapabilities {
        self.facts.capability_snapshot.mosaic_state_slots()
    }

    pub(crate) fn mosaic_sizing_capabilities(
        &self,
    ) -> &crate::capability::FrozenMosaicSizingCapabilities {
        self.facts.capability_snapshot.mosaic_sizing_contracts()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn synthetic_successor_for_certification(
        &self,
        admitted: &WorthUiAdmittedReplacementCandidate,
    ) -> Self {
        Self::seal(WorthUiPreparedApplicationLoweringInput {
            generation_identity: self.facts.generation_identity.clone(),
            source_candidate_basis: admitted.candidate().basis(),
            source_artifact_authority: admitted.artifact_bundle().artifact_authority(),
            graph_authority_identity: self.facts.graph_authority_identity,
            capability_snapshot: Rc::clone(&self.facts.capability_snapshot),
            query_binding_plan: self.facts.query_binding_plan.clone(),
        })
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn synthetic_launch_for_certification(
        &self,
        artifact: Rc<crate::source::WorthUiArtifact>,
        artifact_digest: crate::source::WorthUiArtifactDigest,
    ) -> Self {
        Self::seal(WorthUiPreparedApplicationLoweringInput {
            generation_identity: self.facts.generation_identity.clone(),
            source_candidate_basis: WorthUiReplacementCandidateBasis::new(artifact_digest, 0, 0),
            source_artifact_authority: artifact,
            graph_authority_identity: self.facts.graph_authority_identity,
            capability_snapshot: Rc::clone(&self.facts.capability_snapshot),
            query_binding_plan: self.facts.query_binding_plan.clone(),
        })
    }
}

impl fmt::Debug for WorthUiPreparedApplicationLoweringAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthUiPreparedApplicationLoweringAuthority")
            .field("generation_identity", &self.facts.generation_identity)
            .field(
                "graph_authority_identity",
                &self.facts.graph_authority_identity,
            )
            .field(
                "capability_snapshot_digest",
                &self.facts.capability_snapshot.digest(),
            )
            .field("query_binding_plan", &self.facts.query_binding_plan)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthUiPreparedApplicationLoweringAuthority {
    fn eq(&self, other: &Self) -> bool {
        self.shares_authority_with(other)
    }
}

impl Eq for WorthUiPreparedApplicationLoweringAuthority {}
