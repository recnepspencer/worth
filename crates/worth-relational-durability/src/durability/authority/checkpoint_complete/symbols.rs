//! An admitted private lookup index over authentic already-owned text leaves.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod retained;
use super::source::OriginalCanonicalCommit;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use contracts::symbols::data::stored_text::StoredSymbolText;
use contracts::symbols::data::string_interner::StringInterner;
use contracts::symbols::data::symbol_text_lookup::SymbolTextLookup;
use contracts::symbols::data::Symbol;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct OriginalSymbolDictionary<'a> {
    by_symbol: ExecutionOrderedMap<Symbol, &'a StoredSymbolText>,
    by_text: ExecutionOrderedMap<&'a str, Symbol>,
}
impl<'a> OriginalSymbolDictionary<'a> {
    pub(super) fn capture(
        existing: &'a StringInterner,
        commits: &'a [OriginalCanonicalCommit],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut dictionary = Self {
            by_symbol: Default::default(),
            by_text: Default::default(),
        };
        for (symbol, text) in retained::RetainedSymbols::new(existing, execution)? {
            dictionary.insert(*symbol, text, execution)?;
        }
        for commit in commits {
            for (symbol, text) in &commit.envelope.owner_symbol_texts().entries {
                dictionary.insert(*symbol, text, execution)?;
            }
        }
        Ok(dictionary)
    }
    pub(super) fn len(&self) -> usize {
        self.by_symbol.len()
    }
    pub(super) fn entries(&self) -> impl Iterator<Item = (Symbol, &StoredSymbolText)> + '_ {
        self.by_symbol.iter().map(|(&symbol, &text)| (symbol, text))
    }
    fn insert(
        &mut self,
        symbol: Symbol,
        text: &'a StoredSymbolText,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if let Some(previous) = self.by_symbol.get(&symbol) {
            return if previous.as_str() == text.as_str()
                && self.by_text.get(&text.as_str()) == Some(&symbol)
            {
                Ok(())
            } else {
                Err(Denial::OriginalAuthorityUnavailable)
            };
        }
        if self.by_text.get(&text.as_str()).is_some() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        // Only new fixed index cells are owned. Payloads borrow the authentic
        // source cache or decoded dictionary, never cloning System strings.
        self.by_symbol
            .insert_cloned_native_for_request(&symbol, &text, *execution)
            .map_err(map_stop)?;
        self.by_text
            .insert_cloned_native_for_request(&text.as_str(), &symbol, *execution)
            .map_err(map_stop)
    }
}
impl SymbolTextLookup for OriginalSymbolDictionary<'_> {
    fn symbol(&self, text: &str) -> Option<Symbol> {
        self.by_text.get(&text).copied()
    }
    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.by_symbol.get(&symbol).map(|text| text.as_str())
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}
