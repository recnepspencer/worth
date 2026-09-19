use std::collections::BTreeMap;

use crate::runtime::{
    WorthUiNodeLifecycleTransition, WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily,
    WorthUiPlanNodeTopologyInput,
};
use crate::source::WorthUiArtifactNode;

use super::mosaic_row_lowering::WorthUiMosaicRowLowerer;
use super::{
    WorthUiChildRangePlanMeaning, WorthUiCommandPlanMeaning, WorthUiComponentPlanMeaning,
    WorthUiLayoutPlanMeaning, WorthUiPlanOrdinaryMeaning, WorthUiTokenPlanMeaning,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiOrdinaryLoweringDenial {
    MissingStateSuccession,
    InvalidStateSuccession,
    MissingSpatialContract,
    MissingRealtimeContract,
}

pub(crate) fn lower_launch_node(
    node: &WorthUiArtifactNode,
    topology: WorthUiPlanNodeTopologyInput,
) -> Result<Vec<WorthUiPlanNodeInput>, WorthUiOrdinaryLoweringDenial> {
    lower_node(node, topology, WorthUiNodeLifecycleTransition::Create, None)
}

pub(crate) fn lower_replacement_node(
    node: &WorthUiArtifactNode,
    topology: WorthUiPlanNodeTopologyInput,
    transition: WorthUiNodeLifecycleTransition,
    reconciliation: &crate::runtime::WorthUiDurableStateReconciliationPlan,
) -> Result<Vec<WorthUiPlanNodeInput>, WorthUiOrdinaryLoweringDenial> {
    lower_node(node, topology, transition, Some(reconciliation))
}

fn lower_node(
    node: &WorthUiArtifactNode,
    topology: WorthUiPlanNodeTopologyInput,
    transition: WorthUiNodeLifecycleTransition,
    reconciliation: Option<&crate::runtime::WorthUiDurableStateReconciliationPlan>,
) -> Result<Vec<WorthUiPlanNodeInput>, WorthUiOrdinaryLoweringDenial> {
    let root_identity = node.identity_seed().basis().to_owned();
    let provenance = Some(node.authored_provenance_digest());
    let mut rows = match node {
        WorthUiArtifactNode::Import(_) | WorthUiArtifactNode::Binding(_) => Vec::new(),
        WorthUiArtifactNode::Component(component) => {
            if component.descriptor().execution_lane()
                == crate::capability::ComponentExecutionLane::RealtimeOverlay
            {
                let contract = component
                    .descriptor()
                    .realtime_overlay_contract()
                    .ok_or(WorthUiOrdinaryLoweringDenial::MissingRealtimeContract)?;
                return Ok(vec![WorthUiPlanNodeInput::from_realtime_component(
                    root_identity,
                    provenance,
                    transition,
                    topology,
                    super::WorthUiRealtimePlanMeaning::new(
                        component.descriptor().clone(),
                        contract,
                    ),
                )]);
            }
            if component.descriptor().execution_lane()
                == crate::capability::ComponentExecutionLane::CanvasSpatial
            {
                let contract = component
                    .descriptor()
                    .canvas_spatial_contract()
                    .ok_or(WorthUiOrdinaryLoweringDenial::MissingSpatialContract)?;
                return Ok(vec![WorthUiPlanNodeInput::from_spatial_component(
                    root_identity,
                    provenance,
                    transition,
                    topology,
                    super::WorthUiSpatialPlanMeaning::new(component.descriptor().clone(), contract),
                )]);
            }
            let mosaic = WorthUiMosaicRowLowerer::new(
                &root_identity,
                provenance,
                transition,
                reconciliation,
            )
            .lower(component.structure())?;
            let mut members = mosaic.rows;
            let child_range = add_child_range_for_root(
                &root_identity,
                mosaic.root_children,
                &mut members,
                provenance,
                transition,
            );
            let root = WorthUiPlanNodeInput::from_ordinary_row(
                root_identity.clone(),
                provenance,
                WorthUiPlanNodeInputFamily::ComponentInvocation,
                transition,
                topology,
                None,
                WorthUiPlanOrdinaryMeaning::Component(WorthUiComponentPlanMeaning::new(
                    component.descriptor().clone(),
                    child_range,
                )),
            );
            with_root_manifest(root, members)
        }
        WorthUiArtifactNode::Surface(surface) => {
            let mut members =
                lower_surface_commands(&root_identity, surface, provenance, transition);
            let mut root_children = members
                .iter()
                .map(|row| row.identity_basis().to_owned())
                .collect::<Vec<_>>();
            let mosaic = WorthUiMosaicRowLowerer::new(
                &root_identity,
                provenance,
                transition,
                reconciliation,
            )
            .lower(surface.structure())?;
            root_children.extend(mosaic.root_children);
            members.extend(mosaic.rows);
            let child_range = add_child_range_for_root(
                &root_identity,
                root_children,
                &mut members,
                provenance,
                transition,
            );
            let root = WorthUiPlanNodeInput::from_ordinary_row(
                root_identity.clone(),
                provenance,
                WorthUiPlanNodeInputFamily::LayoutRegion,
                transition,
                topology,
                None,
                WorthUiPlanOrdinaryMeaning::Layout(WorthUiLayoutPlanMeaning::surface(
                    surface.descriptor().clone(),
                    None,
                    child_range,
                )),
            );
            with_root_manifest(root, members)
        }
        WorthUiArtifactNode::Token(token) => vec![WorthUiPlanNodeInput::from_ordinary_row(
            root_identity,
            provenance,
            WorthUiPlanNodeInputFamily::TokenStyle,
            transition,
            topology,
            None,
            WorthUiPlanOrdinaryMeaning::Token(WorthUiTokenPlanMeaning::new(
                token.entry().clone(),
                token.semantics().clone(),
            )),
        )],
    };
    rows.sort_by(|left, right| left.identity_basis().cmp(right.identity_basis()));
    Ok(rows)
}

fn lower_surface_commands(
    root_identity: &str,
    surface: &crate::source::WorthUiArtifactSurfaceNode,
    provenance: Option<u64>,
    transition: WorthUiNodeLifecycleTransition,
) -> Vec<WorthUiPlanNodeInput> {
    let mut occurrences = BTreeMap::<&str, usize>::new();
    surface
        .semantics()
        .command_slots()
        .iter()
        .map(|reference| {
            let id = reference.command().id().as_str();
            let occurrence = next_occurrence(&mut occurrences, id);
            let identity = child_identity(root_identity, "command", id, occurrence);
            WorthUiPlanNodeInput::from_ordinary_row(
                identity,
                provenance,
                WorthUiPlanNodeInputFamily::Command,
                transition,
                WorthUiPlanNodeTopologyInput::empty(),
                Some(root_identity.to_owned()),
                WorthUiPlanOrdinaryMeaning::Command(Box::new(WorthUiCommandPlanMeaning::new(
                    root_identity.to_owned(),
                    reference.clone(),
                ))),
            )
        })
        .collect()
}

fn lower_child_range(
    root_identity: &str,
    owner_identity: &str,
    children: Vec<String>,
    provenance: Option<u64>,
    transition: WorthUiNodeLifecycleTransition,
    rows: &mut Vec<WorthUiPlanNodeInput>,
) -> Option<String> {
    if children.is_empty() {
        return None;
    }
    let identity = format!("{owner_identity}::child-range");
    rows.push(WorthUiPlanNodeInput::from_ordinary_row(
        identity.clone(),
        provenance,
        WorthUiPlanNodeInputFamily::ChildRange,
        transition,
        WorthUiPlanNodeTopologyInput::empty(),
        Some(root_identity.to_owned()),
        WorthUiPlanOrdinaryMeaning::ChildRange(WorthUiChildRangePlanMeaning::new(
            owner_identity.to_owned(),
            children,
        )),
    ));
    Some(identity)
}

fn add_child_range_for_root(
    root_identity: &str,
    children: Vec<String>,
    members: &mut Vec<WorthUiPlanNodeInput>,
    provenance: Option<u64>,
    transition: WorthUiNodeLifecycleTransition,
) -> Option<String> {
    lower_child_range(
        root_identity,
        root_identity,
        children,
        provenance,
        transition,
        members,
    )
}

fn with_root_manifest(
    mut root: WorthUiPlanNodeInput,
    mut members: Vec<WorthUiPlanNodeInput>,
) -> Vec<WorthUiPlanNodeInput> {
    members.sort_by(|left, right| left.identity_basis().cmp(right.identity_basis()));
    root.set_owned_region_identity_bases(
        members
            .iter()
            .map(|member| member.identity_basis().to_owned())
            .collect(),
    );
    let mut rows = Vec::with_capacity(members.len() + 1);
    rows.push(root);
    rows.extend(members);
    rows
}

fn child_identity(parent: &str, kind: &str, id: &str, occurrence: usize) -> String {
    format!("{parent}::{kind}::{id}#{occurrence}")
}

fn next_occurrence<'a>(occurrences: &mut BTreeMap<&'a str, usize>, id: &'a str) -> usize {
    let occurrence = occurrences.entry(id).or_default();
    let current = *occurrence;
    *occurrence += 1;
    current
}
