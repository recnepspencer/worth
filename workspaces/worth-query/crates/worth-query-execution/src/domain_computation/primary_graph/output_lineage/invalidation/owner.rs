use std::fmt;
use std::sync::{Arc, Mutex};

use im::OrdMap;
use worth_relational::facade::{
    history::BranchId,
    mvcc::{
        CompanionBranchCell, CompanionPreflightStop, CompanionPublicationCompletionObserver,
        PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
        RelationalPublicationCompanion,
    },
};

use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

use super::admission::IndexAdmission;
use super::index_capacity;
use super::source_alignment::{BranchMarkRoot, HistoricalMarkState, RetainedTouchDelivery};
use super::{delivery, retention};

/// Callback custody contains derived cells and capacity only. It cannot enter
/// the runtime mutex, retain a source provider, or create a registration cycle.
pub(in crate::domain_computation) struct SourceInvalidationOwner {
    pub(super) runtime_instance_id: u64,
    pub(super) resources: WorthQueryInvalidationResources,
    pub(super) branches: Mutex<BranchCells>,
}

pub(super) struct BranchCells {
    pub(super) cells: OrdMap<BranchId, CompanionBranchCell<BranchMarkRoot>>,
    pub(super) branch_name_bytes: u64,
    pub(super) retained_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
}

