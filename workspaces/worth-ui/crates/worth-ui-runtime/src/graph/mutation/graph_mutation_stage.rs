use std::collections::BTreeMap;

use crate::graph::{
    materialize_graph_mount_eligibilities, materialize_graph_participation_posture,
    materialize_graph_topology, UiGraphAxisParticipation, UiGraphCoreIndexes, UiGraphGeneration,
    UiGraphInstantiationPlan, UiGraphMountEligibilityStore, UiGraphMountEligibilityTransition,
    UiGraphNode, UiGraphParticipationAxis, UiGraphParticipationPosture, UiGraphParticipationStatus,
    UiGraphSnapshot, UiGraphTopology, UiGraphWorldProfile,
};
#[cfg(any(test, feature = "certification-support"))]
use crate::graph::{UiGraphNodeIdentity, UiGraphParticipationMutation};

pub(crate) struct UiGraphMutationStage {
    generation: UiGraphGeneration,
    world_profile: UiGraphWorldProfile,
    nodes: Vec<UiGraphNode>,
    topology: UiGraphTopology,
    mount_eligibilities: UiGraphMountEligibilityStore,
    core_indexes: UiGraphCoreIndexes,
}

impl UiGraphMutationStage {
    pub(crate) fn from_initial_plan(
        plan: &UiGraphInstantiationPlan,
        world_profile: UiGraphWorldProfile,
    ) -> Self {
        let mount_eligibility_reservations =
            plan.mount_eligibility_reservations(world_profile.clone());
        let mut nodes = Vec::with_capacity(plan.node_entries().len());
        let mut node_identities = Vec::with_capacity(plan.node_entries().len());

        for (entry, reservation) in plan
            .node_entries()
            .iter()
            .zip(mount_eligibility_reservations.iter().copied())
        {
            let graph_node_identity = reservation.graph_node_identity();
            let node = UiGraphNode::new(crate::graph::UiGraphNodeInput {
                graph_node_identity,
                declaration_identity: entry.declaration_identity().clone(),
                aspect_contract: entry.aspect_contract().clone(),
                component_reference: entry.component_reference().cloned(),
                appearance_role_attachment: entry.appearance_role_attachment().cloned(),
                structural_digest: entry.topology_seed().structural_digest(),
                structural_role: entry.topology_seed().role(),
                operator_kind: entry.topology_seed().operator_kind(),
                repetition_posture: entry.topology_seed().repetition_posture(),
                measurement_constraint_modifier: entry.measurement_constraint_modifier(),
                measurement_basis_source: entry.measurement_basis_source(),
                authored_provenance_digest: entry.authored_provenance_digest(),
                repeated_instance_basis: entry.repeated_instance_basis().clone(),
                attachment_posture: entry.attachment_posture(),
                participation_posture: materialize_graph_participation_posture(entry),
            });

            node_identities.push(graph_node_identity);
            nodes.push(node);
        }

        let topology = materialize_graph_topology(plan, &node_identities);
        let mount_eligibilities =
            materialize_graph_mount_eligibilities(&mount_eligibility_reservations);
        let core_indexes = UiGraphCoreIndexes::build(&nodes, &topology, &mount_eligibilities);

        Self {
            generation: UiGraphGeneration::initial(),
            world_profile,
            nodes,
            topology,
            mount_eligibilities,
            core_indexes,
        }
    }

