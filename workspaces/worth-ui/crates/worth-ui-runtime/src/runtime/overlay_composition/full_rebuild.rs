use super::extent::UiOverlayMotionSnapshot;
use super::materialization::{current_portals, materialize_backdrops, reserve};
use super::order::compile_order;
use super::planner::{
    UiOverlayCompositionDenial, UiOverlayCompositionInput, UiOverlayCompositionState,
    UiOverlayPlanCounters, UiOverlayReservation,
};
use super::relation_cache::{self, UiOverlayRelationCache};
use super::snapshot::UiOverlayStackSnapshot;

pub(super) fn compile_full(
    state: &UiOverlayCompositionState,
    input: &UiOverlayCompositionInput<'_>,
    relation_cache: &UiOverlayRelationCache,
) -> Result<
    (
        UiOverlayStackSnapshot,
        UiOverlayReservation,
        UiOverlayPlanCounters,
    ),
    UiOverlayCompositionDenial,
> {
    let portal_rows = current_portals(input, state.capacity)?;
    let portal_stack_rows_read = portal_rows.source_rows_read();
    let portal_binding_entries_read = portal_rows.binding_entries_read();
    let portals = portal_rows.rows();
    let declarations = state
        .declarations
        .iter()
        .filter(|declaration| declaration.surface() == input.extent.declaration_surface())
        .cloned()
        .collect::<Vec<_>>();
    let relations = relation_cache::for_surface(relation_cache, input.extent.declaration_surface());
    let backdrops = materialize_backdrops(
        &declarations,
        &portals,
        input.extent,
        input.motion,
        state.capacity,
    )?;
    let (participants, relation_edges) = compile_order(&portals, &backdrops, &relations)?;
    let reservation = reserve(
        portals.len(),
        backdrops.len(),
        participants.len(),
        relation_edges,
        state.capacity,
    )?;
    let snapshot = UiOverlayStackSnapshot::seal(
        input.generation.clone(),
        input.extent.declaration_surface(),
        input.extent.runtime_surface(),
        input.presentation,
        input.portal_snapshot.owner_revision(),
        state.declaration_revision,
        input.extent.revision(),
        input.motion.map(UiOverlayMotionSnapshot::owner_revision),
        participants,
    );
    Ok((
        snapshot,
        reservation,
        UiOverlayPlanCounters {
            portal_stack_rows_read,
            portal_binding_entries_read,
            backdrop_declarations_selected: declarations.len(),
            overlay_relation_edges_visited: relation_edges,
            backdrop_mechanics_changed: backdrops.len(),
            ..UiOverlayPlanCounters::default()
        },
    ))
}
