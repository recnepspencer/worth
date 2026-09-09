use super::frame_storage::{UiMountedProjectionSurface, UiMountedSemanticProjection};
use super::geometry::lower_allocation;
use super::mechanical_role::mechanical_role;
use super::participation::lower_participation;
use super::prepared_projection::{UiPreparedMountedProjection, UiPreparedMountedProjectionInput};
use super::{UiMountedAppearanceProjectionSelection, UiMountedProjectionDenial};

mod appearance_input;
mod delta;
#[path = "lowering/node_draft.rs"]
mod node_draft;
mod node_lowering;
pub(super) mod portal_changes;

pub(crate) struct UiMountedProjectionInput<'input, 'graph> {
    pub(crate) graph: crate::graph::UiGraphAuthority<'graph>,
    pub(crate) plan_digest: u64,
    pub(crate) plan: super::super::UiMountedPlanProjectionSource<'input>,
    pub(crate) allocation_source: &'input crate::runtime::UiMountedAllocationProjectionSource,
    pub(crate) occurrence_geometry: &'input super::super::UiMountedOccurrenceGeometryState,
    pub(crate) requested_surfaces: &'input [worth_ui_host_contract::UiSemanticSurfaceIdentity],
    pub(crate) preview: Option<UiMountedPreviewProjectionInput>,
    pub(crate) visual_overlay: Option<super::super::UiMountedVisualOverlayProjectionInput>,
    pub(crate) portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    pub(crate) semantic_content: &'input super::super::UiMountedSemanticContentInput,
    pub(crate) theme_values: &'input super::super::UiMountedThemeValueSource,
    pub(crate) appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationInput<'input>>,
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
    state: &'input super::super::UiMountedIdentityState,
    graph: crate::graph::UiGraphAuthority<'graph>,
    plan: super::super::UiMountedPlanProjectionSource<'input>,
    allocation_source: &'input crate::runtime::UiMountedAllocationProjectionSource,
    occurrence_geometry: &'input super::super::UiMountedOccurrenceGeometryState,
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
    appearance_allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    appearance_clip: super::appearance::UiMountedAppearanceClip,
    surface_paint_posture: super::super::UiMountedSurfacePaintPosture,
    surface_paint_order: Option<u32>,
    has_appearance_attachment: bool,
    clip_ancestry_entries: usize,
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
    appearance_selection: &'basis UiMountedAppearanceProjectionSelection,
    has_published_frame: bool,
    changes: &'basis super::super::UiMountedProjectionChangeSnapshot,
}

struct UiMountedProjectionBuild {
    semantic: UiMountedSemanticProjection,
    cost: super::cost_accounting::UiMountedProjectionCostInput,
    replaced_order_rows: usize,
    presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    retired_appearance_instances: Vec<worth_ui_host_contract::UiMountedInstanceIdentity>,
}

