use std::collections::BTreeMap;

use crate::declaration::UiDeclarationSlotParticipationIntent;
use crate::graph::{
    UiGraphContainmentClaim, UiGraphInstantiationPlan, UiGraphMembershipFacts,
    UiGraphMosaicMembership, UiGraphNodeIdentity, UiGraphNodeTopology, UiGraphPageMembership,
    UiGraphParentResolutionClaim, UiGraphRegionMembership, UiGraphSlotTopology, UiGraphTopology,
};

pub(crate) fn materialize_graph_topology(
    plan: &UiGraphInstantiationPlan,
    node_identities: &[UiGraphNodeIdentity],
) -> UiGraphTopology {
    if plan.node_entries().is_empty() {
        return UiGraphTopology::new(BTreeMap::new());
    }

    let root_page_identity = single_root_page_identity(plan, node_identities)
        .expect("topology mutation should only run after root-page topology admits coherently");
    let node_topologies = plan
        .node_entries()
        .iter()
        .zip(node_identities.iter().copied())
        .map(|(entry, node_identity)| {
            (
                node_identity,
                UiGraphNodeTopology::new(
                    node_identity,
                    entry.topology_seed().containment_claim().clone(),
                    entry.topology_seed().parent_resolution_claim().clone(),
                    parent_node_identity(
                        entry.topology_seed().parent_resolution_claim(),
                        root_page_identity,
                    ),
                    slot_topology(entry.topology_seed().slot_participation_intent()),
                    entry.topology_seed().ordering_guarantee(),
                    membership_facts(
                        node_identity,
                        entry.topology_seed().containment_claim(),
                        entry.topology_seed().parent_resolution_claim(),
                        root_page_identity,
                    ),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    UiGraphTopology::new(node_topologies)
}

fn single_root_page_identity(
    plan: &UiGraphInstantiationPlan,
    node_identities: &[UiGraphNodeIdentity],
) -> Option<UiGraphNodeIdentity> {
    let root_pages = plan
        .node_entries()
        .iter()
        .zip(node_identities.iter().copied())
        .filter_map(|(entry, node_identity)| {
            entry
                .topology_seed()
                .containment_claim()
                .is_root_page()
                .then_some(node_identity)
        })
        .collect::<Vec<_>>();

    match root_pages.as_slice() {
        [root_page_identity] => Some(*root_page_identity),
        _ => None,
    }
}

fn parent_node_identity(
    parent_resolution_claim: &UiGraphParentResolutionClaim,
    root_page_identity: UiGraphNodeIdentity,
) -> Option<UiGraphNodeIdentity> {
    parent_resolution_claim.resolve_parent_node_identity(root_page_identity)
}

fn slot_topology(
    slot_participation_intent: &UiDeclarationSlotParticipationIntent,
) -> Option<UiGraphSlotTopology> {
    match slot_participation_intent {
        UiDeclarationSlotParticipationIntent::DeclaredSlotParticipant { slot_name } => {
            Some(UiGraphSlotTopology::new(slot_name.clone()))
        }
        UiDeclarationSlotParticipationIntent::None => None,
    }
}

fn membership_facts(
    node_identity: UiGraphNodeIdentity,
    containment_claim: &UiGraphContainmentClaim,
    parent_resolution_claim: &UiGraphParentResolutionClaim,
    root_page_identity: UiGraphNodeIdentity,
) -> UiGraphMembershipFacts {
    let page_membership = Some(UiGraphPageMembership::new(
        parent_resolution_claim.resolve_page_membership(node_identity, root_page_identity),
    ));

    let region_membership = match containment_claim {
        UiGraphContainmentClaim::Region { region_name } => {
            Some(UiGraphRegionMembership::new(region_name.clone()))
        }
        UiGraphContainmentClaim::RootPage
        | UiGraphContainmentClaim::PageSet { .. }
        | UiGraphContainmentClaim::Mosaic { .. }
        | UiGraphContainmentClaim::LocalComposition { .. }
        | UiGraphContainmentClaim::Control { .. }
        | UiGraphContainmentClaim::DiagnosticSurface { .. } => None,
    };

    let mosaic_membership = match containment_claim {
        UiGraphContainmentClaim::Mosaic { mosaic_name, .. } => {
            Some(UiGraphMosaicMembership::new(mosaic_name.clone()))
        }
        UiGraphContainmentClaim::RootPage
        | UiGraphContainmentClaim::PageSet { .. }
        | UiGraphContainmentClaim::Region { .. }
        | UiGraphContainmentClaim::LocalComposition { .. }
        | UiGraphContainmentClaim::Control { .. }
        | UiGraphContainmentClaim::DiagnosticSurface { .. } => None,
    };

    UiGraphMembershipFacts::new(page_membership, region_membership, mosaic_membership)
}
