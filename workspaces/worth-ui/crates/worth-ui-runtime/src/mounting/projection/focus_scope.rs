use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiMountedFocusScope {
    ActiveSurface,
    MosaicRegion {
        owner: crate::graph::UiGraphNodeIdentity,
        kind: crate::capability::MosaicFocusScopeKind,
    },
}

impl UiMountedFocusScope {
    pub(crate) const fn mosaic_owner(self) -> Option<crate::graph::UiGraphNodeIdentity> {
        match self {
            Self::ActiveSurface => None,
            Self::MosaicRegion { owner, .. } => Some(owner),
        }
    }

    pub(crate) const fn kind(self) -> crate::capability::MosaicFocusScopeKind {
        match self {
            Self::ActiveSurface => crate::capability::MosaicFocusScopeKind::ActiveSurfaceScope,
            Self::MosaicRegion { kind, .. } => kind,
        }
    }
}

pub(super) fn resolve(
    graph: crate::graph::UiGraphAuthority<'_>,
    plan: super::super::UiMountedPlanProjectionSource<'_>,
    graph_node: crate::graph::UiGraphNodeIdentity,
) -> Result<Option<UiMountedFocusScope>, super::UiMountedProjectionDenial> {
    let mut owned_region = None;
    for child in graph
        .lookup()
        .child_nodes(graph_node)
        .value()
        .iter()
        .copied()
    {
        if let Some(kind) = region_kind(graph, plan, child)? {
            if owned_region.is_some() {
                return Err(super::UiMountedProjectionDenial::AmbiguousMosaicFocusScope(
                    graph_node,
                ));
            }
            owned_region = Some((child, kind));
        }
    }
    if let Some((owner, kind)) = owned_region {
        return classify_region(owner, kind);
    }

    let mut cursor = Some(graph_node);
    while let Some(node) = cursor {
        if let Some(kind) = region_kind(graph, plan, node)? {
            return classify_region(node, kind);
        }
        cursor = graph
            .lookup()
            .topology_node(node)
            .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
            .value()
            .parent_node_identity();
    }
    Ok(Some(UiMountedFocusScope::ActiveSurface))
}

pub(super) fn container_owner(
    graph: crate::graph::UiGraphAuthority<'_>,
    plan: super::super::UiMountedPlanProjectionSource<'_>,
    graph_node: crate::graph::UiGraphNodeIdentity,
) -> Result<Option<crate::graph::UiGraphNodeIdentity>, super::UiMountedProjectionDenial> {
    let mut cursor = graph
        .lookup()
        .topology_node(graph_node)
        .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
        .value()
        .parent_node_identity();
    while let Some(node) = cursor {
        let record = graph
            .lookup()
            .graph_node(node)
            .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
            .value();
        let index = plan
            .plan_index(record.authored_provenance_digest())
            .map_err(|_| super::UiMountedProjectionDenial::ForeignPlan)?;
        let focus = index
            .and_then(|index| plan.ordinary_meaning(index))
            .and_then(|meaning| match meaning.as_ref() {
                WorthUiPlanOrdinaryMeaning::Component(component) => Some(component.focus_support()),
                WorthUiPlanOrdinaryMeaning::Layout(_)
                | WorthUiPlanOrdinaryMeaning::ChildRange(_)
                | WorthUiPlanOrdinaryMeaning::Command(_)
                | WorthUiPlanOrdinaryMeaning::Token(_)
                | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
            });
        if focus.is_some_and(|support| support.container_policy().is_some()) {
            return Ok(Some(node));
        }
        cursor = graph
            .lookup()
            .topology_node(node)
            .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
            .value()
            .parent_node_identity();
    }
    Ok(None)
}

fn region_kind(
    graph: crate::graph::UiGraphAuthority<'_>,
    plan: super::super::UiMountedPlanProjectionSource<'_>,
    node: crate::graph::UiGraphNodeIdentity,
) -> Result<Option<Option<crate::capability::MosaicFocusScopeKind>>, super::UiMountedProjectionDenial>
{
    let graph_record = graph
        .lookup()
        .graph_node(node)
        .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
        .value();
    let plan_index = plan
        .plan_index(graph_record.authored_provenance_digest())
        .map_err(|_| super::UiMountedProjectionDenial::ForeignPlan)?;
    Ok(plan_index
        .and_then(|index| plan.ordinary_meaning(index))
        .and_then(|meaning| match meaning.as_ref() {
            WorthUiPlanOrdinaryMeaning::Layout(layout) => layout
                .region_descriptor()
                .map(|descriptor| descriptor.focus_scope().copied()),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }))
}

fn classify_region(
    owner: crate::graph::UiGraphNodeIdentity,
    kind: Option<crate::capability::MosaicFocusScopeKind>,
) -> Result<Option<UiMountedFocusScope>, super::UiMountedProjectionDenial> {
    use crate::capability::MosaicFocusScopeKind as Kind;

    match kind {
        Some(Kind::ActiveSurfaceScope) => Ok(Some(UiMountedFocusScope::ActiveSurface)),
        Some(Kind::RegionScope | Kind::ModalTrapScope | Kind::ToolbarScope | Kind::StatusScope) => {
            Ok(Some(UiMountedFocusScope::MosaicRegion {
                owner,
                kind: kind.expect("matched concrete Mosaic focus scope"),
            }))
        }
        Some(Kind::NoFocusScope) => Ok(None),
        Some(Kind::MissingForDiagnostics) | None => Err(
            super::UiMountedProjectionDenial::MissingMosaicFocusScope(owner),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_region, UiMountedFocusScope};
    use crate::capability::MosaicFocusScopeKind as Kind;

    #[test]
    fn mosaic_scope_kinds_are_consumed_without_widening_or_parallel_vocabulary() {
        let owner = crate::graph::UiGraphNodeIdentity::new(41);

        assert_eq!(
            classify_region(owner, Some(Kind::ActiveSurfaceScope)),
            Ok(Some(UiMountedFocusScope::ActiveSurface))
        );
        assert_eq!(classify_region(owner, Some(Kind::NoFocusScope)), Ok(None));
        assert_eq!(
            classify_region(owner, Some(Kind::ModalTrapScope)),
            Ok(Some(UiMountedFocusScope::MosaicRegion {
                owner,
                kind: Kind::ModalTrapScope,
            }))
        );
        assert_eq!(
            classify_region(owner, None),
            Err(super::super::UiMountedProjectionDenial::MissingMosaicFocusScope(owner))
        );
        assert_eq!(
            classify_region(owner, Some(Kind::MissingForDiagnostics)),
            Err(super::super::UiMountedProjectionDenial::MissingMosaicFocusScope(owner))
        );
    }
}
