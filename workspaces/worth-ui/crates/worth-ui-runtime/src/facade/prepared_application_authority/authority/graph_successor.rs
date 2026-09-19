use std::collections::BTreeSet;
use std::rc::Rc;

use super::super::generation_identity::WorthUiPreparedGenerationIdentityInput;
use super::super::lowering_authority::WorthUiPreparedApplicationLoweringInput;
use super::super::{
    WorthUiPreparedApplicationGenerationIdentity, WorthUiPreparedApplicationLoweringAuthority,
};
use super::WorthUiPreparedApplicationAuthority;

pub(crate) struct WorthUiPreparedApplicationGraphSuccessor {
    predecessor_authority: crate::graph::UiGraphAuthorityIdentity,
    graph_snapshot: crate::graph::UiGraphSnapshot,
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    generation_succession: WorthUiPreparedApplicationGenerationSuccession,
    lowering_authority: WorthUiPreparedApplicationLoweringAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiPreparedApplicationGenerationSuccession {
    predecessor: WorthUiPreparedApplicationGenerationIdentity,
    successor: WorthUiPreparedApplicationGenerationIdentity,
}

impl WorthUiPreparedApplicationGenerationSuccession {
    pub(crate) fn new(
        predecessor: WorthUiPreparedApplicationGenerationIdentity,
        successor: WorthUiPreparedApplicationGenerationIdentity,
    ) -> Self {
        Self {
            predecessor,
            successor,
        }
    }

    pub(crate) fn predecessor(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.predecessor
    }

