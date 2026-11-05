//! One registry owner; production supplies its original controlled comparison extent.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_query::source_identity::WorthQueryObservedSourceCoordinate;
use crate::domain_computation::primary_graph::vocabulary::application_query::source_identity::WorthQueryObservedSourceOccurrence;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl WorthQueryObservedSourceMeaningRegistry {
    pub(in crate::domain_computation::primary_graph) fn intern_compared(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        self.intern_with_storage(query, parameters, footprint, selection, equal, None)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn intern_material_compared(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        self.intern_with_storage(
            query,
            parameters,
            footprint,
            selection,
            equal,
            Some(material_storage::ManagedInternerStorage {
                execution,
                allocation,
            }),
        )
    }

    fn intern_with_storage(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
        storage: Option<material_storage::ManagedInternerStorage<'_, '_, '_>>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        let occurrence = match selection {
            WorthQueryApplicationBasisSelectionIdentity::Relational => {
                WorthQueryObservedSourceOccurrence::Relational
            }
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                WorthQueryObservedSourceOccurrence::Product(product.lifecycle_incarnation())
            }
        };
        let coordinate = WorthQueryObservedSourceCoordinate {
            query: *query,
            parameters: *parameters,
            occurrence,
            root: footprint.root,
        };
        // Both incoming and upgraded custody outlive the guard, including unwinding.
        let mut incoming = Some(footprint);
        let mut unmatched = Vec::new();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let result = (|| {
            use std::collections::btree_map::Entry;
            let WorthQueryObservedSourceMeaningRegistryState {
                next_ordinal,
                meanings,
            } = &mut *state;
            let count = meanings.len();
            if let Some(storage) = &storage {
                storage.lookup(count)?;
            }
            let mut entry = meanings.entry(coordinate.clone());
            if let Some(storage) = &storage {
                let size = match &entry {
                    Entry::Occupied(entry) => entry.get().entries.len(),
                    Entry::Vacant(_) => 0,
                };
                unmatched = storage.scratch(size)?;
            }
            if let Entry::Occupied(entry) = &mut entry {
                let meanings = entry.get_mut();
                meanings
                    .entries
                    .retain(|meaning| meaning.strong_count() > 0);
                for existing in meanings.entries.iter().filter_map(Weak::upgrade) {
                    unmatched.push(existing);
                    let existing = unmatched.last().expect("retained comparator operand");
                    let footprint = incoming.as_ref().expect("incoming until admitted");
                    if let Some(storage) = &storage {
                        storage.comparison(&existing.footprint, footprint)?;
                    }
                    if equal(&existing.footprint, footprint)? {
                        return Ok(Some(Arc::clone(existing)));
                    }
                }
            }
            let ordinal = *next_ordinal;
            let Some(successor) = ordinal.checked_add(1) else {
                return Ok(None);
            };
            let mut vacant = material_storage::CoordinateMeanings::default();
            if let Some(storage) = &storage {
                storage.meaning()?;
                let bucket = match &mut entry {
                    Entry::Occupied(entry) => entry.get_mut(),
                    Entry::Vacant(_) => &mut vacant,
                };
                bucket.reserve(storage, count)?;
            }
            let footprint = incoming.as_ref().expect("incoming until admitted");
            let checkpoint_identity = if let Some(storage) = &storage {
                checkpoint_commitment::checkpoint_source_identity_admitted(
                    query,
                    parameters,
                    footprint,
                    &mut |bytes| storage.work(bytes),
                )?
            } else {
                checkpoint_commitment::checkpoint_source_identity(query, parameters, footprint)
            };
            let meaning = Arc::new(WorthQueryObservedSourceMeaning {
                coordinate: coordinate.clone(),
                footprint: incoming.take().expect("successful incoming admission"),
                identity: source_identity(self.runtime_authority, ordinal),
                runtime_idempotency_identity: stable_runtime_identity(
                    self.runtime_authority,
                    &coordinate,
                    &checkpoint_identity,
                ),
                checkpoint_identity,
                registry: Arc::downgrade(&self.state),
                _material_allocation: storage
                    .as_ref()
                    .map(|storage| Arc::clone(storage.allocation)),
            });
            *next_ordinal = successor;
            let bucket = match entry {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(vacant),
            };
            bucket.entries.push(Arc::downgrade(&meaning));
            Ok(Some(meaning))
        })();
        // Also covers failed pre-insertion paths: an empty map cannot strand a
        // newly admitted container ticket. Drop actual backing before its grants.
        let retired = state.take_empty_material_storage();
        drop(state);
        drop(unmatched);
        drop(retired);
        result
    }

    /// Set membership is a distinct source contract from any returned row.
    pub(in crate::domain_computation::primary_graph) fn intern_result_set_compared(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        self.intern_compared(
            &result_set_query(query),
            parameters,
            footprint,
            selection,
            equal,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn intern_material_result_set_compared(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        self.intern_material_compared(
            &result_set_query(query),
            parameters,
            footprint,
            selection,
            equal,
            execution,
            allocation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn intern(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        self.intern_compared(query, parameters, footprint, selection, &mut |a, b| {
            Ok(a == b)
        })
        .expect("ordinary equality comparison cannot refuse")
    }

    pub(in crate::domain_computation::primary_graph) fn intern_result_set(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        self.intern_result_set_compared(query, parameters, footprint, selection, &mut |a, b| {
            Ok(a == b)
        })
        .expect("ordinary equality comparison cannot refuse")
    }
}

fn result_set_query(query: &[u8; 32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth-query:observed-result-set:v1");
    digest.update(query);
    digest.finalize().into()
}

mod court_doors_for_test;
