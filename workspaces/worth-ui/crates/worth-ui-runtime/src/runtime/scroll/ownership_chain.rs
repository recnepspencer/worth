#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollOwnershipResolutionDenial {
    OwnershipNotIndexed,
    UnknownGraphNode,
    ForeignPlan,
    MissingMosaicScrollOwnership(crate::graph::UiGraphNodeIdentity),
    AmbiguousMosaicScrollOwnership(crate::graph::UiGraphNodeIdentity),
    ChainDepthExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedScrollOwnershipChain {
    owners: Vec<super::UiScrollOwnerIdentity>,
    graph_nodes_visited: u16,
    plan_nodes_visited: u32,
}

impl UiResolvedScrollOwnershipChain {
    pub(crate) fn owners(&self) -> &[super::UiScrollOwnerIdentity] {
        &self.owners
    }

    pub(in crate::runtime) const fn graph_nodes_visited(&self) -> u16 {
        self.graph_nodes_visited
    }

    pub(in crate::runtime) const fn plan_nodes_visited(&self) -> u32 {
        self.plan_nodes_visited
    }
}

pub(super) fn resolve(
    graph: crate::graph::UiGraphAuthority<'_>,
    plan: crate::mounting::UiMountedPlanProjectionSource<'_>,
    graph_node: crate::graph::UiGraphNodeIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    repeated_instance_digest: u64,
) -> Result<UiResolvedScrollOwnershipChain, UiScrollOwnershipResolutionDenial> {
    let mut owners = Vec::new();
    let mut graph_nodes_visited = 0_u16;
    let mut plan_nodes_visited = 0_u32;
    let direct_children = graph.lookup().child_nodes(graph_node);
    let mut direct_owners = None;
    for child in direct_children.value().iter().copied() {
        graph_nodes_visited = graph_nodes_visited
            .checked_add(1)
            .ok_or(UiScrollOwnershipResolutionDenial::ChainDepthExceeded)?;
        let owners = ownership_for(
            graph,
            plan,
            child,
            surface,
            repeated_instance_digest,
            &mut plan_nodes_visited,
        )?;
        if !owners.is_empty() {
            if direct_owners.is_some() {
                return Err(
                    UiScrollOwnershipResolutionDenial::AmbiguousMosaicScrollOwnership(graph_node),
                );
            }
            direct_owners = Some(owners);
        }
    }
    if let Some(direct_owners) = direct_owners {
        for owner in direct_owners {
            push_owner(&mut owners, owner)?;
        }
    }

    let mut cursor = Some(graph_node);
    while let Some(node) = cursor {
        graph_nodes_visited = graph_nodes_visited
            .checked_add(1)
            .ok_or(UiScrollOwnershipResolutionDenial::ChainDepthExceeded)?;
        for owner in ownership_for(
            graph,
            plan,
            node,
            surface,
            repeated_instance_digest,
            &mut plan_nodes_visited,
        )? {
            push_owner(&mut owners, owner)?;
        }
        cursor = graph
            .lookup()
            .topology_node(node)
            .ok_or(UiScrollOwnershipResolutionDenial::UnknownGraphNode)?
            .value()
            .parent_node_identity();
    }
    Ok(UiResolvedScrollOwnershipChain {
        owners,
        graph_nodes_visited,
        plan_nodes_visited,
    })
}

fn push_owner(
    owners: &mut Vec<super::UiScrollOwnerIdentity>,
    owner: super::UiScrollOwnerIdentity,
) -> Result<(), UiScrollOwnershipResolutionDenial> {
    if owners.contains(&owner) {
        return Ok(());
    }
    if owners.len() == super::request::UI_SCROLL_CHAIN_DEPTH_LIMIT {
        return Err(UiScrollOwnershipResolutionDenial::ChainDepthExceeded);
    }
    owners.push(owner);
    Ok(())
}

fn ownership_for(
    graph: crate::graph::UiGraphAuthority<'_>,
    plan: crate::mounting::UiMountedPlanProjectionSource<'_>,
    node: crate::graph::UiGraphNodeIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    repeated_instance_digest: u64,
    plan_nodes_visited: &mut u32,
) -> Result<Vec<super::UiScrollOwnerIdentity>, UiScrollOwnershipResolutionDenial> {
    let graph_record = graph
        .lookup()
        .graph_node(node)
        .ok_or(UiScrollOwnershipResolutionDenial::UnknownGraphNode)?
        .value();
    let plan_index = plan
        .plan_index(graph_record.authored_provenance_digest())
        .map_err(|_| UiScrollOwnershipResolutionDenial::ForeignPlan)?;
    let Some(root) = plan_index.and_then(|index| plan.ordinary_meaning(index)) else {
        return Ok(Vec::new());
    };
    plan_owner_branch(
        plan,
        node,
        surface,
        repeated_instance_digest,
        None,
        root.as_ref(),
        0,
        plan_nodes_visited,
    )
}

