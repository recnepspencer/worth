//! The installed Source roster carries its original compilation funding.
use super::SourcePreparationGroup;
use std::{ops::Deref, sync::Arc};
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct CustomInvariantCompilationFunding {
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for CustomInvariantCompilationFunding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CustomInvariantCompilationFunding")
    }
}
#[derive(Debug, Default)]
pub(crate) struct SourcePreparationInventory {
    groups: Vec<SourcePreparationGroup>,
    _funding: Option<Arc<CustomInvariantCompilationFunding>>,
}
impl SourcePreparationInventory {
    pub(super) fn new(
        groups: Vec<SourcePreparationGroup>,
        funding: Option<Arc<CustomInvariantCompilationFunding>>,
    ) -> Self {
        Self {
            groups,
            _funding: funding,
        }
    }
}
impl Deref for SourcePreparationInventory {
    type Target = [SourcePreparationGroup];
    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}
impl PartialEq for SourcePreparationInventory {
    fn eq(&self, other: &Self) -> bool {
        self.groups == other.groups
    }
}
impl Eq for SourcePreparationInventory {}

#[cfg(test)]
mod tests {
    use super::super::{
        compilation_quote::CustomInvariantCompilationQuote, FrozenCustomInvariantRegistry,
    };
    use super::*;
    use crate::schema::{data::RelationalSchemaRegistry, SchemaCompilationControl};
    use worth_execution::SerialMemoryBudget;

    #[test]
    fn source_inventory_and_history_shares_keep_the_original_compiler_charge_until_last_release() {
        let registry = RelationalSchemaRegistry::default();
        let control = SchemaCompilationControl::Configuration;
        let registrations = Vec::new();
        let public = Vec::new();
        let quote =
            CustomInvariantCompilationQuote::measure(&registrations, &public, control).unwrap();
        let memory = SerialMemoryBudget::new(None);
        let funding = Arc::new(CustomInvariantCompilationFunding {
            _memory: Some(memory.reserve(quote.retained as u64).unwrap()),
        });
        let temporary = memory.reserve(quote.temporary as u64).unwrap();
        // This invokes the real shared owned compiler, without a request,
        // runtime, Source issuer, publication or replacement execution policy.
        let frozen = FrozenCustomInvariantRegistry::compile_owned(
            &registry,
            registrations,
            public,
            quote,
            Some(funding),
            control,
        )
        .unwrap();
        drop(temporary);
        let charged = memory.charged_memory_bytes();
        assert!(charged > 0);
        let inventory = frozen.source_inventory();
        let history = frozen.source_history.clone();
        drop(frozen);
        assert_eq!(memory.charged_memory_bytes(), charged);
        assert!(inventory.is_empty());
        assert_eq!(history.len(), 0);
        drop(inventory);
        assert_eq!(memory.charged_memory_bytes(), charged);
        drop(history);
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
}
