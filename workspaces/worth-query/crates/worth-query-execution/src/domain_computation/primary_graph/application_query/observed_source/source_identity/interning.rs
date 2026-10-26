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
        // Keep every upgraded non-match alive until after the registry guard
        // is released. A concurrent final owner may otherwise leave this
        // temporary as the last Arc, whose Drop would re-lock this mutex.
        let mut unmatched_meanings = Vec::new();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(meanings) = state.meanings.get_mut(&coordinate) {
            meanings.retain(|meaning| meaning.strong_count() > 0);
            for existing in meanings.iter().filter_map(Weak::upgrade) {
                let matches = match equal(&existing.footprint, &footprint) {
                    Ok(value) => value,
                    Err(stop) => {
                        drop(state);
                        drop(existing);
                        drop(unmatched_meanings);
                        return Err(stop);
                    }
                };
                if matches {
                    drop(state);
                    drop(unmatched_meanings);
                    return Ok(Some(existing));
                }
                unmatched_meanings.push(existing);
            }
        }
        let ordinal = state.next_ordinal;
        let Some(next_ordinal) = ordinal.checked_add(1) else {
            drop(state);
            drop(unmatched_meanings);
            return Ok(None);
        };
        state.next_ordinal = next_ordinal;
        let checkpoint_identity =
            checkpoint_commitment::checkpoint_source_identity(query, parameters, &footprint);
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
        });
        state
            .meanings
            .entry(coordinate)
            .or_default()
            .push(Arc::downgrade(&meaning));
        drop(state);
        drop(unmatched_meanings);
        Ok(Some(meaning))
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
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth-query:observed-result-set:v1");
        digest.update(query);
        let set_query = digest.finalize().into();
        self.intern_compared(&set_query, parameters, footprint, selection, equal)
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