    pub(crate) fn successor(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.successor
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiPreparedApplicationGraphSuccessorDenial {
    StaleGraphPredecessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiPreparedRebindGraphDenial {
    MountEligibility(crate::graph::UiGraphMountEligibilityAdmissionDenial),
}

impl WorthUiPreparedApplicationGraphSuccessor {
    pub(crate) fn graph_snapshot(&self) -> &crate::graph::UiGraphSnapshot {
        &self.graph_snapshot
    }

    pub(crate) fn lowering_authority(&self) -> WorthUiPreparedApplicationLoweringAuthority {
        self.lowering_authority.clone()
    }

    pub(crate) fn generation_succession(&self) -> WorthUiPreparedApplicationGenerationSuccession {
        self.generation_succession.clone()
    }
}

impl WorthUiPreparedApplicationAuthority {
    pub(crate) fn prepare_rebind_mount_eligibility(
        &mut self,
    ) -> Result<BTreeSet<crate::graph::UiGraphNodeIdentity>, WorthUiPreparedRebindGraphDenial> {
        let graph = crate::graph::UiGraphAuthority::new(&self.graph_snapshot);
        let mut changed_nodes = BTreeSet::new();
        let mut transitions = Vec::new();
        for node in graph.node_identities() {
            let record = graph
                .lookup()
                .graph_node(node)
                .expect("prepared graph node remains addressable");
            let record_value = record.value();
            let declaration = record_value.declaration_identity();
            let participates_in_allocation = self
                .declaration_artifacts
                .iter()
                .find(|artifact| artifact.identity() == declaration)
                .and_then(|artifact| artifact.graph_handoff().ok())
                .is_some_and(|handoff| handoff.measurement_policy().admitted().is_some());
            if !participates_in_allocation {
                continue;
            }
            let prior = record_value
                .participation_posture()
                .axis(crate::graph::UiGraphParticipationAxis::Mounted);
            if prior.status() == crate::graph::UiGraphParticipationStatus::Admitted {
                continue;
            }
            let transition = graph
                .mount_eligibility_transition_for_node(
                    node,
                    prior,
                    crate::graph::UiGraphAxisParticipation::runtime_mutation(
                        crate::graph::UiGraphParticipationStatus::Admitted,
                    ),
                )
                .expect("prepared graph node has one mount-eligibility slot");
            changed_nodes.insert(node);
            transitions.push(transition);
        }
        if transitions.is_empty() {
            return Ok(changed_nodes);
        }
        let committed = graph
            .commit_mount_eligibility_admissions(transitions)
            .map_err(WorthUiPreparedRebindGraphDenial::MountEligibility)?;
        self.advance_graph_snapshot(committed);
        Ok(changed_nodes)
    }

    pub(crate) fn advance_graph_snapshot(
        &mut self,
        committed: crate::graph::UiGraphMutationCommitResult,
    ) {
        self.graph_snapshot = committed.into_committed_snapshot();
        self.generation_identity = self.derive_generation_identity(&self.graph_snapshot);
        self.lowering_authority =
            self.seal_lowering_authority(&self.graph_snapshot, self.generation_identity.clone());
        self.rebuild_derived_indexes();
    }

    pub(crate) fn prepare_graph_successor(
        &self,
        committed: crate::graph::UiGraphMutationCommitResult,
    ) -> Result<
        WorthUiPreparedApplicationGraphSuccessor,
        WorthUiPreparedApplicationGraphSuccessorDenial,
    > {
        let predecessor_authority = self.graph_snapshot.authority_identity();
        if committed.predecessor_authority() != Some(predecessor_authority) {
            return Err(WorthUiPreparedApplicationGraphSuccessorDenial::StaleGraphPredecessor);
        }
        let predecessor_generation = self.generation_identity.clone();
        let graph_snapshot = committed.into_committed_snapshot();
        let generation_identity = self.derive_generation_identity(&graph_snapshot);
        let lowering_authority =
            self.seal_lowering_authority(&graph_snapshot, generation_identity.clone());
        Ok(WorthUiPreparedApplicationGraphSuccessor {
            predecessor_authority,
            graph_snapshot,
            generation_identity: generation_identity.clone(),
            generation_succession: WorthUiPreparedApplicationGenerationSuccession::new(
                predecessor_generation,
                generation_identity.clone(),
            ),
            lowering_authority,
        })
    }

    pub(crate) fn commit_graph_successor(
        &mut self,
        successor: WorthUiPreparedApplicationGraphSuccessor,
    ) -> Result<
        WorthUiPreparedApplicationLoweringAuthority,
        WorthUiPreparedApplicationGraphSuccessorDenial,
    > {
        if self.graph_snapshot.authority_identity() != successor.predecessor_authority {
            return Err(WorthUiPreparedApplicationGraphSuccessorDenial::StaleGraphPredecessor);
        }
        if self.generation_identity != *successor.generation_succession.predecessor() {
            return Err(WorthUiPreparedApplicationGraphSuccessorDenial::StaleGraphPredecessor);
        }
        self.graph_snapshot = successor.graph_snapshot;
        self.generation_identity = successor.generation_identity;
        self.lowering_authority = successor.lowering_authority;
        self.rebuild_derived_indexes();
        Ok(self.lowering_authority.clone())
    }

    fn derive_generation_identity(
        &self,
        graph_snapshot: &crate::graph::UiGraphSnapshot,
    ) -> WorthUiPreparedApplicationGenerationIdentity {
        WorthUiPreparedApplicationGenerationIdentity::derive(
            WorthUiPreparedGenerationIdentityInput {
                capability_snapshot: self.capability_snapshot.digest(),
                canonical_artifact: self.canonical_artifact.identity(),
                lineage: self.generation_lineage.clone(),
                declaration_source: self.declaration_source_identity.clone(),
                semantic_package: self.semantic_handoff.identity().clone(),
                graph_authority_digest: graph_snapshot.authority_digest(),
                query_binding_plan: &self.query_binding_plan,
                intent_application_fact_digest: self.intent_application_facts.digest_basis(),
                intent_execution_binding_digest: self.intent_execution_bindings.digest_basis(),
                service_policy_digest: self.service_policy_plan.digest_basis(),
                visual_inspection_policy: self.visual_inspection_policy,
                change_profile: self.change_profile,
            },
        )
    }

    fn seal_lowering_authority(
        &self,
        graph_snapshot: &crate::graph::UiGraphSnapshot,
        generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    ) -> WorthUiPreparedApplicationLoweringAuthority {
        WorthUiPreparedApplicationLoweringAuthority::seal(WorthUiPreparedApplicationLoweringInput {
            generation_identity,
            source_candidate_basis: self.source_backed_candidate_basis(),
            source_artifact_authority: self.canonical_artifact.runtime_artifact_authority().0,
            graph_authority_identity: graph_snapshot.authority_identity(),
            capability_snapshot: Rc::clone(&self.capability_snapshot),
            query_binding_plan: self.query_binding_plan.clone(),
        })
    }
}
