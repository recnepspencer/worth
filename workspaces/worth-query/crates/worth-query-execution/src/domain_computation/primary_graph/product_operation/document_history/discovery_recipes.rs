//! Per-step canonical discovery inputs share the journal's source binding.
mod enrollment;
use super::allocation::HistoryInsertionAllocation;
use super::{
    HistoryModelIdentity, WorthQueryHistoryNavigationDenial as Denial, WorthQueryHistoryNodeId,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::{
    application_query::request_recipe::CapturedSourceQueryRequest,
    WorthQueryHistoryOutputReadDenial as ReadDenial,
};
use crate::equality_type_id::EqualityTypeId;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct HistoryDiscoveryRecipeKey {
    connection: EqualityTypeId,
    tree: (EqualityTypeId, EqualityTypeId, EqualityTypeId),
    root: WorthQueryOperationScopeEntityBinding,
    root_partition: [u8; 32],
    parent: WorthQueryOperationScopeEntityBinding,
    parent_partition: [u8; 32],
}
impl HistoryDiscoveryRecipeKey {
    pub(in crate::domain_computation::primary_graph) fn new<C: 'static>(
        profile: &super::WorthQueryHistoryOutputProfile,
        parent: (WorthQueryOperationScopeEntityBinding, [u8; 32]),
    ) -> Self {
        Self {
            connection: EqualityTypeId::of::<C>(),
            tree: profile.discovery_recipe_tree_identity(),
            root: WorthQueryOperationScopeEntityBinding::from_entity(
                profile.source_intent().source_subject(),
            ),
            root_partition: profile.source_intent().source_partition(),
            parent: parent.0,
            parent_partition: parent.1,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct DeclaredRecipeKey {
    root: WorthQueryOperationScopeEntityBinding,
    root_partition: [u8; 32],
    parent: WorthQueryOperationScopeEntityBinding,
    parent_partition: [u8; 32],
}
impl From<&HistoryDiscoveryRecipeKey> for DeclaredRecipeKey {
    fn from(key: &HistoryDiscoveryRecipeKey) -> Self {
        Self {
            root: key.root,
            root_partition: key.root_partition,
            parent: key.parent,
            parent_partition: key.parent_partition,
        }
    }
}
/// One declared-order lookup, then equality within that declared key's bucket.
#[derive(Default)]
pub(super) struct HistoryDiscoveryRecipeIndex {
    declared: BTreeMap<DeclaredRecipeKey, Vec<(HistoryDiscoveryRecipeKey, Slot)>>,
    len: usize,
}
impl HistoryDiscoveryRecipeIndex {
    pub(super) fn get(&self, key: &HistoryDiscoveryRecipeKey) -> Option<&Slot> {
        self.declared
            .get(&DeclaredRecipeKey::from(key))?
            .iter()
            .find_map(|(stored, slot)| (stored == key).then_some(slot))
    }
    pub(super) fn contains(&self, key: &HistoryDiscoveryRecipeKey) -> bool {
        self.get(key).is_some()
    }
    pub(super) fn len(&self) -> usize {
        self.len
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = (&HistoryDiscoveryRecipeKey, &Slot)> {
        self.declared
            .values()
            .flat_map(|bucket| bucket.iter().map(|(key, slot)| (key, slot)))
    }
    pub(super) fn values_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.declared
            .values_mut()
            .flat_map(|bucket| bucket.iter_mut().map(|(_, slot)| slot))
    }
    pub(super) fn insert(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        key: HistoryDiscoveryRecipeKey,
        entry: Arc<Entry>,
    ) -> Result<(), Cause> {
        let declared = DeclaredRecipeKey::from(&key);
        let bucket = self.declared.get(&declared);
        let growth = bucket.is_none_or(|bucket| bucket.len() == bucket.capacity());
        // Keep the existing per-slot tree grant and charge the bucket's added element too.
        let additional = if growth {
            std::mem::size_of::<(HistoryDiscoveryRecipeKey, Slot)>()
        } else {
            0
        };
        let (allocation, _) = HistoryInsertionAllocation::reserve::<(
            HistoryDiscoveryRecipeKey,
            Slot,
        )>(execution, self.len(), additional)?;
        let bucket = self.declared.entry(declared).or_default();
        if let Some((_, slot)) = bucket.iter_mut().find(|(stored, _)| stored == &key) {
            *slot = Slot {
                entry,
                _allocation: allocation,
            };
        } else {
            bucket.reserve_exact(1);
            bucket.push((
                key,
                Slot {
                    entry,
                    _allocation: allocation,
                },
            ));
            self.len += 1;
        }
        Ok(())
    }
}
pub(super) struct Entry {
    pub(super) step: WorthQueryHistoryNodeId,
    pub(super) model: HistoryModelIdentity,
    pub(super) key: HistoryDiscoveryRecipeKey,
    pub(super) request: Arc<CapturedSourceQueryRequest>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(super) struct Slot {
    pub(super) entry: Arc<Entry>,
    pub(super) _allocation: HistoryInsertionAllocation,
}
pub(in crate::domain_computation::primary_graph) struct HistoryDiscoveryRecipes {
    pub(super) step: WorthQueryHistoryNodeId,
    pub(super) model: HistoryModelIdentity,
    pub(super) entries: HistoryDiscoveryRecipeIndex,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::domain_computation::primary_graph) struct HistoryDiscoveryRecipeRecorder {
    pub(super) step: WorthQueryHistoryNodeId,
    pub(super) model: HistoryModelIdentity,
    state: Mutex<RecorderState>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
enum RecorderState {
    Recording(HistoryDiscoveryRecipeIndex),
    Sealed(Arc<HistoryDiscoveryRecipes>),
}
impl HistoryDiscoveryRecipeRecorder {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        step: WorthQueryHistoryNodeId,
        model: HistoryModelIdentity,
    ) -> Result<Arc<Self>, Cause> {
        let memory = reserve(
            execution,
            std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>(),
        )?;
        Ok(Arc::new(Self {
            step,
            model,
            state: Mutex::new(RecorderState::Recording(
                HistoryDiscoveryRecipeIndex::default(),
            )),
            _memory: memory,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn record(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: HistoryDiscoveryRecipeKey,
        capture: impl FnOnce() -> Result<Arc<CapturedSourceQueryRequest>, ReadDenial>,
    ) -> Result<Arc<CapturedSourceQueryRequest>, ReadDenial> {
        // No application mapping runs under the recorder lock.
        {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match &*state {
                RecorderState::Recording(entries) => {
                    if let Some(slot) = entries.get(&key) {
                        return Ok(Arc::clone(&slot.entry.request));
                    }
                }
                RecorderState::Sealed(_) => {
                    return Err(ReadDenial::Authority(Denial::DiscoveryRecipeSealed {
                        step: self.step,
                    }))
                }
            }
        }
        let request = capture()?;
        let memory =
            super::allocation::reserve_arc::<Entry>(execution).map_err(ReadDenial::Preparation)?;
        let entry = Arc::new(Entry {
            step: self.step,
            model: self.model.clone(),
            key: key.clone(),
            request: Arc::clone(&request),
            _memory: memory,
        });
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let RecorderState::Recording(entries) = &mut *state else {
            return Err(ReadDenial::Authority(Denial::DiscoveryRecipeSealed {
                step: self.step,
            }));
        };
        if entries.contains(&key) {
            return Err(ReadDenial::Authority(Denial::Busy));
        }
        entries
            .insert(execution, key, entry)
            .map_err(ReadDenial::Preparation)?;
        Ok(request)
    }
    pub(super) fn seal(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<HistoryDiscoveryRecipes>, Cause> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let RecorderState::Sealed(recipes) = &*state {
            return Ok(Arc::clone(recipes));
        }
        let RecorderState::Recording(entries) = &mut *state else {
            unreachable!()
        };
        let memory = super::allocation::reserve_arc::<HistoryDiscoveryRecipes>(execution)?;
        let recipes = Arc::new(HistoryDiscoveryRecipes {
            step: self.step,
            model: self.model.clone(),
            entries: std::mem::take(entries),
            _memory: memory,
        });
        *state = RecorderState::Sealed(Arc::clone(&recipes));
        Ok(recipes)
    }
}
impl HistoryDiscoveryRecipes {
    pub(in crate::domain_computation::primary_graph) fn get(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        step: WorthQueryHistoryNodeId,
        model: &HistoryModelIdentity,
        key: &HistoryDiscoveryRecipeKey,
    ) -> Result<Arc<CapturedSourceQueryRequest>, ReadDenial> {
        if self.step != step || &self.model != model {
            return Err(ReadDenial::Authority(
                Denial::DiscoveryRecipeForeignDocument { step },
            ));
        }
        let entry = &self
            .entries
            .get(key)
            .ok_or(ReadDenial::Authority(Denial::DiscoveryRecipeMissing {
                step,
            }))?
            .entry;
        if entry.step != step
            || &entry.model != model
            || &entry.key != key
            || !entry
                .request
                .validate(execution)
                .map_err(ReadDenial::Preparation)?
        {
            return Err(ReadDenial::Authority(Denial::DiscoveryRecipeCorrupt {
                step,
            }));
        }
        Ok(Arc::clone(&entry.request))
    }
}
pub(super) fn reserve(
    execution: &ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Cause> {
    crate::domain_computation::primary_graph::structural_walk::memory::reserve(
        execution,
        execution.has_accounting(),
        u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?,
    )
}
