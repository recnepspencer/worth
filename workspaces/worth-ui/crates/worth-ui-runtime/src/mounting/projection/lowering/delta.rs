use super::super::frame_storage::{UiMountedProjectionSurface, UiMountedSemanticProjection};
use super::super::{UiMountedAppearanceProjectionSelection, UiMountedProjectionDenial};
use super::{UiMountedNodeLoweringContext, UiMountedProjectionBuild};

pub(super) struct UiMountedDeltaProjectionInput<'borrow, 'input, 'graph> {
    pub(super) state: &'borrow super::super::super::UiMountedIdentityState,
    pub(super) lowering: &'borrow UiMountedNodeLoweringContext<'input, 'graph>,
    pub(super) predecessor: &'borrow UiMountedSemanticProjection,
    pub(super) requested_surfaces: &'borrow [worth_ui_host_contract::UiSemanticSurfaceIdentity],
    pub(super) changes: &'borrow super::super::super::UiMountedProjectionChangeSnapshot,
    pub(super) allocation_delta: &'borrow crate::runtime::UiMountedAllocationExactDelta,
    pub(super) appearance_selection: &'borrow UiMountedAppearanceProjectionSelection,
}

struct UiMountedDeltaScope {
    changed: Vec<worth_ui_host_contract::UiMountedInstanceIdentity>,
    retired: Vec<worth_ui_host_contract::UiMountedInstanceIdentity>,
    changed_surfaces: Vec<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    removed_surfaces: Vec<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    declared_semantic_changed: bool,
    allocation_delta_observed: bool,
    initial_index_entries: usize,
}

struct UiMountedDeltaApplication {
    semantic: UiMountedSemanticProjection,
    index_entries: usize,
    changed_projected: usize,
    changed_projected_outside_changed_surfaces: usize,
    membership_changed: bool,
}

pub(super) fn build(
    input: UiMountedDeltaProjectionInput<'_, '_, '_>,
) -> Result<Option<UiMountedProjectionBuild>, UiMountedProjectionDenial> {
    let scope = UiMountedDeltaScope::derive(&input)?;
    if !scope.has_work() {
        return UiMountedDeltaApplication::begin(&input, &scope)
            .finish(&input, &scope, 0, 0)
            .map(Some);
    }
    let mut application = UiMountedDeltaApplication::begin(&input, &scope);
    application.apply_retired(&scope)?;
    application.apply_changed(&input, &scope)?;
    let changed_binding_count = application.apply_surface_changes(&input, &scope)?;
    let replaced_order_rows = application.replace_order_if_needed(&input);
    application
        .finish(&input, &scope, changed_binding_count, replaced_order_rows)
        .map(Some)
}

impl UiMountedDeltaScope {
    fn derive(
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
    ) -> Result<Self, UiMountedProjectionDenial> {
        let allocation_affected = input
            .state
            .try_projection_instances_for_graph_nodes(input.allocation_delta.changed_graph_nodes())
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        let content_graph_nodes = input
            .lowering
            .semantic_content
            .graph_nodes()
            .collect::<Vec<_>>();
        let content_affected = input
            .state
            .try_projection_instances_for_graph_nodes(&content_graph_nodes)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        let theme_changed = input.lowering.theme_values.has_theme_changes()
            && !input.appearance_selection.selected_instances().is_empty();
        let mut changed = input.changes.changed_instances().collect::<Vec<_>>();
        changed.extend_from_slice(allocation_affected.instances());
        changed.extend_from_slice(content_affected.instances());
        if theme_changed {
            changed.extend_from_slice(input.appearance_selection.selected_instances());
        }
        changed.sort();
        changed.dedup();
        let initial_index_entries = input
            .allocation_delta
            .journal_entries_touched()
            .checked_add(allocation_affected.index_entries_touched())
            .and_then(|count| count.checked_add(content_affected.index_entries_touched()))
            .and_then(|count| count.checked_add(input.appearance_selection.index_entries_touched()))
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        Ok(Self {
            changed,
            retired: input.changes.retired_instances().collect(),
            changed_surfaces: input.changes.changed_surfaces().collect(),
            removed_surfaces: input.changes.removed_surfaces().collect(),
            declared_semantic_changed: input.changes.changed_instances().next().is_some()
                || input.changes.retired_instances().next().is_some()
                || input.changes.order_changed()
                || !input.lowering.semantic_content.is_empty()
                || theme_changed,
            allocation_delta_observed: input.allocation_delta.journal_entries_touched() > 0
                || !input.allocation_delta.changed_graph_nodes().is_empty(),
            initial_index_entries,
        })
    }

    fn has_work(&self) -> bool {
        self.declared_semantic_changed
            || !self.changed.is_empty()
            || !self.retired.is_empty()
            || !self.changed_surfaces.is_empty()
            || !self.removed_surfaces.is_empty()
            || self.allocation_delta_observed
    }

    fn work_class(&self) -> super::super::super::UiMountWorkClass {
        if !self.has_work() {
            super::super::super::UiMountWorkClass::UnchangedReuse
        } else if self.declared_semantic_changed {
            super::super::super::UiMountWorkClass::SemanticDelta
        } else if self.allocation_delta_observed {
            super::super::super::UiMountWorkClass::BatchDelta
        } else {
            super::super::super::UiMountWorkClass::SurfaceOnly
        }
    }
}

impl UiMountedDeltaApplication {
    fn begin(
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        scope: &UiMountedDeltaScope,
    ) -> Self {
        Self {
            semantic: input.predecessor.clone(),
            index_entries: scope.initial_index_entries,
            changed_projected: 0,
            changed_projected_outside_changed_surfaces: 0,
            membership_changed: false,
        }
    }

