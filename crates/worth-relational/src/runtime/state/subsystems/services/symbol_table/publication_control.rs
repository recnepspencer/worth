//! A closed court owns the actual dictionary and observes its retained keys.
use super::{RuntimeSymbolTable, Symbol, SymbolTableSnapshot};
use crate::symbols::data::{RelationalFieldTextAdmissionDenial, StoredSymbolText};
use worth_execution::ExecutionRequest;

#[derive(Default)]
pub struct RelationalSymbolTableForTest {
    table: RuntimeSymbolTable,
}
impl RelationalSymbolTableForTest {
    pub fn copy_text(
        request: &ExecutionRequest<'_, '_>,
        text: &str,
    ) -> Result<StoredSymbolText, RelationalFieldTextAdmissionDenial> {
        StoredSymbolText::copy_from_str(request, text)
    }
    pub fn import_canonical_snapshot(
        &self,
        snapshot: &SymbolTableSnapshot,
    ) -> Result<(), &'static str> {
        self.table.import_canonical_snapshot(snapshot)
    }
    pub fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.table
            .with_read(|state| state.resolve(symbol).map(str::to_owned))
    }
    pub fn configuration_key_is_managed(&self, text: &str) -> bool {
        self.table.read_state(|state| {
            state
                .configuration
                .get_key_value(text)
                .is_some_and(|(key, _)| key.is_managed())
        })
    }
}
