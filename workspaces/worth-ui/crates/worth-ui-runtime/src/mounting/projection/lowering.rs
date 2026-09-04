use super::frame_storage::{UiMountedProjectionSurface, UiMountedSemanticProjection};
use super::geometry::lower_allocation;
use super::mechanical_role::mechanical_role;
use super::participation::lower_participation;
use super::prepared_projection::{UiPreparedMountedProjection, UiPreparedMountedProjectionInput};
use super::UiMountedProjectionDenial;

mod delta;
#[path = "lowering/node_draft.rs"]
mod node_draft;
mod node_lowering;

pub(crate) struct UiMountedProjectionInput<'input, 'graph> {
    pub(crate) graph: crate::graph::UiGraphAuthority<'graph>,
    pub(crate) plan_digest: u64,
    pub(crate) plan: super::super::UiMountedPlanProjectionSource<'input>,
    pub(crate) allocation_source: &'input crate::runtime::UiMountedAllocationProjectionSource,
    pub(crate) requested_surfaces: &'input [worth_ui_host_contract::UiSemanticSurfaceIdentity],
    pub(crate) preview: Option<UiMountedPreviewProjectionInput>,
    pub(crate) visual_overlay: Option<super::super::UiMountedVisualOverlayProjectionInput>,
    pub(crate) portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    pub(crate) semantic_content: &'input super::super::UiMountedSemanticContentInput,
    pub(crate) theme_values: &'input super::super::UiMountedThemeValueSource,
    pub(crate) appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    pub(crate) font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    pub(in crate::mounting) semantic_predecessor: Option<&'input UiMountedSemanticProjection>,
    pub(crate) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(crate) capability_profile_digest: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedPreviewProjectionInput {
    pub(crate) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(crate) frame_epoch: u64,
    pub(crate) extent_subpixels: u32,
    pub(crate) candidate_count: u16,
    pub(crate) all_candidates_admitted: bool,
}

struct UiMountedNodeLoweringContext<'input, 'graph> {
    graph: crate::graph::UiGraphAuthority<'graph>,
    plan: super::super::UiMountedPlanProjectionSource<'input>,
    allocation_source: &'input crate::runtime::UiMountedAllocationProjectionSource,
    plan_digest: u64,
    semantic_content: &'input super::super::UiMountedSemanticContentInput,
    theme_values: &'input super::super::UiMountedThemeValueSource,
    appearance_invalidation: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    predecessor: Option<&'input UiMountedSemanticProjection>,
    mechanics_predecessor_available: bool,
}

struct UiMountedProjectionNodeDraft {
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    plan_digest: u64,
    role: worth_ui_host_contract::UiMountedMechanicalRole,
    participation: worth_ui_host_contract::UiMountedParticipation,
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    plan_index: Option<u32>,
    static_paint: Option<super::static_paint::UiMountedStaticPaintSeed>,
    semantic_text: Option<super::semantic_text::UiMountedSemanticTextSeed>,
    hit_test: Option<super::hit_test::UiMountedHitTestSeed>,
    focus_support: crate::capability::ComponentFocusSupport,
    focus_scope: Option<super::UiMountedFocusScope>,
    focus_container_owner: Option<crate::graph::UiGraphNodeIdentity>,
    component_id: Option<crate::capability::ComponentId>,
    portal_child_owner: Option<crate::capability::ComponentId>,
}

struct UiMountedFullProjectionInput<'basis, 'input, 'graph> {
    state: &'basis super::super::UiMountedIdentityState,
    lowering: &'basis UiMountedNodeLoweringContext<'input, 'graph>,
    requested_surfaces: &'basis [worth_ui_host_contract::UiSemanticSurfaceIdentity],
    has_published_frame: bool,
    changes: &'basis super::super::UiMountedProjectionChangeSnapshot,
}

struct UiMountedProjectionBuild {
    semantic: UiMountedSemanticProjection,
    cost: super::cost_accounting::UiMountedProjectionCostInput,
    replaced_order_rows: usize,
    presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
}

pub(crate) fn prepare_projection(
    state: &super::super::UiMountedIdentityState,
    input: UiMountedProjectionInput<'_, '_>,
) -> Result<UiPreparedMountedProjection, UiMountedProjectionDenial> {
    let lowering = UiMountedNodeLoweringContext {
        graph: input.graph,
        plan: input.plan,
        allocation_source: input.allocation_source,
        plan_digest: input.plan_digest,
        semantic_content: input.semantic_content,
        theme_values: input.theme_values,
        appearance_invalidation: input.appearance_invalidation,
        predecessor: input.semantic_predecessor,
        mechanics_predecessor_available: state
            .current_projection()
            .is_some_and(|current| current.plan_digest() == input.plan_digest),
    };
    let projection_changes = state.projection_change_snapshot();
    let delta_predecessor = state
        .current_projection()
        .filter(|current| current.plan_digest() == input.plan_digest)
        .filter(|current| {
            current
                .semantic_projection()
                .supports_surfaces(input.requested_surfaces)
        });
    let portal_changed_instances =
        portal_changed_instances(delta_predecessor, input.portal_overlays.as_ref());
    let portal_overlays_changed = !portal_changed_instances.is_empty();
    let delta = match (delta_predecessor, input.allocation_source.delta()) {
        (
            Some(current),
            crate::runtime::UiMountedAllocationProjectionDelta::Exact(allocation_delta),
        ) => delta::build(delta::UiMountedDeltaProjectionInput {
            state,
            lowering: &lowering,
            predecessor: current.semantic_projection(),
            requested_surfaces: input.requested_surfaces,
            changes: &projection_changes,
            allocation_delta,
        })?,
        _ => None,
    };
    let mut build = match delta {
        Some(build) => build,
        None => build_full_projection(UiMountedFullProjectionInput {
            state,
            lowering: &lowering,
            requested_surfaces: input.requested_surfaces,
            has_published_frame: state.has_published_frame(),
            changes: &projection_changes,
        })?,
    };
    build
        .semantic
        .apply_projection_inputs(input.semantic_content);
    if !portal_changed_instances.is_empty() {
        let mut changed = build.presentation_changed_instances.to_vec();
        changed.extend(
            build
                .semantic
                .portal_children_for_owners(&portal_changed_instances),
        );
        changed.extend(portal_changed_instances);
        changed.sort_unstable();
        changed.dedup();
        build.presentation_changed_instances = changed.into();
    }
    let counters = begin_build_counters(build.cost, build.replaced_order_rows)?;
    Ok(UiPreparedMountedProjection::new(
        UiPreparedMountedProjectionInput {
            plan_digest: input.plan_digest,
            semantic: build.semantic,
            preview: input.preview,
            visual_overlay: input.visual_overlay,
            portal_overlays: input.portal_overlays,
            projection_changes,
            presentation_changed_instances: build.presentation_changed_instances,
            portal_overlays_changed,
            counters,
            capability_generation: input.capability_generation,
            capability_profile_digest: input.capability_profile_digest,
            font_collection: input.font_collection,
        },
    ))
}