fn plan_owner_branch(
    plan: crate::mounting::UiMountedPlanProjectionSource<'_>,
    graph_node: crate::graph::UiGraphNodeIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    repeated_instance_digest: u64,
    plan_index: Option<u32>,
    meaning: &crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning,
    depth: usize,
    plan_nodes_visited: &mut u32,
) -> Result<Vec<super::UiScrollOwnerIdentity>, UiScrollOwnershipResolutionDenial> {
    *plan_nodes_visited = plan_nodes_visited
        .checked_add(1)
        .ok_or(UiScrollOwnershipResolutionDenial::ChainDepthExceeded)?;
    if depth == super::request::UI_SCROLL_CHAIN_DEPTH_LIMIT {
        return Err(UiScrollOwnershipResolutionDenial::ChainDepthExceeded);
    }
    let mut child_branch = Vec::new();
    if let Some(range_identity) = meaning.child_range_identity() {
        let (_, range) = plan
            .ordinary_meaning_for_identity(range_identity)
            .ok_or(UiScrollOwnershipResolutionDenial::ForeignPlan)?;
        let range = range
            .child_range()
            .ok_or(UiScrollOwnershipResolutionDenial::ForeignPlan)?;
        for child_identity in range.child_identities() {
            let Some((child_index, child)) = plan.ordinary_meaning_for_identity(child_identity)
            else {
                return Err(UiScrollOwnershipResolutionDenial::ForeignPlan);
            };
            let candidate = plan_owner_branch(
                plan,
                graph_node,
                surface,
                repeated_instance_digest,
                Some(child_index),
                child.as_ref(),
                depth + 1,
                plan_nodes_visited,
            )?;
            if candidate.is_empty() {
                continue;
            }
            admit_child_owner_branch(graph_node, &mut child_branch, candidate)?;
        }
    }
    let Some(owner) = owner_for_layout(
        graph_node,
        surface,
        repeated_instance_digest,
        plan_index,
        meaning,
    )?
    else {
        return Ok(child_branch);
    };
    child_branch.push(owner);
    Ok(child_branch)
}

fn admit_child_owner_branch(
    graph_node: crate::graph::UiGraphNodeIdentity,
    admitted: &mut Vec<super::UiScrollOwnerIdentity>,
    candidate: Vec<super::UiScrollOwnerIdentity>,
) -> Result<(), UiScrollOwnershipResolutionDenial> {
    if !admitted.is_empty() {
        return Err(UiScrollOwnershipResolutionDenial::AmbiguousMosaicScrollOwnership(graph_node));
    }
    *admitted = candidate;
    Ok(())
}

fn owner_for_layout(
    graph_node: crate::graph::UiGraphNodeIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    repeated_instance_digest: u64,
    plan_index: Option<u32>,
    meaning: &crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning,
) -> Result<Option<super::UiScrollOwnerIdentity>, UiScrollOwnershipResolutionDenial> {
    let crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Layout(layout) =
        meaning
    else {
        return Ok(None);
    };
    let Some(descriptor) = layout.region_descriptor() else {
        return Ok(None);
    };
    use crate::capability::MosaicScrollOwnership as Ownership;
    match descriptor
        .scroll_ownership()
        .cloned()
        .unwrap_or_else(Ownership::missing_for_diagnostics)
    {
        Ownership::NoScrolling => Ok(None),
        Ownership::RegionOwned => Ok(Some(super::UiScrollOwnerIdentity::declared_region(
            surface,
            graph_node,
            repeated_instance_digest,
            plan_index.ok_or(UiScrollOwnershipResolutionDenial::ForeignPlan)?,
        ))),
        Ownership::SurfaceOwned => Ok(Some(super::UiScrollOwnerIdentity::surface(surface))),
        Ownership::ViewportOwned => Ok(Some(super::UiScrollOwnerIdentity::viewport(surface))),
        Ownership::MissingForDiagnostics => {
            Err(UiScrollOwnershipResolutionDenial::MissingMosaicScrollOwnership(graph_node))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity")
    }

    #[test]
    fn ambiguous_child_owner_branches_are_denied_before_chain_construction() {
        let graph_node = crate::graph::UiGraphNodeIdentity::new(315_501);
        let surface = surface();
        let mut admitted = vec![super::super::UiScrollOwnerIdentity::declared_region(
            surface, graph_node, 1, 3,
        )];
        let candidate = vec![super::super::UiScrollOwnerIdentity::declared_region(
            surface, graph_node, 1, 7,
        )];
        assert_eq!(
            admit_child_owner_branch(graph_node, &mut admitted, candidate),
            Err(UiScrollOwnershipResolutionDenial::AmbiguousMosaicScrollOwnership(graph_node))
        );
    }

    #[test]
    fn missing_mosaic_ownership_is_a_typed_resolution_denial() {
        let graph_node = crate::graph::UiGraphNodeIdentity::new(315_502);
        let descriptor = crate::capability::MosaicRegionKindDescriptor::new(
            crate::capability::MosaicRegionKindId::new("phase315.scroll.missing")
                .expect("valid region id"),
            crate::capability::MosaicRegionRole::primary(),
        )
        .with_scroll_ownership(crate::capability::MosaicScrollOwnership::missing_for_diagnostics());
        let meaning =
            crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Layout(
                crate::runtime::planning::execution_plan_input::WorthUiLayoutPlanMeaning::region(
                    descriptor, None, None,
                ),
            );
        assert_eq!(
            owner_for_layout(graph_node, surface(), 1, Some(0), &meaning),
            Err(UiScrollOwnershipResolutionDenial::MissingMosaicScrollOwnership(graph_node))
        );
    }
}
