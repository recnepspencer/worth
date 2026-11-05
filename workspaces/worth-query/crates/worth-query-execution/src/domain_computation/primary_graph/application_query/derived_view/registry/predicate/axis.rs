//! One existing Native Source service owns point-routed World/scope roots.
mod registration;
use super::PredicateWorld;
use {crate::domain_computation::primary_graph::application_query::managed_index::predicate::PredicateFactory, crate::domain_computation::primary_graph::application_query::managed_index::predicate::PredicateRoot, crate::domain_computation::primary_graph::application_query::managed_index::predicate::RetainedPredicateProbe, crate::domain_computation::primary_graph::vocabulary::application_query::tree::WatchSlot, crate::domain_computation::primary_graph::application_query::managed_index::SourceImageView, crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexSourceChange, crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop, crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Denial, crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion, crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation};
pub(in crate::domain_computation::primary_graph::application_query) use registration::{
    PredicateRegistration, PredicateRootReservation,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent as Parent;
use worth_relational::facade::{
    identity::EntityId, transactions::RelationalExecutionDenialCause as Cause,
};
type Key = (PredicateWorld, EntityId);
struct RootEntry {
    root: Arc<dyn PredicateRoot>,
    pending: usize,
    allocation: Arc<SourceFactsAllocation>,
}
#[derive(Default)]
struct State {
    roots: BTreeMap<Key, RootEntry>,
    closed: bool,
}
pub(in crate::domain_computation::primary_graph) struct SourcePredicateAxis {
    factory: Arc<dyn PredicateFactory>,
    state: Mutex<State>,
    allocation: Arc<SourceFactsAllocation>,
}
struct Retired {
    _empty_backing: Option<BTreeMap<Key, RootEntry>>,
    _entry: RootEntry,
}
impl SourcePredicateAxis {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        execution: &ExecutionRequest<'_, '_>,
        factory: Arc<dyn PredicateFactory>,
    ) -> Result<Arc<Self>, Cause> {
        let allocation = SourceFactsAllocation::open(execution)?;
        allocation.claim_arc::<Self>(execution)?;
        Ok(Arc::new(Self {
            factory,
            state: Mutex::new(State::default()),
            allocation,
        }))
    }
    fn lookup_work(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: &Key,
        count: usize,
    ) -> Result<(), Denial> {
        let levels = u64::from(
            usize::BITS
                - count
                    .checked_add(1)
                    .ok_or(Stop::CapacityOverflow)?
                    .leading_zeros(),
        );
        let width = key
            .0
            .name_bytes()
            .checked_add(size_of::<Key>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Stop::CapacityOverflow)?;
        let work = levels
            .checked_mul(11)
            .and_then(|n| n.checked_mul(width))
            .ok_or(Stop::CapacityOverflow)?;
        MaterialCompletion::new(execution, &self.allocation)
            .work(work)
            .map_err(storage)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn reserve(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        world: PredicateWorld,
        scope: EntityId,
        parent: &Parent,
    ) -> Result<PredicateRootReservation, Denial> {
        if !self.factory.admits(parent) {
            return Err(Stop::MembershipUnavailable.into());
        }
        let key = (world, scope);
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.lookup_work(execution, &key, state.roots.len())?;
            if state.closed {
                return Err(Stop::MembershipUnavailable.into());
            }
            if let Some(entry) = state.roots.get_mut(&key) {
                entry.pending = entry.pending.checked_add(1).ok_or(Stop::CapacityOverflow)?;
                return Ok(PredicateRootReservation::of(self, key, entry));
            }
        }
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        let bytes = size_of::<(Key, RootEntry)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(key.0.name_bytes() + 2 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Stop::CapacityOverflow)?;
        allocation.claim(execution, bytes).map_err(storage)?;
        let mut candidate = Some(RootEntry {
            root: self.factory.open(execution)?,
            pending: 1,
            allocation,
        });
        let result = (|| {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.lookup_work(execution, &key, state.roots.len())?;
            if state.closed {
                return Err(Stop::MembershipUnavailable.into());
            }
            if let Some(entry) = state.roots.get_mut(&key) {
                entry.pending = entry.pending.checked_add(1).ok_or(Stop::CapacityOverflow)?;
            } else {
                state.roots.insert(
                    key.clone(),
                    candidate.take().expect("prepared missing root"),
                );
            }
            let entry = state.roots.get(&key).expect("reserved actual root");
            Ok(PredicateRootReservation::of(self, key, entry))
        })();
        drop(candidate); // Unused provider/root/grants retire outside the axis gate.
        result
    }
    fn release(&self, key: &Key, root: &Arc<dyn PredicateRoot>, opening: bool) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(entry) = state
                .roots
                .get_mut(key)
                .filter(|entry| Arc::ptr_eq(&entry.root, root))
            else {
                return;
            };
            if opening {
                entry.pending = entry
                    .pending
                    .checked_sub(1)
                    .expect("move-only opening reservation owns one pending count");
            }
            // Final live-count check is protected by axis→predicate lock order.
            // Registration reserves under this gate before inserting outside it.
            let empty = entry.pending == 0 && entry.root.len() == 0;
            if empty {
                let entry = state
                    .roots
                    .remove(key)
                    .expect("actual empty root remains owned");
                Some(Retired {
                    _empty_backing: state
                        .roots
                        .is_empty()
                        .then(|| std::mem::take(&mut state.roots)),
                    _entry: entry,
                })
            } else {
                None
            }
        };
        drop(retired);
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn route(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        world: &PredicateWorld,
        scope: EntityId,
        before: SourceImageView<'_>,
        after: SourceImageView<'_>,
        changes: &[WorthQueryManagedIndexSourceChange],
        receive: &mut dyn FnMut(WatchSlot, &crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl<'_>) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        if let Some(root) = self.find_root(execution, world, scope)? {
            root.route(execution, before, after, changes, receive)?;
        }
        Ok(())
    }
    fn find_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        world: &PredicateWorld,
        scope: EntityId,
    ) -> Result<Option<Arc<dyn PredicateRoot>>, Denial> {
        let key = (world.clone(), scope);
        let root = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.lookup_work(execution, &key, state.roots.len())?;
            state.roots.get(&key).map(|entry| entry.root.clone())
        };
        Ok(root)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn invalidate_scope(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        world: &PredicateWorld,
        scope: EntityId,
        budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
        receive: &mut dyn FnMut(WatchSlot, &crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl<'_>) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        if let Some(root) = self.find_root(execution, world, scope)? {
            root.invalidate_scope(execution, budget, receive)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn dispose(&self) {
        let mut retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.closed = true;
            std::mem::take(&mut state.roots)
        };
        while let Some((_, entry)) = retired.pop_first() {
            if retired.is_empty() {
                drop(std::mem::take(&mut retired));
            }
            entry.root.dispose();
            drop(entry);
        }
    }
}
impl Drop for SourcePredicateAxis {
    fn drop(&mut self) {
        self.dispose();
    }
}
fn storage(cause: Cause) -> Denial {
    Stop::MaterialStorage(cause).into()
}