    pub(crate) fn from_successor_plan(
        prior_snapshot: &UiGraphSnapshot,
        plan: &UiGraphInstantiationPlan,
    ) -> Self {
        let mut stage = Self::from_initial_plan(plan, prior_snapshot.world_profile().clone());
        stage.generation = UiGraphGeneration::successor_of(prior_snapshot.generation());
        stage
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn layout_admitted_successor(
        prior_snapshot: &UiGraphSnapshot,
        admitted_nodes: &[UiGraphNodeIdentity],
    ) -> Self {
        let nodes = prior_snapshot
            .nodes()
            .iter()
            .map(|node| {
                let posture = if admitted_nodes.contains(&node.graph_node_identity()) {
                    let page = prior_snapshot
                        .lookup()
                        .topology_node(node.graph_node_identity())
                        .and_then(|row| row.value().page_membership())
                        .map(|membership| membership.page_node_identity())
                        .unwrap_or_else(|| node.graph_node_identity());
                    UiGraphParticipationMutation::axis_transition(
                        node.graph_node_identity(),
                        page,
                        node.participation_posture(),
                        UiGraphParticipationAxis::Layout,
                        UiGraphAxisParticipation::runtime_mutation(
                            UiGraphParticipationStatus::Admitted,
                        ),
                    )
                    .updated_posture()
                } else {
                    node.participation_posture()
                };
                clone_node_with_posture(node, posture)
            })
            .collect::<Vec<_>>();

        Self::successor_with_nodes(prior_snapshot, nodes)
    }

    pub(crate) fn mount_eligibility_admitted_successor(
        prior_snapshot: &UiGraphSnapshot,
        transitions: &[UiGraphMountEligibilityTransition],
    ) -> Self {
        let transitions = transitions
            .iter()
            .map(|transition| {
                (
                    transition.eligibility_record().graph_node_identity(),
                    *transition,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let nodes = prior_snapshot
            .nodes()
            .iter()
            .map(|node| {
                let Some(transition) = transitions.get(&node.graph_node_identity()) else {
                    return node.clone();
                };
                let posture = node
                    .participation_posture()
                    .with_axis(
                        UiGraphParticipationAxis::Mounted,
                        transition.next_eligibility(),
                    )
                    .with_axis(
                        UiGraphParticipationAxis::Layout,
                        UiGraphAxisParticipation::runtime_mutation(
                            UiGraphParticipationStatus::Admitted,
                        ),
                    );
                clone_node_with_posture(node, posture)
            })
            .collect::<Vec<_>>();

        Self::successor_with_nodes(prior_snapshot, nodes)
    }

    fn successor_with_nodes(prior_snapshot: &UiGraphSnapshot, nodes: Vec<UiGraphNode>) -> Self {
        let core_indexes = UiGraphCoreIndexes::rebuild(
            &nodes,
            prior_snapshot.topology(),
            prior_snapshot.mount_eligibilities(),
        );

        Self {
            generation: UiGraphGeneration::successor_of(prior_snapshot.generation()),
            world_profile: prior_snapshot.world_profile().clone(),
            nodes,
            topology: prior_snapshot.topology().clone(),
            mount_eligibilities: prior_snapshot.mount_eligibilities().clone(),
            core_indexes,
        }
    }

    pub(crate) fn commit(self) -> UiGraphSnapshot {
        UiGraphSnapshot::new(
            self.generation,
            self.world_profile,
            self.nodes,
            self.topology,
            self.mount_eligibilities,
            self.core_indexes,
        )
    }
}

fn clone_node_with_posture(
    node: &UiGraphNode,
    participation_posture: UiGraphParticipationPosture,
) -> UiGraphNode {
    UiGraphNode::new(crate::graph::UiGraphNodeInput {
        graph_node_identity: node.graph_node_identity(),
        declaration_identity: node.declaration_identity().clone(),
        aspect_contract: node.aspect_contract().clone(),
        component_reference: node.component_reference().cloned(),
        appearance_role_attachment: node.appearance_role_attachment().cloned(),
        structural_digest: node.structural_digest(),
        structural_role: node.structural_role(),
        operator_kind: node.operator_kind(),
        repetition_posture: node.repetition_posture(),
        measurement_constraint_modifier: node.measurement_constraint_modifier(),
        measurement_basis_source: node.measurement_basis_source(),
        authored_provenance_digest: node.authored_provenance_digest(),
        repeated_instance_basis: node.repeated_instance_basis().clone(),
        attachment_posture: node.attachment_posture(),
        participation_posture,
    })
}

#[cfg(test)]
pub(crate) fn adversarial_snapshot_with_swapped_node_index_for_test(
    snapshot: &UiGraphSnapshot,
    attached: UiGraphNodeIdentity,
    peer: UiGraphNodeIdentity,
) -> UiGraphSnapshot {
    let mut indexed_nodes = snapshot.nodes().to_vec();
    let attached_position = indexed_nodes
        .iter()
        .position(|node| node.graph_node_identity() == attached)
        .expect("attached node should be present in the index source");
    let peer_position = indexed_nodes
        .iter()
        .position(|node| node.graph_node_identity() == peer)
        .expect("peer node should be present in the index source");
    indexed_nodes.swap(attached_position, peer_position);
    let indexes = UiGraphCoreIndexes::build(
        &indexed_nodes,
        snapshot.topology(),
        snapshot.mount_eligibilities(),
    );
    UiGraphSnapshot::new(
        snapshot.generation(),
        snapshot.world_profile().clone(),
        snapshot.nodes().to_vec(),
        snapshot.topology().clone(),
        snapshot.mount_eligibilities().clone(),
        indexes,
    )
}