impl UiMountedNodeLoweringContext<'_, '_> {
    fn theme_value_changed(&self, graph_node: crate::graph::UiGraphNodeIdentity) -> bool {
        self.theme_values.has_theme_changes()
            && self
                .appearance_invalidation
                .as_ref()
                .is_some_and(|batch| batch.selects(graph_node))
    }
}

fn portal_changed_instances(
    predecessor: Option<&super::UiMountedProjectionFrame>,
    successor: &[super::super::UiMountedPortalOverlayProjectionInput],
) -> Vec<worth_ui_host_contract::UiMountedInstanceIdentity> {
    let predecessor = predecessor
        .map(super::UiMountedProjectionFrame::portal_overlay_inputs)
        .unwrap_or(&[]);
    if predecessor == successor {
        return Vec::new();
    }
    predecessor
        .iter()
        .chain(successor)
        .map(|overlay| overlay.owner())
        .collect()
}

fn begin_build_counters(
    cost: super::cost_accounting::UiMountedProjectionCostInput,
    replaced_order_rows: usize,
) -> Result<super::super::UiMountStageCounters, UiMountedProjectionDenial> {
    let mut counters = super::cost_accounting::begin_projection_cost(cost)?;
    if replaced_order_rows > 0 {
        counters
            .replace_rows::<worth_ui_host_contract::UiMountedInstanceIdentity>(replaced_order_rows)
            .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
    }
    Ok(counters)
}

fn build_full_projection(
    input: UiMountedFullProjectionInput<'_, '_, '_>,
) -> Result<UiMountedProjectionBuild, UiMountedProjectionDenial> {
    let instances = input.state.projection_instances(input.requested_surfaces);
    let current_instances = instances
        .iter()
        .map(|instance| instance.identity())
        .collect::<std::collections::BTreeSet<_>>();
    let predecessor_instances = input
        .lowering
        .predecessor
        .into_iter()
        .flat_map(UiMountedSemanticProjection::mounted_instances)
        .collect::<Vec<_>>();
    let retired = predecessor_instances
        .iter()
        .filter(|instance| !current_instances.contains(instance))
        .count();
    let mut presentation_changed_instances = current_instances.iter().copied().collect::<Vec<_>>();
    presentation_changed_instances.extend(predecessor_instances.iter().copied());
    presentation_changed_instances.sort_unstable();
    presentation_changed_instances.dedup();
    let nodes = instances
        .iter()
        .map(|instance| {
            input
                .lowering
                .lower(instance)
                .map(UiMountedProjectionNodeDraft::materialize)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let surfaces = projection_surfaces(input.state, input.requested_surfaces)?;
    let node_count = nodes.len();
    let surface_count = surfaces.len();
    let has_predecessor = input.has_published_frame || input.lowering.predecessor.is_some();
    let work_class = if has_predecessor {
        super::super::UiMountWorkClass::ComparisonRequired
    } else {
        super::super::UiMountWorkClass::InitialMount
    };
    let mut semantic = UiMountedSemanticProjection::initial(nodes, surfaces);
    semantic.inherit_projection_inputs(input.lowering.predecessor);
    Ok(UiMountedProjectionBuild {
        semantic,
        cost: super::cost_accounting::UiMountedProjectionCostInput {
            work_class,
            considered: node_count
                .checked_mul(3)
                .and_then(|count| count.checked_add(surface_count))
                .and_then(|count| count.checked_add(predecessor_instances.len()))
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?,
            index_entries: node_count
                .checked_mul(2)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?,
            projected_instances: node_count,
            surface_instance_pairs: node_count,
            changed_bindings: usize::from(!has_predecessor) * surface_count,
            reused: 0,
            retired,
            coalesced: input.changes.coalesced(),
            overflowed: input.changes.overflowed(),
        },
        replaced_order_rows: node_count,
        presentation_changed_instances: presentation_changed_instances.into(),
    })
}

fn projection_surfaces(
    state: &super::super::UiMountedIdentityState,
    requested: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
) -> Result<Vec<UiMountedProjectionSurface>, UiMountedProjectionDenial> {
    requested
        .iter()
        .map(|surface| {
            let (binding, audience) = state
                .projection_surface(*surface)
                .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
            Ok(UiMountedProjectionSurface {
                surface: *surface,
                binding: binding.binding_generation(),
                audience,
            })
        })
        .collect()
}