impl fmt::Debug for SourceInvalidationOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceInvalidationOwner")
            .field("resources", &self.resources)
            .finish_non_exhaustive()
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation) fn new(
        resources: WorthQueryInvalidationResources,
        runtime_instance_id: u64,
    ) -> Self {
        Self {
            runtime_instance_id,
            resources,
            branches: Mutex::new(BranchCells {
                cells: OrdMap::new(),
                branch_name_bytes: 0,
                retained_capacity: None,
            }),
        }
    }

    fn selected_cell(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<CompanionBranchCell<BranchMarkRoot>, CompanionPreflightStop> {
        // The writer waits for the map like readers do. Every holder only
        // looks up, or mints and inserts one cell through atomic capacity
        // counters; none calls out to another lock, so no cycle can form.
        let mut branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        context.work(
            (context.branch_id().0.len() as u64)
                .checked_mul(
                    index_capacity::ordered_navigation_work(branches.cells.len())
                        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                )
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        context.ordered_read(branches.cells.len())?;
        if let Some(cell) = branches.cells.get(context.branch_id()) {
            return Ok(cell.clone());
        }
        context.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        context.bytes(
            index_capacity::arc_bytes::<super::mark_state::MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut initial = BranchMarkRoot::initial();
        retention::admit_state(
            Arc::get_mut(&mut initial.current).expect("new state is exclusive"),
            &self.resources,
            context,
        )?;
        retention::admit_root(&mut initial, &self.resources, context)?;
        let cell = context.mint_selected_branch_cell(Arc::new(initial))?;
        let branch_bytes = context.branch_id().0.len() as u64;
        context.bytes(branch_bytes)?;
        context.work(branch_bytes)?;
        let next_name_bytes = branches
            .branch_name_bytes
            .checked_add(branch_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let next_count = branches
            .cells
            .len()
            .checked_add(1)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bound = index_capacity::retained_map_bytes::<
            BranchId,
            CompanionBranchCell<BranchMarkRoot>,
        >(next_count)
        .and_then(|n| n.checked_add(next_name_bytes))
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let capacity = retention::reserve(&self.resources, bound, context)?;
        context
            .ordered_edit::<BranchId, CompanionBranchCell<BranchMarkRoot>>(branches.cells.len())?;
        branches
            .cells
            .insert(context.branch_id().clone(), cell.clone());
        branches.branch_name_bytes = next_name_bytes;
        branches.retained_capacity = Some(capacity);
        Ok(cell)
    }
}

impl RelationalPublicationCompanion for SourceInvalidationOwner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        if context.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let cell = self.selected_cell(context)?;
        let reserved = cell.reserve_preflight(context)?;
        let observed = reserved.current();
        let budget = self.resources.preflight_budget();
        let delivered = delivery::selectors(context, budget)?;
        let commit = context.canonical_commit().commit.commit_id;
        let delivery::MarkedDelivery {
            state,
            report,
            selected,
            mut hint_allowance,
            keys,
            retained_key_bytes,
        } = delivery::mark(
            &observed.payload().current,
            delivered,
            commit,
            (budget, &self.resources),
            context,
        )?;
        let delivery_bytes = index_capacity::arc_bytes::<RetainedTouchDelivery>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        context.bytes(delivery_bytes)?;
        let key_bytes = retained_key_bytes
            .checked_add(delivery_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        // Retained keys only sharpen late settlement replay. Without room for
        // them, replay through this commit is a declared-change discontinuity.
        let (keys, key_capacity) = match retention::reserve(&self.resources, key_bytes, context) {
            Ok(capacity) => (keys, capacity),
            Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. })
                if keys.is_some() =>
            {
                (
                    None,
                    retention::reserve(&self.resources, delivery_bytes, context)?,
                )
            }
            Err(stop) => return Err(stop),
        };
        let delivered = Arc::new(RetainedTouchDelivery {
            keys,
            _capacity: key_capacity,
        });
        context.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut root = (**observed.payload()).clone();
        context.ordered_edit::<Option<worth_relational::facade::publication::PatchStreamPosition>, HistoricalMarkState>(root.past.len())?;
        root.past.insert(
            observed.position(),
            HistoricalMarkState {
                root_id: observed.root_id(),
                commit_id: observed.commit_id(),
                state: Arc::clone(&observed.payload().current),
                next_delivery: delivered,
            },
        );
        if root.past.len() > self.resources.installation().maximum_retained_positions {
            context.ordered_remove::<Option<worth_relational::facade::publication::PatchStreamPosition>, HistoricalMarkState>(root.past.len())?;
            if let Some(oldest) = root.past.get_min().map(|(position, _)| *position) {
                root.past.remove(&oldest);
            }
        }
        root.current = Arc::new(state);
        root.last_native_marking = Some(report);
        retention::admit_root(&mut root, &self.resources, context)?;
        let mut effect = context.seal_replacement(reserved, Arc::new(root))?;
        if !selected.is_empty() {
            let retained = retention::reserve(
                &self.resources,
                CompanionPublicationCompletionObserver::retained_bytes(),
                context,
            )?;
            let observer = effect.attach_completion_observer(context, retained)?;
            // Each hint's preparation was admitted with the marking that
            // selected it; construction draws down exactly that allowance.
            let prepared_bytes = u64::try_from(selected.len())
                .ok()
                .and_then(|count| count.checked_mul((2 * std::mem::size_of::<usize>()) as u64))
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            hint_allowance.bytes(prepared_bytes)?;
            let mut prepared = Vec::with_capacity(selected.len());
            for (membership, identity) in selected {
                let branch_bytes = u64::try_from(context.branch_id().0.len())
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
                let hint_bytes = RequiredWorkMembership::native_hint_bytes();
                let retained_branch_bytes = RequiredWorkMembership::native_branch_retained_bytes(
                    context.branch_id().0.len(),
                )
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                hint_allowance.bytes(hint_bytes)?;
                hint_allowance.bytes(retained_branch_bytes)?;
                hint_allowance.work(
                    branch_bytes
                        .checked_add(7)
                        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                )?;
                let hint_capacity =
                    retention::reserve(&self.resources, hint_bytes, &mut hint_allowance)?;
                let branch_capacity = retention::reserve(
                    &self.resources,
                    retained_branch_bytes,
                    &mut hint_allowance,
                )?;
                let branch = RequiredWorkMembership::prepared_native_branch(
                    context.branch_id().clone(),
                    branch_capacity,
                );
                let hint = RequiredWorkMembership::prepared_native_hint(
                    observer.clone(),
                    identity,
                    branch,
                    hint_capacity,
                );
                prepared.push((membership, hint));
            }
            // Every Box and Vec slot is already owned. A later preflight
            // denial cannot leave a partial set of published token hints.
            for (membership, hint) in prepared {
                membership.prepare_hint(hint);
            }
        }
        Ok(effect)
    }
}
