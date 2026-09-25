use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{ManagedDerivedViewState, WorthQueryManagedDerivedValue};
use crate::domain_computation::primary_graph::application_query::derived_view::{
    publication::{PreparedViewTransition, ViewChange},
    registry::RegisteredDerivedView,
    ViewPublicationBasis,
};

impl<Key, Value> ManagedDerivedViewState<Key, Value>
where
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
{
    fn warm_at(&self, before: &CompositeCommitIdentity) -> bool {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !retained.disposed
            && !retained.revision_exhausted
            && !retained.cold
            && &retained.current_commit == before
    }

    fn prepare_transition(
        &self,
        before: &CompositeCommitIdentity,
        changes: &[ViewChange],
        maximum_work_units: usize,
    ) -> Option<PreparedViewTransition<Key>> {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed
            || retained.revision_exhausted
            || retained.cold
            || &retained.current_commit != before
        {
            return None;
        }
        let mut affected = retained.index.affected(changes, maximum_work_units)?;
        if affected.entries.len().saturating_add(retained.dirty.len()) > maximum_work_units {
            return None;
        }
        affected.entries.extend(retained.dirty.iter().cloned());
        if affected.entries.len() > self.limits.maximum_entries() {
            return None;
        }
        Some(PreparedViewTransition {
            before: before.clone(),
            revision: retained.revision,
            work_units: affected.work_units.saturating_add(retained.dirty.len()),
            affected,
        })
    }

    fn apply_transition(
        &self,
        prepared: PreparedViewTransition<Key>,
        after: CompositeCommitIdentity,
    ) {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed {
            return;
        }
        if retained.current_commit != prepared.before || retained.revision != prepared.revision {
            retained.cold = true;
        } else {
            if prepared.affected.membership {
                if self.entry_query.is_some() && self.secondary_entry_query.is_some() {
                    retained.membership_dirty = true;
                } else {
                    retained.cold = true;
                }
            }
            retained.dirty = prepared.affected.entries;
        }
        retained.current_commit = after;
        retained.advance_revision();
    }

    fn advance_cold(&self, after: CompositeCommitIdentity) {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !retained.disposed {
            retained.cold = true;
            retained.current_commit = after;
            retained.advance_revision();
        }
    }
}

impl<Key, Value> RegisteredDerivedView for ManagedDerivedViewState<Key, Value>
where
    Key: Clone + Ord + Send + Sync + 'static,
    Value: WorthQueryManagedDerivedValue,
{
    fn dispose(&self) {
        self.discard();
        self.retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .disposed = true;
    }

    fn belongs_to(&self, basis: &ViewPublicationBasis<'_>) -> bool {
        &self.branch == basis.relational_branch
            && &self.product_branch == basis.product_branch
            && self.incarnation == basis.incarnation
    }

    fn warm_at(&self, before: &CompositeCommitIdentity) -> bool {
        ManagedDerivedViewState::warm_at(self, before)
    }

    fn prepare_transition(
        &self,
        before: &CompositeCommitIdentity,
        changes: &[ViewChange],
        maximum_work_units: usize,
    ) -> Option<(Box<dyn std::any::Any + Send>, usize)> {
        let prepared =
            ManagedDerivedViewState::prepare_transition(self, before, changes, maximum_work_units)?;
        let work = prepared.work_units;
        Some((Box::new(prepared), work))
    }

    fn apply_transition(
        &self,
        prepared: Option<Box<dyn std::any::Any + Send>>,
        after: &CompositeCommitIdentity,
    ) {
        match prepared.and_then(|plan| plan.downcast::<PreparedViewTransition<Key>>().ok()) {
            Some(plan) => ManagedDerivedViewState::apply_transition(self, *plan, after.clone()),
            None => self.advance_cold(after.clone()),
        }
    }
}
