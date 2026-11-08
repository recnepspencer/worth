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
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct HistoryDiscoveryRecipeKey {
    connection: std::any::TypeId,
    tree: (std::any::TypeId, std::any::TypeId, std::any::TypeId),
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
            connection: std::any::TypeId::of::<C>(),
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
    pub(super) entries: BTreeMap<HistoryDiscoveryRecipeKey, Slot>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::domain_computation::primary_graph) struct HistoryDiscoveryRecipeRecorder {
    pub(super) step: WorthQueryHistoryNodeId,
    pub(super) model: HistoryModelIdentity,
    state: Mutex<RecorderState>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
enum RecorderState {
    Recording(BTreeMap<HistoryDiscoveryRecipeKey, Slot>),
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
            state: Mutex::new(RecorderState::Recording(BTreeMap::new())),
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
        if entries.contains_key(&key) {
            return Err(ReadDenial::Authority(Denial::Busy));
        }
        let (allocation, _) = HistoryInsertionAllocation::reserve::<(
            HistoryDiscoveryRecipeKey,
            Slot,
        )>(execution, entries.len(), 0)
        .map_err(ReadDenial::Preparation)?;
        entries.insert(
            key,
            Slot {
                entry,
                _allocation: allocation,
            },
        );
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
