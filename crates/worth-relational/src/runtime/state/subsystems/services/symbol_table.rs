mod cold_pages;
mod state;
pub(crate) use cold_pages::ColdSymbolDirectoryDenial;
use state::RuntimeSymbolState;

use std::sync::{Arc, RwLock};

use crate::symbols::data::{StoredSymbolText, StringInterner, Symbol, SymbolTableSnapshot};

#[path = "symbol_table/reservations.rs"]
mod reservations;
pub(crate) use reservations::{
    PendingSymbolMeanings, PreparedSymbolPublication, SymbolReservationDenial,
};

#[derive(Debug)]
pub(crate) struct RuntimeSymbolTable {
    state: Arc<RwLock<RuntimeSymbolState>>,
    cold: Arc<RwLock<Option<Arc<super::symbol_residency::RuntimeSymbolResidency>>>>,
    reservations: Arc<std::sync::Mutex<reservations::SymbolReservationDirectory>>,
}

impl Default for RuntimeSymbolTable {
    fn default() -> Self {
        Self {
            state: Arc::default(),
            cold: Arc::new(RwLock::new(None)),
            reservations: Arc::default(),
        }
    }
}

impl Clone for RuntimeSymbolTable {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            cold: self.cold.clone(),
            reservations: self.reservations.clone(),
        }
    }
}

impl PartialEq for RuntimeSymbolTable {
    fn eq(&self, other: &Self) -> bool {
        let left = self
            .cold
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let right = other
            .cold
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let same_directory = match (left, right) {
            (None, None) => true,
            (Some(left), Some(right)) => left.same_directory(&right),
            _ => false,
        };
        same_directory
            && self.interner_snapshot() == other.interner_snapshot()
            && self.configuration_snapshot() == other.configuration_snapshot()
    }
}

impl Eq for RuntimeSymbolTable {}

impl RuntimeSymbolTable {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let cold = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        let snapshot = || self.read_state(Clone::clone);
        // Page admission locks pages before the owner dictionary. Fork uses
        // that same order, so copied admission marks and text are one snapshot.
        let (cold, state) = match cold {
            Some(pages) => {
                let (pages, snapshot) = pages.detached_with_owner(snapshot);
                (Some(Arc::new(pages)), snapshot)
            }
            None => (None, snapshot()),
        };
        Self {
            state: Arc::new(RwLock::new(state)),
            reservations: Arc::default(),
            cold: Arc::new(RwLock::new(cold)),
        }
    }
    pub(crate) fn normalize_client_keys(
        &self,
        normalize: impl FnOnce(&mut StringInterner) -> Vec<(Symbol, String)>,
    ) {
        self.update(|state| {
            for (symbol, value) in normalize(&mut state.interner) {
                state.configuration.insert(value.into(), symbol);
            }
        });
    }

    pub(crate) fn admit_field_text(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        input: &worth_execution::ExecutionImmutableBytes,
    ) -> Result<(Symbol, StoredSymbolText), crate::symbols::data::RelationalFieldTextAdmissionDenial>
    {
        use crate::symbols::data::RelationalFieldTextAdmissionDenial as Denial;
        let raw = std::str::from_utf8(input.bytes()).map_err(|_| Denial::InvalidUtf8)?;
        self.demand_text(execution, raw)
            .map_err(|failure| match failure {
                crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(
                    failure,
                ) => Denial::StorageUnavailable(failure),
                crate::storage::residency::RelationalResidencyDemandFailure::Request(cause) => {
                    Denial::Request(cause)
                }
            })?;
        self.update(|state| {
            if self.pending_symbol(raw).is_none() {
                state.interner.check_text_capacity(raw)?;
            }
            let text = StoredSymbolText::admit(execution, input)?;
            self.publish_admitted_text(execution, state, text)
        })
    }

    fn publish_admitted_text(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        state: &mut RuntimeSymbolState,
        text: StoredSymbolText,
    ) -> Result<(Symbol, StoredSymbolText), crate::symbols::data::RelationalFieldTextAdmissionDenial>
    {
        use crate::symbols::data::RelationalFieldTextAdmissionDenial as Denial;
        execution.check_live().map_err(Denial::Stopped)?;
        let symbol = self.admit_with_pending_identity(&mut state.interner, text.clone())?;
        state.configuration.remove(text.as_str());
        state.configuration.insert(text.clone(), symbol);
        Ok((symbol, text))
    }

    pub(crate) fn import_canonical_snapshot(
        &self,
        snapshot: &SymbolTableSnapshot,
    ) -> Result<(), &'static str> {
        self.update(|state| {
            state.interner.import_canonical_snapshot(snapshot)?;
            for (symbol, _) in &snapshot.entries {
                // Import validates all meanings before installing both maps.
                let text = state
                    .interner
                    .retained_text(*symbol)
                    .ok_or("admitted canonical dictionary lacks its symbol")?;
                state.configuration.remove(text.as_str());
                state.configuration.insert(text.clone(), *symbol);
            }
            Ok(())
        })
    }
    pub(crate) fn interner_snapshot(&self) -> StringInterner {
        self.with_read(Clone::clone)
    }
    pub(crate) fn snapshot(&self) -> SymbolTableSnapshot {
        self.with_read(StringInterner::snapshot)
    }
    pub(crate) fn configuration_snapshot(&self) -> SymbolTableSnapshot {
        self.read_state(|state| SymbolTableSnapshot {
            entries: state
                .configuration
                .iter()
                .map(|(text, symbol)| (*symbol, text.clone()))
                .collect(),
        })
    }
    #[cfg(test)]
    pub(crate) fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.with_read(|state| state.resolve(symbol).map(str::to_owned))
    }
    pub(crate) fn replace(&self, interner: StringInterner) {
        *self.cold.write().unwrap_or_else(|error| error.into_inner()) = None;
        self.update(|state| {
            state.configuration = interner
                .snapshot()
                .entries
                .into_iter()
                .map(|(symbol, text)| (text, symbol))
                .collect();
            state.interner = interner;
        });
    }
    pub(crate) fn with_read<T>(&self, read: impl FnOnce(&StringInterner) -> T) -> T {
        self.read_state(|state| read(&state.interner))
    }
    pub(crate) fn with_write<T>(&self, write: impl FnOnce(&mut StringInterner) -> T) -> T {
        self.update(|state| write(&mut state.interner))
    }
}

#[cfg(test)]
#[path = "symbol_table/tests.rs"]
mod tests;
