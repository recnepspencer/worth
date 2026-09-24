use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

const MAXIMUM_REGISTERED_VIEWS: usize = 64;

pub(super) trait RegisteredDerivedView: Send + Sync {
    fn dispose(&self);
    fn belongs_to(&self, basis: &super::publication::ViewPublicationBasis<'_>) -> bool;
    fn warm_at(&self, before: &worth_runtime_world::facade::CompositeCommitIdentity) -> bool;
    fn prepare_transition(
        &self,
        before: &worth_runtime_world::facade::CompositeCommitIdentity,
        changes: &[super::publication::ViewChange],
        maximum_work_units: usize,
    ) -> Option<(Box<dyn Any + Send>, usize)>;
    fn apply_transition(
        &self,
        prepared: Option<Box<dyn Any + Send>>,
        after: &worth_runtime_world::facade::CompositeCommitIdentity,
    );
}

pub(in crate::domain_computation::primary_graph) struct PreparedManagedViewPublication {
    views: Vec<(Arc<dyn RegisteredDerivedView>, Option<Box<dyn Any + Send>>)>,
}

impl PreparedManagedViewPublication {
    pub(in crate::domain_computation::primary_graph) fn apply(
        self,
        after: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        for (view, prepared) in self.views {
            view.apply_transition(prepared, after);
        }
    }
}

#[derive(Default)]
struct RegistryState {
    next_id: u64,
    views: BTreeMap<u64, Weak<dyn RegisteredDerivedView>>,
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ManagedDerivedViewRegistry {
    state: Mutex<RegistryState>,
}

impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn prepare_publication(
        &self,
        basis: super::publication::ViewPublicationBasis<'_>,
        changes: &[super::publication::ViewChange],
        maximum_work_units: usize,
    ) -> PreparedManagedViewPublication {
        let views = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .views
            .values()
            .filter_map(Weak::upgrade)
            .filter(|view| view.belongs_to(&basis))
            .collect::<Vec<_>>();
        let mut remaining = maximum_work_units;
        let mut prepared = Vec::with_capacity(views.len());
        for view in views {
            let plan = if remaining == 0 {
                None
            } else if !view.warm_at(basis.before) {
                remaining -= 1;
                None
            } else {
                let plan = view.prepare_transition(basis.before, changes, remaining - 1);
                if plan.is_none() {
                    remaining = 0;
                }
                plan
            };
            let plan = plan.map(|(plan, work)| {
                remaining = remaining.saturating_sub(work.saturating_add(1));
                plan
            });
            prepared.push((view, plan));
        }
        PreparedManagedViewPublication { views: prepared }
    }

    pub(super) fn register(self: &Arc<Self>, view: &Arc<dyn RegisteredDerivedView>) -> Option<u64> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.views.retain(|_, view| view.strong_count() > 0);
        if state.views.len() >= MAXIMUM_REGISTERED_VIEWS {
            return None;
        }
        let id = state.next_id.checked_add(1)?;
        state.next_id = id;
        state.views.insert(id, Arc::downgrade(view));
        Some(id)
    }

    pub(super) fn unregister(&self, id: u64) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .views
            .remove(&id);
    }

    pub(in crate::domain_computation::primary_graph) fn dispose(&self) {
        let views = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut state.views)
                .into_values()
                .filter_map(|view| view.upgrade())
                .collect::<Vec<_>>()
        };
        for view in views {
            view.dispose();
        }
    }
}

impl Drop for ManagedDerivedViewRegistry {
    fn drop(&mut self) {
        self.dispose();
    }
}
