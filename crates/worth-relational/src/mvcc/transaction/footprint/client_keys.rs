//! Harvest and rebuild created-key loci while retaining the original typed backing.
use super::super::footprint_client_keys::{
    collect_created_entity_raw_key, collect_created_relation_raw_keys, normalize_read_locus,
    normalize_write_locus,
};
use super::{
    Denial, OrderedStore, RelationalTransactionFootprint, RelationalTransactionReadLocus,
    RelationalTransactionWriteLocus,
};
use std::collections::BTreeSet;

impl RelationalTransactionFootprint {
    pub(crate) fn collect_raw_client_keys(
        &self,
        raw_values: &mut BTreeSet<String>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        allocation_policy.check_live()?;
        for read in &self.reads {
            allocation_policy.check_live()?;
            match read {
                RelationalTransactionReadLocus::CreatedEntity(created) => {
                    collect_created_entity_raw_key(created, raw_values);
                }
                RelationalTransactionReadLocus::CreatedRelation(created) => {
                    collect_created_relation_raw_keys(created, raw_values);
                }
                RelationalTransactionReadLocus::Existing(_)
                | RelationalTransactionReadLocus::ValidationPartition(_)
                | RelationalTransactionReadLocus::EntitySchema(_)
                | RelationalTransactionReadLocus::RelationSchema(_) => {}
            }
        }
        for write in &self.writes {
            allocation_policy.check_live()?;
            match write {
                RelationalTransactionWriteLocus::CreatedEntity(created) => {
                    collect_created_entity_raw_key(created, raw_values);
                }
                RelationalTransactionWriteLocus::CreatedRelation(created) => {
                    collect_created_relation_raw_keys(created, raw_values);
                }
                RelationalTransactionWriteLocus::Existing(_) => {}
            }
        }
        Ok(())
    }

    pub(crate) fn normalized_created_loci(
        &self,
        interner: &mut crate::symbols::data::StringInterner,
        policy: crate::symbols::data::ClientKeySymbolPolicy,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, Denial> {
        let reads = OrderedStore::default().with_inserted(
            self.reads
                .iter()
                .cloned()
                .map(|read| normalize_read_locus(read, interner, policy)),
            allocation_policy,
        )?;
        let writes = OrderedStore::default().with_inserted(
            self.writes
                .iter()
                .cloned()
                .map(|write| normalize_write_locus(write, interner, policy)),
            allocation_policy,
        )?;
        Ok(Self {
            basis: self.basis.clone(),
            reads,
            writes,
            write_partitions: self.write_partitions.clone(),
        })
    }
}