    fn apply_retired(
        &mut self,
        scope: &UiMountedDeltaScope,
    ) -> Result<(), UiMountedProjectionDenial> {
        for instance in &scope.retired {
            self.membership_changed |= self.semantic.contains(*instance);
            self.index_entries =
                add_mutation_work(self.index_entries, self.semantic.remove_node(*instance))?;
        }
        Ok(())
    }

    fn apply_changed(
        &mut self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        scope: &UiMountedDeltaScope,
    ) -> Result<(), UiMountedProjectionDenial> {
        for instance in &scope.changed {
            let previously_projected = self.semantic.contains(*instance);
            match input.state.projection_instance(*instance).filter(|view| {
                input
                    .requested_surfaces
                    .contains(&view.basis().semantic_surface_identity())
            }) {
                Some(view) => {
                    self.replace_changed_node(input, scope, &view, previously_projected)?
                }
                None => {
                    self.index_entries = add_mutation_work(
                        self.index_entries,
                        self.semantic.remove_node(*instance),
                    )?;
                    self.membership_changed |= previously_projected;
                }
            }
        }
        Ok(())
    }

    fn replace_changed_node(
        &mut self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        scope: &UiMountedDeltaScope,
        view: &super::super::super::UiMountedInstanceIdentityView,
        previously_projected: bool,
    ) -> Result<(), UiMountedProjectionDenial> {
        let belongs_to_changed_surface = scope
            .changed_surfaces
            .contains(&view.basis().semantic_surface_identity());
        let node = input.lowering.lower(view)?.materialize();
        self.index_entries = self
            .index_entries
            .checked_add(2)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        self.index_entries =
            add_mutation_work(self.index_entries, self.semantic.insert_node(node))?;
        self.changed_projected += 1;
        self.changed_projected_outside_changed_surfaces += usize::from(!belongs_to_changed_surface);
        self.membership_changed |= !previously_projected;
        Ok(())
    }

    fn apply_surface_changes(
        &mut self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        scope: &UiMountedDeltaScope,
    ) -> Result<usize, UiMountedProjectionDenial> {
        let mut applied = 0usize;
        for surface in scope
            .removed_surfaces
            .iter()
            .filter(|surface| input.requested_surfaces.contains(surface))
        {
            self.index_entries =
                add_mutation_work(self.index_entries, self.semantic.remove_surface(*surface))?;
            applied += 1;
        }
        for surface in scope
            .changed_surfaces
            .iter()
            .filter(|surface| input.requested_surfaces.contains(surface))
        {
            self.replace_surface(input, *surface)?;
            applied += 1;
        }
        Ok(applied)
    }

    fn replace_surface(
        &mut self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedProjectionDenial> {
        let (binding, audience) = input
            .state
            .projection_surface(surface)
            .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
        self.index_entries = add_mutation_work(
            self.index_entries,
            self.semantic.replace_surface(UiMountedProjectionSurface {
                surface,
                binding: binding.binding_generation(),
                audience,
            }),
        )?;
        Ok(())
    }

    fn replace_order_if_needed(
        &mut self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
    ) -> usize {
        if !input.changes.order_changed() && !self.membership_changed {
            return 0;
        }
        if let Some(order) = input
            .state
            .projection_order_snapshot(input.requested_surfaces)
        {
            let declared_rows = usize::from(input.changes.order_changed()) * order.len();
            self.semantic.replace_order_snapshot(order);
            return declared_rows;
        }
        let order = input.state.projection_order(input.requested_surfaces);
        let copied_rows = order.len();
        self.semantic.replace_order(order);
        copied_rows
    }

    fn finish(
        self,
        input: &UiMountedDeltaProjectionInput<'_, '_, '_>,
        scope: &UiMountedDeltaScope,
        changed_binding_count: usize,
        replaced_order_rows: usize,
    ) -> Result<UiMountedProjectionBuild, UiMountedProjectionDenial> {
        let affected_surface_pairs = self
            .semantic
            .surface_instance_count(&scope.changed_surfaces)
            .checked_add(self.changed_projected_outside_changed_surfaces)
            .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?;
        let reused = self
            .semantic
            .node_count()
            .saturating_sub(self.changed_projected);
        Ok(UiMountedProjectionBuild {
            semantic: self.semantic,
            cost: super::super::cost_accounting::UiMountedProjectionCostInput {
                work_class: scope.work_class(),
                considered: scope
                    .changed
                    .len()
                    .checked_add(scope.retired.len())
                    .and_then(|count| count.checked_add(changed_binding_count))
                    .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?,
                index_entries: self.index_entries,
                projected_instances: self.changed_projected,
                surface_instance_pairs: affected_surface_pairs,
                changed_bindings: changed_binding_count,
                reused,
                retired: scope.retired.len(),
                coalesced: input.changes.coalesced(),
                overflowed: input.changes.overflowed(),
            },
            replaced_order_rows,
            presentation_changed_instances: presentation_changed_instances(scope),
            retired_appearance_instances: scope.retired.clone(),
        })
    }
}

fn presentation_changed_instances(
    scope: &UiMountedDeltaScope,
) -> std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]> {
    let mut instances = scope.changed.clone();
    instances.extend_from_slice(&scope.retired);
    instances.sort_unstable();
    instances.dedup();
    instances.into()
}

fn add_mutation_work(
    total: usize,
    work: crate::runtime::persistent_index::UiPersistentIndexMutationWork,
) -> Result<usize, UiMountedProjectionDenial> {
    total
        .checked_add(work.key_probes())
        .and_then(|count| count.checked_add(work.node_copies()))
        .ok_or(UiMountedProjectionDenial::CostCounterOverflow)
}
