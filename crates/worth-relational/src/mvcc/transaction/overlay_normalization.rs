//! Fallible backing replacement precedes mutation of the retained input intents.
//! Interner/key strings and snapshot-entry Vecs remain existing System owners.
use super::{
    footprint_client_keys::{normalize_created_entity, normalize_created_relation},
    index_row::{IndexKey, IndexRow},
    staging_storage::OrderedStore,
    DetachedRelationalTransactionOverlay, RelationalTransactionFootprint,
    RelationalTransactionStagingDenial as Denial,
};
use crate::symbols::data::{ClientKeySymbolPolicy, StringInterner, Symbol};
use std::collections::BTreeSet;

impl DetachedRelationalTransactionOverlay {
    /// New symbol entries must still reach the symbol-table snapshot on backing
    /// refusal, because the existing interner has already admitted those symbols.
    pub(crate) fn normalize_client_keys(
        &mut self,
        footprint: &mut RelationalTransactionFootprint,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> (Vec<(Symbol, String)>, Result<(), Denial>) {
        if let Err(denial) = allocation_policy.check_live() {
            return (Vec::new(), Err(denial.into()));
        }
        if !policy.interns_requested_strings() {
            return (Vec::new(), Ok(()));
        }
        let mut raw_values = BTreeSet::new();
        for intent in self.batches.iter().flat_map(|batch| &batch.intents) {
            if let Err(denial) = intent.collect_raw_client_keys(&mut raw_values, allocation_policy)
            {
                return (Vec::new(), Err(denial.into()));
            }
        }
        if let Err(denial) = footprint.collect_raw_client_keys(&mut raw_values, allocation_policy) {
            return (Vec::new(), Err(denial));
        }
        // Existing IDs and already-interned created keys require no rewrite.
        // Preserve their exact admitted index/footprint backings and generation.
        if raw_values.is_empty() {
            return (
                Vec::new(),
                allocation_policy.check_live().map_err(Denial::from),
            );
        }
        let Some(generation) = self.normalization_generation.checked_add(1) else {
            return (Vec::new(), Err(Denial::CardinalityOverflow));
        };
        let mut normalized = self.normalized_client_keys.clone();
        let mut entries = Vec::new();
        for raw in raw_values {
            if let Err(denial) = allocation_policy.check_live() {
                return (entries, Err(denial.into()));
            }
            let existed = interner.contains(&raw);
            let symbol = interner.intern(&raw);
            normalized.insert(raw.clone(), symbol);
            if !existed {
                entries.push((symbol, raw));
            }
        }
        let replacement = (|| {
            let footprint =
                footprint.normalized_created_loci(interner, policy, allocation_policy)?;
            let index = OrderedStore::default().with_inserted(
                self.index.iter().map(|row| {
                    let key = match &row.key {
                        IndexKey::CreatedEntity(key) => IndexKey::CreatedEntity(
                            normalize_created_entity(key.clone(), interner, policy),
                        ),
                        IndexKey::CreatedRelation(key) => IndexKey::CreatedRelation(
                            normalize_created_relation(key.clone(), interner, policy),
                        ),
                        key => key.clone(),
                    };
                    IndexRow {
                        key,
                        location: row.location,
                        ordinal: row.ordinal,
                        observes: row.observes,
                    }
                }),
                allocation_policy,
            )?;
            allocation_policy.check_live()?;
            Ok::<_, Denial>((index, footprint))
        })();
        match replacement {
            Err(denial) => (entries, Err(denial)),
            Ok((index, new_footprint)) => {
                for intent in self.batches.iter_mut().flat_map(|batch| &mut batch.intents) {
                    intent.normalize_client_keys(interner, policy);
                }
                self.index = index;
                *footprint = new_footprint;
                self.normalized_client_keys = normalized;
                self.normalization_generation = generation;
                (entries, Ok(()))
            }
        }
    }
}

#[cfg(test)]
#[path = "overlay_normalization/no_raw_keys.rs"]
mod no_raw_keys;

#[cfg(test)]
#[path = "overlay_normalization/selected_policy.rs"]
mod selected_policy;
