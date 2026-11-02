//! Live initial-installation evidence includes the complete Source roster.
use crate::validation::candidate_source::{ManagedIndexSourceProgram, SourcePreparationGroup};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalInitialSchemaInstallationReceipt {
    pub(super) source_groups: Arc<[SourcePreparationGroup]>,
    pub(super) runtime_instance_id: u64,
    pub(super) custom_invariant_generation: u64,
    pub(super) custom_invariant_inventory_digest: [u8; 32],
    pub(super) retained_entity_kind_count: usize,
    pub(super) retained_relation_kind_count: usize,
    pub(super) retained_schema_authority_digest: [u8; 32],
}

impl RelationalInitialSchemaInstallationReceipt {
    /// Descriptive correspondence with the exact installed program owners.
    /// This grants neither Source READ nor publication authority.
    pub fn has_exact_managed_index_sources(&self, expected: &[ManagedIndexSourceProgram]) -> bool {
        let programs = || {
            self.source_groups
                .iter()
                .flat_map(SourcePreparationGroup::programs)
        };
        programs().all(|program| expected.contains(program))
            && expected
                .iter()
                .all(|program| programs().any(|actual| actual == program))
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
    pub const fn custom_invariant_generation(&self) -> u64 {
        self.custom_invariant_generation
    }
    pub const fn custom_invariant_inventory_digest(&self) -> &[u8; 32] {
        &self.custom_invariant_inventory_digest
    }
    pub const fn retained_entity_kind_count(&self) -> usize {
        self.retained_entity_kind_count
    }

    pub const fn retained_relation_kind_count(&self) -> usize {
        self.retained_relation_kind_count
    }
}
