use std::sync::{Arc, RwLock};

use im::OrdMap;

use crate::symbols::data::{StringInterner, Symbol, SymbolTableSnapshot};

#[derive(Debug)]
pub(crate) struct RuntimeSymbolTable {
    state: Arc<RwLock<StringInterner>>,
    configuration_snapshot: Arc<RwLock<OrdMap<String, Symbol>>>,
}

impl Default for RuntimeSymbolTable {
    fn default() -> Self {
        Self {
            state: Arc::new(RwLock::new(StringInterner::default())),
            configuration_snapshot: Arc::new(RwLock::new(OrdMap::new())),
        }
    }
}

impl Clone for RuntimeSymbolTable {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            configuration_snapshot: Arc::clone(&self.configuration_snapshot),
        }
    }
}

impl PartialEq for RuntimeSymbolTable {
    fn eq(&self, other: &Self) -> bool {
        self.interner_snapshot() == other.interner_snapshot()
            && self.configuration_snapshot() == other.configuration_snapshot()
    }
}

impl Eq for RuntimeSymbolTable {}

impl RuntimeSymbolTable {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let state = self
            .state
            .read()
            .expect("runtime symbol table lock poisoned");
        let configuration = self
            .configuration_snapshot
            .read()
            .expect("runtime symbol configuration snapshot lock poisoned");
        Self {
            state: Arc::new(RwLock::new(state.clone())),
            configuration_snapshot: Arc::new(RwLock::new(configuration.clone())),
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
            configuration.insert(value, symbol);
        }
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
        *self
            .configuration_snapshot
            .write()
            .expect("runtime symbol configuration snapshot lock poisoned") = snapshot
            .entries
            .into_iter()
            .map(|(symbol, value)| (value, symbol))
            .collect();
    }

    pub(crate) fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.state
            .read()
            .expect("runtime symbol table lock poisoned")
            .resolve(symbol)
            .map(str::to_owned)
    }

    pub(crate) fn replace(&self, interner: StringInterner) {
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
mod tests {
    use super::RuntimeSymbolTable;

    #[test]
    fn configuration_snapshot_stays_sorted_and_detached_owners_do_not_share_new_keys() {
        let owner = RuntimeSymbolTable::default();
        owner.normalize_client_keys(|interner| {
            let beta = interner.intern("beta");
            let alpha = interner.intern("alpha");
            vec![(beta, "beta".to_string()), (alpha, "alpha".to_string())]
        });
        let detached = owner.detached_owner_snapshot();
        owner.normalize_client_keys(|interner| {
            let gamma = interner.intern("gamma");
            vec![(gamma, "gamma".to_string())]
        });

        let values = |table: &RuntimeSymbolTable| {
            table
                .configuration_snapshot()
                .entries
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>()
        };
        assert_eq!(values(&owner), ["alpha", "beta", "gamma"]);
        assert_eq!(values(&detached), ["alpha", "beta"]);
    }
}