pub(crate) fn prepare_projection(
    state: &super::super::UiMountedIdentityState,
    input: UiMountedProjectionInput<'_, '_>,
) -> Result<UiPreparedMountedProjection, UiMountedProjectionDenial> {
    for surface in input.requested_surfaces {
        let binding = state
            .projection_surface(*surface)
            .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?
            .0
            .binding_generation();
        if !input
            .occurrence_geometry
            .validates_binding(*surface, binding)
        {
            return Err(UiMountedProjectionDenial::OccurrenceGeometry(
                super::super::UiMountedOccurrenceGeometryDenial::StaleOccurrenceGeometry,
            ));
        }
    }
    let appearance_input = input.appearance_invalidation;
    if appearance_input.as_ref().is_some_and(|input| {
        input
            .pending
            .as_ref()
            .is_some_and(|batch| batch.basis() != input.index.basis())
    }) {
        return Err(UiMountedProjectionDenial::AppearanceSelectionFrameMismatch);
    }
    let pending = appearance_input
        .as_ref()
        .and_then(|input| input.pending.as_ref());
    let mut appearance_selection =
        UiMountedAppearanceProjectionSelection::derive(state, input.requested_surfaces, pending)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
    let lowering = UiMountedNodeLoweringContext {
        state,
        graph: input.graph,
        plan: input.plan,
        allocation_source: input.allocation_source,
        occurrence_geometry: input.occurrence_geometry,
        plan_digest: input.plan_digest,
        semantic_content: input.semantic_content,
        theme_values: input.theme_values,
        appearance_invalidation: pending.cloned(),
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
        portal_changes::changed_owners(delta_predecessor, input.portal_overlays.as_ref());
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
            appearance_selection: &appearance_selection,
        })?,
        _ => None,
    };
    let mut build = match delta {
        Some(build) => build,
        None => build_full_projection(UiMountedFullProjectionInput {
            state,
            lowering: &lowering,
            requested_surfaces: input.requested_surfaces,
            appearance_selection: &appearance_selection,
            has_published_frame: state.has_published_frame(),
            changes: &projection_changes,
        })?,
    };
    appearance_selection.set_retired_instances(build.retired_appearance_instances.clone());
    build
        .semantic
        .apply_projection_inputs(input.semantic_content);
    let mut portal_geometry_changes = Vec::new();
    if !portal_changed_instances.is_empty() {
        let mut changed = build.presentation_changed_instances.to_vec();
        let (children, work) = portal_changes::children_in_transition(
            delta_predecessor.map(super::UiMountedProjectionFrame::semantic_projection),
            &build.semantic,
            &portal_changed_instances,
        );
        portal_geometry_changes.extend_from_slice(&children);
        portal_geometry_changes.extend_from_slice(&portal_changed_instances);
        portal_geometry_changes.sort_unstable();
        portal_geometry_changes.dedup();
        changed.extend(children);
        build.cost.index_entries = build
            .cost
            .index_entries
            .checked_add(work)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        changed.extend(portal_changed_instances);
        changed.sort_unstable();
        changed.dedup();
        build.presentation_changed_instances = changed.into();
    }
    let appearance_invalidation = appearance_input::finish(
        state,
        input.requested_surfaces,
        appearance_input,
        &mut build,
        &mut appearance_selection,
        &portal_geometry_changes,
        &projection_changes,
    )?;
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
            appearance_selection: std::rc::Rc::new(appearance_selection),
            appearance_invalidation,
            theme_revision: input.theme_values.active_theme_revision(),
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
                .is_some_and(|batch| batch.selects_graph(graph_node))
    }
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
    let mut retired_instances = predecessor_instances
        .iter()
        .filter(|instance| {
            !current_instances.contains(instance)
                && input.state.projection_instance(**instance).is_none()
        })
        .copied()
        .collect::<Vec<_>>();
    // A request omitting a still-mounted surface is not an unmount. Its
    // appearance predecessor remains owned until that surface is requested.
    // An identity-only capsule can retain appearance after losing semantic
    // rows. Its explicit unmounts still retire physical predecessors.
    retired_instances.extend(input.changes.retired_instances());
    retired_instances.sort_unstable();
    retired_instances.dedup();
    let retired = retired_instances.len();
    let mut presentation_changed_instances = current_instances.iter().copied().collect::<Vec<_>>();
    presentation_changed_instances.extend(predecessor_instances.iter().copied());
    presentation_changed_instances.sort_unstable();
    presentation_changed_instances.dedup();
    let mut clip_ancestry_entries = 0usize;
    let nodes = instances
        .iter()
        .map(|instance| {
            let draft = input.lowering.lower(instance)?;
            clip_ancestry_entries = clip_ancestry_entries
                .checked_add(draft.clip_ancestry_entries)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
            Ok(draft.materialize())
        })
        .collect::<Result<Vec<_>, UiMountedProjectionDenial>>()?;
    let surfaces = projection_surfaces(input.state, input.requested_surfaces)?;
    let node_count = nodes.len();
    let surface_count = surfaces.len();
    let has_predecessor = input.has_published_frame || input.lowering.predecessor.is_some();
    let work_class = if has_predecessor {
        super::super::UiMountWorkClass::ComparisonRequired
    } else {
        super::super::UiMountWorkClass::InitialMount
    };
    let (mut semantic, portal_work) = UiMountedSemanticProjection::build_initial(nodes, surfaces);
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
                .and_then(|count| count.checked_add(clip_ancestry_entries))
                .and_then(|count| count.checked_add(portal_work.key_probes()))
                .and_then(|count| count.checked_add(portal_work.node_copies()))
                .and_then(|count| {
                    count.checked_add(input.appearance_selection.index_entries_touched())
                })
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
        retired_appearance_instances: retired_instances,
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
