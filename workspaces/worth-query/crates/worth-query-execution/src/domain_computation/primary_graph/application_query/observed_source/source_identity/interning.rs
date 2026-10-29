//! One registry owner; production supplies its original controlled comparison extent.
use super::*;

impl WorthQueryObservedSourceMeaningRegistry {
    pub(in crate::domain_computation::primary_graph) fn intern_compared<E>(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, E>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, E> {
        self.intern_with_storage(
            query,
            parameters,
            footprint,
            selection,
            equal,
            None,
            &mut |_| unreachable!("ordinary interning has no material allocation"),
        )
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn intern_material_compared<
        E,
    >(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, E>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
        failure: &mut impl FnMut(
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        ) -> E,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, E> {
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
            failure,
        )
    }

    fn intern_with_storage<E>(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, E>,
        storage: Option<material_storage::ManagedInternerStorage<'_, '_, '_>>,
        failure: &mut impl FnMut(
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        ) -> E,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, E> {
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
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Every upgraded Arc, including the current comparator operand, survives
        // the guard. An early typed refusal must not run its reentrant Drop here.
        let mut unmatched = Vec::new();
        let result = (|| {
            if let Some(storage) = &storage {
                storage
                    .lookup(state.meanings.len())
                    .map_err(&mut *failure)?;
                unmatched = storage
                    .scratch(state.meanings.get(&coordinate).map_or(0, Vec::len))
                    .map_err(&mut *failure)?;
                storage
                    .lookup(state.meanings.len())
                    .map_err(&mut *failure)?;
            }
            if let Some(meanings) = state.meanings.get_mut(&coordinate) {
                meanings.retain(|meaning| meaning.strong_count() > 0);
                for existing in meanings.iter().filter_map(Weak::upgrade) {
                    unmatched.push(existing);
                    let existing = unmatched.last().expect("just retained comparator operand");
                    if let Some(storage) = &storage {
                        storage
                            .comparison(&existing.footprint, &footprint)
                            .map_err(&mut *failure)?;
                    }
                    if equal(&existing.footprint, &footprint)? {
                        return Ok(Some(Arc::clone(existing)));
                    }
                }
            }
            let ordinal = state.next_ordinal;
            let Some(next_ordinal) = ordinal.checked_add(1) else {
                return Ok(None);
            };
            let prepared = if let Some(storage) = &storage {
                storage.meaning().map_err(&mut *failure)?;
                let prepared = storage
                    .prepare_container(&mut state, &coordinate)
                    .map_err(&mut *failure)?;
                storage
                    .lookup(state.meanings.len())
                    .map_err(&mut *failure)?;
                prepared
            } else {
                None
            };
            let checkpoint_identity = if let Some(storage) = &storage {
                checkpoint_commitment::checkpoint_source_identity_admitted(
                    query,
                    parameters,
                    &footprint,
                    &mut |bytes| storage.work(bytes),
                )
                .map_err(&mut *failure)?
            } else {
                checkpoint_commitment::checkpoint_source_identity(query, parameters, &footprint)
            };
            let meaning = Arc::new(WorthQueryObservedSourceMeaning {
                coordinate: coordinate.clone(),
                footprint,
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
            state.next_ordinal = next_ordinal;
            state
                .meanings
                .entry(coordinate)
                .or_insert_with(|| prepared.unwrap_or_default())
                .push(Arc::downgrade(&meaning));
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
    pub(in crate::domain_computation::primary_graph) fn intern_result_set_compared<E>(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, E>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, E> {
        self.intern_compared(
            &result_set_query(query),
            parameters,
            footprint,
            selection,
            equal,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn intern_material_result_set_compared<
        E,
    >(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, E>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
        failure: &mut impl FnMut(
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        ) -> E,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, E> {
        self.intern_material_compared(
            &result_set_query(query),
            parameters,
            footprint,
            selection,
            equal,
            execution,
            allocation,
            failure,
        )
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn intern(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        match self.intern_compared(query, parameters, footprint, selection, &mut |a, b| {
            Ok::<bool, std::convert::Infallible>(a == b)
        }) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn intern_result_set(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        match self.intern_result_set_compared(
            query,
            parameters,
            footprint,
            selection,
            &mut |a, b| Ok::<bool, std::convert::Infallible>(a == b),
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
}

fn result_set_query(query: &[u8; 32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth-query:observed-result-set:v1");
    digest.update(query);
    digest.finalize().into()
}
