//! Invocation/image-confined decoded text; no shared text handle crosses this owner.
use crate::domain_computation::primary_graph::access_strategy::ordered_storage;
use std::{cell::RefCell, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest,
    ExecutionRequestArrayBuilder,
};
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey as Key;
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding as Binding;
use {
    crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot as Slot,
    crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

#[derive(Default)]
pub(super) struct SourceText {
    bindings: RefCell<ExecutionOrderedMap<Arc<Key>, Arc<Binding>>>,
    values: RefCell<ExecutionOrderedMap<(Slot, Arc<Key>), Arc<TextBacking>>>,
}

pub(super) struct TextBacking {
    value: ExecutionArray<String>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl TextBacking {
    pub(super) fn as_str(&self) -> &str {
        &self.value[0]
    }
}

impl SourceText {
    pub(super) fn binding(
        &self,
        key: &Arc<Key>,
        key_work: usize,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        read: impl FnOnce() -> Result<Arc<Binding>, Stop>,
    ) -> Result<Arc<Binding>, Stop> {
        let found = {
            let bindings = self.bindings.borrow();
            ordered_storage::lookup(control, bindings.len(), key_work)?;
            bindings.get(key).cloned()
        };
        if let Some(found) = found {
            return Ok(found);
        }
        let binding = read()?;
        let mut bindings = self.bindings.borrow_mut();
        for _ in 0..4 {
            ordered_storage::lookup(control, bindings.len(), key_work)?;
        }
        ordered_storage::insert(&mut bindings, key, &binding, execution, control)?;
        Ok(binding)
    }

    pub(super) fn text(
        &self,
        slot: Slot,
        key: &Arc<Key>,
        key_work: usize,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        read: impl FnOnce() -> Result<String, Stop>,
    ) -> Result<Arc<TextBacking>, Stop> {
        let key = (slot, Arc::clone(key));
        let found = {
            let values = self.values.borrow();
            ordered_storage::lookup(control, values.len(), key_work)?;
            values.get(&key).cloned()
        };
        if let Some(found) = found {
            return Ok(found);
        }
        // Fund inline String/backing before invoking the registered codec. Its
        // separately retained parse grant lives in the original decode state.
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<TextBacking>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| control.refuse(Stop::Request(cause)))?;
        let mut value = ExecutionRequestArrayBuilder::allocate(*execution, 1)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        value
            .push(read()?)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let value = value
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let value = Arc::new(TextBacking {
            value,
            _memory: memory,
        });
        let mut values = self.values.borrow_mut();
        for _ in 0..4 {
            ordered_storage::lookup(control, values.len(), key_work)?;
        }
        ordered_storage::insert(&mut values, &key, &value, execution, control)?;
        Ok(value)
    }
}
