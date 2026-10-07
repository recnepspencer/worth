//! Source publication installs exact marks or evicts the derived index.
use super::admission::IndexAdmission;
use super::source_alignment::{BranchMarkRoot, HistoricalMarkState, RetainedTouchDelivery};
use super::{delivery, index_capacity, retention, SourceInvalidationOwner};
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;
use std::sync::Arc;
use worth_relational::facade::mvcc::{
    CompanionPreflightStop, CompanionPublicationCompletionObserver,
    PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};

impl RelationalPublicationCompanion for SourceInvalidationOwner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        if context.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let cell = self.selected_cell(context)?;
        match self.prepare_delivery(&cell, context) {
            Ok(effect) => Ok(effect),
            Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => {
                // Source authority cannot depend on retaining a derived index.
                // The empty image was paid for before any rows could fill it.
                let vacant = Arc::clone(
                    self.branches
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .vacant
                        .as_ref()
                        .expect("minted cell has its empty image"),
                );
                let reserved = cell.reserve_preflight(context)?;
                #[cfg(feature = "test-query-execution-observer")]
                super::delivery_observation::record_delivery(&super::logical_marking::NativeMarkingReport {
                    commit: context.canonical_commit().commit.commit_id,
                    precision: super::logical_marking::NativeMarkingPrecision::RetainedCapacityExhausted(Default::default()),
                });
                context.seal_replacement(reserved, vacant)
            }
            Err(stop) => Err(stop),
        }
    }
}

impl SourceInvalidationOwner {
    fn prepare_delivery(
        &self,
        cell: &super::publication_cell::SelectedPublicationCell,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
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
        #[cfg(feature = "test-query-execution-observer")]
        super::delivery_observation::record_delivery(&report);
        let delivery_bytes = index_capacity::arc_bytes::<RetainedTouchDelivery>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        context.bytes(delivery_bytes)?;
        let key_bytes = retained_key_bytes
            .checked_add(delivery_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        // Retained keys only sharpen late settlement replay. Without room for
        // them, replay through this commit is a declared-change discontinuity.
        let reserve = |bytes, context: &mut _| retention::reserve(&self.resources, bytes, context);
        let (keys, key_capacity) = match reserve(key_bytes, context) {
            Ok(capacity) => (keys, capacity),
            Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. })
                if keys.is_some() =>
            {
                (None, reserve(delivery_bytes, context)?)
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
        let mut left = None;
        if root.past.len() > self.resources.installation().maximum_retained_positions {
            context.ordered_remove::<Option<worth_relational::facade::publication::PatchStreamPosition>, HistoricalMarkState>(root.past.len())?;
            if let Some(oldest) = root.past.get_min().map(|(position, _)| *position) {
                left = root.past.remove(&oldest);
            }
        }
        root.current = Arc::new(state);
        root.last_native_marking = Some(report);
        let left = left.as_ref().map(|left| &*left.state);
        retention::admit_root(&mut root, left, &self.resources, context)?;
        // A same-position Query edit must retain its predecessor while it
        // prepares the replacement. Keep this cache only if a whole current
        // index still has that headroom; otherwise publish the paid empty
        // image now, rather than strand subsequent registration behind it.
        let edit_bytes = retention::state_bound(&root.current)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let retained = self.resources.retained_capacity_bytes();
        let maximum = self.resources.installation().maximum_retained_bytes;
        if edit_bytes > maximum.saturating_sub(retained) {
            return Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted {
                requested: edit_bytes,
                retained,
                maximum,
            });
        }
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
            for membership in selected {
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
