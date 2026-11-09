//! Publish a complete persistent dictionary after native editing succeeds.
use super::*;

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeSymbolState {
    pub(super) interner: StringInterner,
    pub(super) admissions:
        im::OrdMap<crate::storage::residency::RelationalRegionSymbolAdmissionIdentity, ()>,
    pub(super) configuration: im::OrdMap<SymbolTextLookupKey, Symbol>,
}
impl RuntimeSymbolTable {
    pub(super) fn read_state<T>(&self, read: impl FnOnce(&RuntimeSymbolState) -> T) -> T {
        // Writers stage edits before replacing this state; unwinding preserves
        // the last complete dictionary even when it poisons the lock.
        let state = self.state.read().unwrap_or_else(|error| error.into_inner());
        read(&state)
    }
    pub(super) fn update<T>(&self, write: impl FnOnce(&mut RuntimeSymbolState) -> T) -> T {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());
        // Persistent map roots clone in O(1); only edited paths allocate.
        let mut staged = state.clone();
        let result = write(&mut staged);
        let prior = std::mem::replace(&mut *state, staged);
        drop(state);
        drop(prior);
        result
    }
}

impl RuntimeSymbolState {
    /// Skip replacement only when both identity and retained custody already agree.
    pub(super) fn retains_configuration_text(
        &self,
        symbol: Symbol,
        text: &StoredSymbolText,
    ) -> bool {
        let Some((prior, recorded)) = self.configuration.get_key_value(text.as_str()) else {
            return false;
        };
        *recorded == symbol
            && match (prior.is_managed(), text.is_managed()) {
                (false, false) => true,
                (true, true) => {
                    !text.as_str().is_empty() && std::ptr::eq(prior.as_str(), text.as_str())
                }
                (false, true) | (true, false) => false,
            }
    }
}
