mod cold_pages;
pub(crate) use cold_pages::ColdSymbolDirectoryDenial;

use std::sync::{Arc, RwLock};

use im::OrdMap;

use crate::symbols::data::{StoredSymbolText, StringInterner, Symbol, SymbolTableSnapshot};

#[path = "symbol_table/reservations.rs"]
mod reservations;
pub(crate) use reservations::{
    PendingSymbolMeanings, PreparedSymbolPublication, SymbolReservationDenial,
};

#[derive(Debug)]
pub(crate) struct RuntimeSymbolTable {
    state: Arc<RwLock<StringInterner>>,
    configuration_snapshot: Arc<RwLock<OrdMap<StoredSymbolText, Symbol>>>,
    cold: Arc<RwLock<Option<Arc<super::symbol_residency::RuntimeSymbolResidency>>>>,
    reservations: Arc<std::sync::Mutex<reservations::SymbolReservationDirectory>>,
}

impl Default for RuntimeSymbolTable {
    fn default() -> Self {
        Self {
            state: Arc::new(RwLock::new(StringInterner::default())),
            configuration_snapshot: Arc::new(RwLock::new(OrdMap::new())),
            cold: Arc::new(RwLock::new(None)),
            reservations: Arc::default(),
        }
    }
}

impl Clone for RuntimeSymbolTable {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            configuration_snapshot: Arc::clone(&self.configuration_snapshot),
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
        let snapshot = || {
            let state = self
                .state
                .read()
                .expect("runtime symbol table lock poisoned");
            let configuration = self
                .configuration_snapshot
                .read()
                .expect("runtime symbol configuration snapshot lock poisoned");
            (state.clone(), configuration.clone())
        };
        // Page admission locks pages before the owner dictionary. Fork uses
        // that same order, so copied admission marks and text are one snapshot.
        let (cold, (state, configuration)) = match cold {
            Some(pages) => {
                let (pages, snapshot) = pages.detached_with_owner(snapshot);
                (Some(Arc::new(pages)), snapshot)
            }
            None => (None, snapshot()),
        };
        Self {
            state: Arc::new(RwLock::new(state)),
            configuration_snapshot: Arc::new(RwLock::new(configuration)),
            reservations: Arc::default(),
            cold: Arc::new(RwLock::new(cold)),
        }
    }
    pub(crate) fn normalize_client_keys(
        &self,
        normalize: impl FnOnce(&mut StringInterner) -> Vec<(Symbol, String)>,
    ) {
        let mut interner = self
            .state
            .write()
            .expect("runtime symbol table lock poisoned");
        let new_entries = normalize(&mut interner);
        let mut configuration = self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned");
        for (symbol, value) in new_entries {
            configuration.insert(value.into(), symbol);
        }
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
        let mut state = self
            .state
            .write()
            .expect("runtime symbol table lock poisoned");
        if self.pending_symbol(raw).is_none() {
            state.check_text_capacity(raw)?;
        }
        let text = StoredSymbolText::admit(execution, input)?;
        self.publish_admitted_text(execution, &mut state, text)
    }

    fn publish_admitted_text(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        state: &mut StringInterner,
        text: StoredSymbolText,
    ) -> Result<(Symbol, StoredSymbolText), crate::symbols::data::RelationalFieldTextAdmissionDenial>
    {
        use crate::symbols::data::RelationalFieldTextAdmissionDenial as Denial;
        let mut configuration = self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned");
        execution.check_live().map_err(Denial::Stopped)?;
        let symbol = self.admit_with_pending_identity(state, text.clone())?;
        configuration.remove(text.as_str());
        configuration.insert(text.clone(), symbol);
        Ok((symbol, text))
    }

    pub(crate) fn import_canonical_snapshot(
        &self,
        snapshot: &SymbolTableSnapshot,
    ) -> Result<(), &'static str> {
        let mut state = self
            .state
            .write()
            .expect("runtime symbol table lock poisoned");
        let mut configuration = self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned");
        state.import_canonical_snapshot(snapshot)?;
        for (symbol, _) in &snapshot.entries {
            let text = state
                .retained_text(*symbol)
                .expect("preflighted canonical symbol");
            configuration.remove(text.as_str());
            configuration.insert(text.clone(), *symbol);
        }
        Ok(())
    }
    pub(crate) fn interner_snapshot(&self) -> StringInterner {
        self.state
            .read()
            .expect("runtime symbol table lock poisoned")
            .clone()
    }

    pub(crate) fn snapshot(&self) -> SymbolTableSnapshot {
        self.state
            .read()
            .expect("runtime symbol table lock poisoned")
            .snapshot()
    }

    pub(crate) fn configuration_snapshot(&self) -> SymbolTableSnapshot {
        let configuration = self
            .configuration_snapshot
            .read()
            .expect("runtime symbol configuration snapshot lock poisoned");
        SymbolTableSnapshot {
            entries: configuration
                .iter()
                .map(|(value, symbol)| (*symbol, value.clone()))
                .collect(),
        }
    }

    pub(crate) fn initialize_configuration_snapshot(&self, snapshot: SymbolTableSnapshot) {
        self.state
            .write()
            .expect("runtime symbol table lock poisoned")
            .restore_snapshot(snapshot.clone());
        *self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned") = snapshot
            .entries
            .into_iter()
            .map(|(symbol, value)| (value, symbol))
            .collect();
    }

    #[cfg(test)]
    pub(crate) fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.state
            .read()
            .expect("runtime symbol table lock poisoned")
            .resolve(symbol)
            .map(str::to_owned)
    }

    pub(crate) fn replace(&self, interner: StringInterner) {
        *self.cold.write().unwrap_or_else(|error| error.into_inner()) = None;
        let snapshot = interner.snapshot();
        let mut state = self
            .state
            .write()
            .expect("runtime symbol table lock poisoned");
        let mut configuration = self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned");
        *state = interner;
        *configuration = snapshot
            .entries
            .into_iter()
            .map(|(symbol, value)| (value, symbol))
            .collect();
    }

    pub(crate) fn with_read<T>(&self, read: impl FnOnce(&StringInterner) -> T) -> T {
        let guard = self
            .state
            .read()
            .expect("runtime symbol table lock poisoned");
        read(&guard)
    }

    pub(crate) fn with_write<T>(&self, write: impl FnOnce(&mut StringInterner) -> T) -> T {
        let mut guard = self
            .state
            .write()
            .expect("runtime symbol table lock poisoned");
        write(&mut guard)
    }
}

#[cfg(test)]
#[path = "symbol_table/tests.rs"]
mod tests;
