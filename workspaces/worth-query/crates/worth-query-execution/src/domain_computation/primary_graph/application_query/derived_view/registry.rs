use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

const MAXIMUM_REGISTERED_VIEWS: usize = 64;
mod index_directory;
mod managed_index;
mod managed_source;
mod native_count;
mod predicate;
pub(in crate::domain_computation::primary_graph) use predicate::PredicateNoticeDisposition;
pub(in crate::domain_computation::primary_graph::application_query) use predicate::{
    mark_prepared_predicates_unavailable, route_prepared_predicates,
};
pub(in crate::domain_computation::primary_graph) use predicate::{
    PredicatePreparationFactory, PredicatePreparationOwner, PredicatePreparationRetirement,
    PredicateWorld, PreparedPredicateNotices, PreparedPredicatePublication, SourcePredicateAxis,
};
pub(in crate::domain_computation::primary_graph::application_query) use predicate::{
    PredicateRegistration, PredicateRootReservation,
};

pub(in crate::domain_computation::primary_graph::application_query) trait RegisteredManagedIndex:
    Send + Sync
{
    fn dispose(&self);
}

pub(in crate::domain_computation::primary_graph::application_query) trait RegisteredDerivedView:
    Send + Sync
{
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

    pub(in crate::domain_computation::primary_graph) fn apply_cold(
        self,
        after: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        for (view, _) in self.views {
            view.apply_transition(None, after);
        }
    }
}

type NativeCountState = Arc<
    super::retention::ManagedDerivedViewState<
        worth_relational::facade::identity::EntityId,
        native_count::ClosedCount,
    >,
>;

#[derive(Default)]
struct RegistryState {
    source_branches:
        BTreeMap<worth_relational::facade::history::BranchId, managed_source::SourceBranchSlot>,
    next_id: u64,
    views: BTreeMap<u64, Weak<dyn RegisteredDerivedView>>,
    native_counts: Vec<(u64, NativeCountState)>,
    indexes: index_directory::ManagedIndexDirectory,
}

impl RegistryState {
    fn registration_count(&self) -> usize {
        self.views.len() + self.indexes.len()
    }
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ManagedDerivedViewRegistry {
    state: Mutex<RegistryState>,
    native_runtime: Option<u64>,
    sources: super::super::managed_index::ManagedSourceInstallation,
    initialization: Mutex<Weak<managed_source::initialization::InitializationSession>>,
}

impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn with_sources(
        sources: super::super::managed_index::ManagedSourceInstallation,
        owner: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> Self {
        Self {
            state: Mutex::new(RegistryState::default()),
            native_runtime: Some(owner.runtime_instance_id()),
            sources,
            initialization: Mutex::new(Weak::new()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn has_live_views(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .views
            .values()
            .any(|view| view.strong_count() > 0)
    }

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
        let result = (|| {
            if state.registration_count() >= MAXIMUM_REGISTERED_VIEWS {
                return None;
            }
            let id = state.next_id.checked_add(1)?;
            state.next_id = id;
            state.views.insert(id, Arc::downgrade(view));
            Some(id)
        })();
        let empty = state
            .views
            .is_empty()
            .then(|| std::mem::take(&mut state.views));
        drop(state);
        drop(empty);
        result
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn unregister(
        &self,
        id: u64,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.views.remove(&id);
        let removed_index = state.indexes.remove(id);
        let removed_count = if removed_index.is_none() {
            state
                .native_counts
                .iter()
                .position(|(actual, _)| *actual == id)
                .map(|position| state.native_counts.remove(position))
        } else {
            None
        };
        let empty = state
            .views
            .is_empty()
            .then(|| std::mem::take(&mut state.views));
        drop(state);
        drop(removed_index);
        drop(removed_count);
        drop(empty);
    }

    pub(in crate::domain_computation::primary_graph) fn dispose(&self) {
        let (views, native_counts, indexes, source_branches) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (
                std::mem::take(&mut state.views),
                std::mem::take(&mut state.native_counts),
                std::mem::take(&mut state.indexes),
                std::mem::take(&mut state.source_branches),
            )
        };
        // Move the real containers out. No upgrade, disposal, final owner drop
        // or scratch collection allocation runs while holding the registry.
        for (_, view) in views {
            if let Some(view) = view.upgrade() {
                view.dispose();
            }
        }
        indexes.dispose();
        self.sources.dispose_predicates();
        drop(native_counts);
        drop(source_branches);
    }
}

impl Drop for ManagedDerivedViewRegistry {
    fn drop(&mut self) {
        self.dispose();
    }
}
