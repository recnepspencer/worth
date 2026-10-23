//! A recovery dictionary belongs to the selected retained truth, not pending work.
use crate::{
    branch::RelationalBranchRootCheckpoint,
    durability::data::{DurabilityError, RecoveryFailureClass},
    history::data::PositionedCanonicalCommit,
    runtime::RelationalRuntime,
    storage::overlay::PartitionState,
    symbols::data::{StoredSymbolText, Symbol, SymbolTableSnapshot},
};
use std::collections::BTreeMap;

pub(super) fn retained_dictionary(
    runtime: &RelationalRuntime,
    envelopes: &[PositionedCanonicalCommit],
    mirror: &[PartitionState],
    roots: &[RelationalBranchRootCheckpoint],
) -> Result<SymbolTableSnapshot, DurabilityError> {
    // Installation configuration remains recoverable even when no graph value
    // references it. config() would include speculative normalization entries;
    // the retained installation configuration deliberately does not.
    let mut entries = BTreeMap::new();
    for (symbol, text) in &runtime.config.identity.symbol_table.entries {
        retain(&mut entries, *symbol, text)?;
    }
    // Canonical envelopes own the meanings of all referenced client keys and
    // field text, including records retired since publication.
    for envelope in envelopes {
        for (symbol, text) in &envelope.symbol_texts.entries {
            retain(&mut entries, *symbol, text)?;
        }
    }
    // Roots and the historical mirror remain authoritative after log retention.
    // This is cold checkpoint reconstruction, never ordinary commit work.
    runtime.services.symbols.with_read(|symbols| {
        for partition in mirror
            .iter()
            .chain(roots.iter().flat_map(|root| root.partitions()))
        {
            partition.visit_retained_symbols(&mut |symbol| {
                let text = symbols
                    .retained_text(symbol)
                    .ok_or_else(|| corrupt("retained checkpoint storage has no symbol meaning"))?;
                retain(&mut entries, symbol, text)
            })?;
        }
        Ok::<_, DurabilityError>(())
    })?;
    Ok(SymbolTableSnapshot {
        entries: entries.into_iter().collect(),
    })
}

fn retain(
    entries: &mut BTreeMap<Symbol, StoredSymbolText>,
    symbol: Symbol,
    text: &StoredSymbolText,
) -> Result<(), DurabilityError> {
    if entries.get(&symbol).is_some_and(|prior| prior != text) {
        return Err(corrupt(
            "retained checkpoint dictionaries disagree on symbol meaning",
        ));
    }
    entries.entry(symbol).or_insert_with(|| text.clone());
    Ok(())
}

fn corrupt(detail: &'static str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
