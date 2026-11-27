use im::{HashMap, OrdMap};

use serde::{Deserialize, Serialize};

use super::{InternedString, StoredSymbolText, Symbol, SymbolTableSnapshot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringInterner {
    next_symbol: u32,
    by_value: HashMap<StoredSymbolText, Symbol>,
    by_symbol: OrdMap<Symbol, StoredSymbolText>,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self {
            next_symbol: 1,
            by_value: HashMap::new(),
            by_symbol: OrdMap::new(),
        }
    }
}

impl StringInterner {
    pub fn symbol(&self, value: &str) -> Option<Symbol> {
        self.by_value.get(value).copied()
    }

    pub fn contains(&self, value: &str) -> bool {
        self.by_value.contains_key(value)
    }

    pub fn intern(&mut self, value: &str) -> Result<Symbol, super::RelationalSymbolExhausted> {
        if let Some(symbol) = self.by_value.get(value) {
            return Ok(*symbol);
        }
        let symbol = Symbol(self.next_symbol);
        self.next_symbol = self
            .next_symbol
            .checked_add(1)
            .ok_or(super::RelationalSymbolExhausted)?;
        let text = StoredSymbolText::from(value);
        self.by_value.insert(text.clone(), symbol);
        self.by_symbol.insert(symbol, text);
        Ok(symbol)
    }

    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.by_symbol.get(&symbol).map(StoredSymbolText::as_str)
    }

    pub fn retained_text(&self, symbol: Symbol) -> Option<&StoredSymbolText> {
        self.by_symbol.get(&symbol)
    }

    pub fn normalize(
        &mut self,
        value: InternedString,
    ) -> Result<InternedString, super::RelationalSymbolExhausted> {
        match value {
            InternedString::Raw(raw) => self.intern(&raw).map(InternedString::Symbol),
            symbol => Ok(symbol),
        }
    }

    pub fn snapshot(&self) -> SymbolTableSnapshot {
        let mut entries = self
            .by_value
            .iter()
            .map(|(value, symbol)| (*symbol, value.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.1.cmp(&right.1));
        super::snapshot_observation::materialized(entries.len());
        SymbolTableSnapshot { entries }
    }

    pub fn restore_snapshot(
        &mut self,
        _lane: &crate::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        snapshot: SymbolTableSnapshot,
    ) {
        self.by_value = HashMap::new();
        self.by_symbol = OrdMap::new();
        self.next_symbol = 1;
        for (symbol, value) in snapshot.entries {
            self.by_value.insert(value.clone(), symbol);
            self.by_symbol.insert(symbol, value);
            self.next_symbol = self.next_symbol.max(symbol.0.saturating_add(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringInterner;
    use crate::symbols::data::Symbol;
    use serde::{Deserialize, Serialize};
    use std::collections::{BTreeMap, HashMap};

    #[derive(Serialize, Deserialize)]
    struct LegacyStringInterner {
        next_symbol: u32,
        by_value: HashMap<String, Symbol>,
        by_symbol: BTreeMap<Symbol, String>,
    }

    #[test]
    fn snapshot_entries_are_ordered_by_string_value() {
        let mut left = StringInterner::default();
        left.intern("beta")
            .expect("symbol fixture has identity capacity");
        left.intern("alpha")
            .expect("symbol fixture has identity capacity");

        let mut right = StringInterner::default();
        right
            .intern("alpha")
            .expect("symbol fixture has identity capacity");
        right
            .intern("beta")
            .expect("symbol fixture has identity capacity");

        let left_snapshot = left.snapshot();
        let right_snapshot = right.snapshot();

        assert_eq!(
            left_snapshot
                .entries
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
        assert_eq!(
            right_snapshot
                .entries
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
    }

    #[test]
    fn restore_snapshot_preserves_symbol_ids_after_canonical_snapshot_sort() {
        let mut interner = StringInterner::default();
        let beta = interner
            .intern("beta")
            .expect("symbol fixture has identity capacity");
        let alpha = interner
            .intern("alpha")
            .expect("symbol fixture has identity capacity");

        let snapshot = interner.snapshot();
        let mut restored = StringInterner::default();
        restored.restore_snapshot(
            &{
                use crate::storage::substrate::maintenance::reconstruction::ReconstructionLaneOwnerPort as _;
                crate::storage::substrate::maintenance::reconstruction::ReconstructionLane::oracle()
            },
            snapshot,
        );

        assert_eq!(restored.resolve(beta), Some("beta"));
        assert_eq!(restored.resolve(alpha), Some("alpha"));
        assert_eq!(restored.resolve(Symbol(9999)), None);
    }

    #[test]
    fn detached_clone_shares_prior_meaning_without_publishing_new_symbols() {
        let mut owner = StringInterner::default();
        let original = owner
            .intern("original")
            .expect("symbol fixture has identity capacity");
        let mut detached = owner.clone();
        let candidate = detached
            .intern("candidate")
            .expect("symbol fixture has identity capacity");

        assert_eq!(owner.resolve(original), Some("original"));
        assert_eq!(owner.symbol("candidate"), None);
        assert_eq!(detached.resolve(candidate), Some("candidate"));
        assert_eq!(detached.symbol("original"), Some(original));
    }

    #[test]
    fn persistent_maps_read_and_write_the_prior_symbol_wire_shape() {
        let mut interner = StringInterner::default();
        let alpha = interner
            .intern("alpha")
            .expect("symbol fixture has identity capacity");
        let beta = interner
            .intern("beta")
            .expect("symbol fixture has identity capacity");
        let bytes = rmp_serde::to_vec_named(&interner).expect("serialize persistent maps");
        let legacy: LegacyStringInterner =
            rmp_serde::from_slice(&bytes).expect("legacy reader accepts symbol mapping");
        assert_eq!(legacy.by_value.get("alpha"), Some(&alpha));
        assert_eq!(
            legacy.by_symbol.get(&beta).map(String::as_str),
            Some("beta")
        );

        let legacy_bytes = rmp_serde::to_vec_named(&legacy).expect("serialize legacy maps");
        let restored: StringInterner =
            rmp_serde::from_slice(&legacy_bytes).expect("persistent reader accepts prior mapping");
        assert_eq!(restored, interner);
    }
}

#[cfg(test)]
mod exhaustion_tests {
    use super::*;
    #[test]
    fn exhausted_frontier_preserves_existing_meanings_and_never_wraps() {
        let mut symbols = StringInterner::default();
        let kept = symbols.intern("retained").unwrap();
        symbols.next_symbol = u32::MAX;
        let before = symbols.clone();
        for _ in 0..2 {
            assert_eq!(
                symbols.intern("refused"),
                Err(super::super::RelationalSymbolExhausted)
            );
            assert_eq!(symbols, before);
        }
        assert_eq!(symbols.intern("retained"), Ok(kept));
        assert_eq!(symbols.resolve(kept), Some("retained"));
    }
}

#[cfg(test)]
mod allocation_tests;
#[cfg(test)]
mod import_tests;

/// Operations used by the Relational owners.
pub trait StringInternerOwnerPort: Sized {
    /// Ordinary borrowed traversal. Its two path buffers require caller admission
    /// when used inside an original-request operation.
    fn retained_entries(&self) -> im::ordmap::Iter<'_, Symbol, StoredSymbolText>;
    fn retained_entry_count(&self) -> usize;
    fn identity_frontier(&self) -> u32;
    /// Spend an identity without publishing any text meaning. The runtime owns
    /// pending reservation lifetime and never restores this frontier on abort.
    fn reserve_identity(&mut self) -> Result<Symbol, super::RelationalFieldTextAdmissionDenial>;
    fn reserve_frontier(&mut self, frontier: u32) -> Result<(), &'static str>;
    fn check_text_capacity(
        &self,
        text: &str,
    ) -> Result<(), super::RelationalFieldTextAdmissionDenial>;
    fn admit_text(
        &mut self,
        text: StoredSymbolText,
    ) -> Result<Symbol, super::RelationalFieldTextAdmissionDenial>;
    /// Install one canonical meaning, refusing one that aliases an existing
    /// meaning. `O(log N)` persistent-map writes sharing every untouched node
    /// and text; none when this exact custody is already installed.
    fn import_canonical_meaning(
        &mut self,
        symbol: Symbol,
        text: &StoredSymbolText,
    ) -> Result<(), &'static str>;
    /// All of `snapshot`'s meanings or none: `O(|snapshot| log N)`.
    fn import_canonical_snapshot(
        &mut self,
        snapshot: &SymbolTableSnapshot,
    ) -> Result<(), &'static str>;
    /// Compare persistent roots, visiting only nonshared map paths.
    fn checked_additions_since(&self, previous: &Self)
        -> Result<SymbolTableSnapshot, &'static str>;
}

impl StringInternerOwnerPort for StringInterner {
    fn retained_entries(&self) -> im::ordmap::Iter<'_, Symbol, StoredSymbolText> {
        self.by_symbol.iter()
    }

    fn retained_entry_count(&self) -> usize {
        self.by_symbol.len()
    }

    fn identity_frontier(&self) -> u32 {
        self.next_symbol
    }

    fn reserve_identity(&mut self) -> Result<Symbol, super::RelationalFieldTextAdmissionDenial> {
        let next = self
            .next_symbol
            .checked_add(1)
            .ok_or(super::RelationalFieldTextAdmissionDenial::SymbolExhausted)?;
        let symbol = Symbol(self.next_symbol);
        self.next_symbol = next;
        Ok(symbol)
    }

    fn reserve_frontier(&mut self, frontier: u32) -> Result<(), &'static str> {
        if frontier == 0 {
            return Err("symbol frontier is empty");
        }
        self.next_symbol = self.next_symbol.max(frontier);
        Ok(())
    }

    fn check_text_capacity(
        &self,
        text: &str,
    ) -> Result<(), super::RelationalFieldTextAdmissionDenial> {
        if self.symbol(text).is_none() && self.next_symbol.checked_add(1).is_none() {
            return Err(super::RelationalFieldTextAdmissionDenial::SymbolExhausted);
        }
        Ok(())
    }

    fn admit_text(
        &mut self,
        text: StoredSymbolText,
    ) -> Result<Symbol, super::RelationalFieldTextAdmissionDenial> {
        let symbol = match self.symbol(text.as_str()) {
            Some(symbol) => symbol,
            None => {
                let next = self
                    .next_symbol
                    .checked_add(1)
                    .ok_or(super::RelationalFieldTextAdmissionDenial::SymbolExhausted)?;
                let symbol = Symbol(self.next_symbol);
                self.next_symbol = next;
                symbol
            }
        };
        // Equal-key insert alone keeps the old System key in persistent maps.
        self.by_value.remove(text.as_str());
        self.by_value.insert(text.clone(), symbol);
        self.by_symbol.insert(symbol, text);
        Ok(symbol)
    }

    fn import_canonical_meaning(
        &mut self,
        symbol: Symbol,
        text: &StoredSymbolText,
    ) -> Result<(), &'static str> {
        let prior = self.by_symbol.get(&symbol);
        if prior.is_some_and(|prior| prior != text)
            || self
                .symbol(text.as_str())
                .is_some_and(|prior| prior != symbol)
        {
            return Err("canonical tail dictionary aliases existing symbol meaning");
        }
        if symbol.0 == u32::MAX {
            return Err("canonical tail symbol exhausted");
        }
        // Imported System text never displaces the same meaning's Managed custody.
        let text = prior
            .filter(|prior| prior.is_managed() && !text.is_managed())
            .unwrap_or(text);
        if !prior.is_some_and(|prior| prior.is_same_custody(text)) {
            let text = text.clone();
            // Equal-key insert alone keeps the old key in persistent maps.
            self.by_value.remove(text.as_str());
            self.by_value.insert(text.clone(), symbol);
            self.by_symbol.insert(symbol, text);
        }
        self.next_symbol = self.next_symbol.max(symbol.0.saturating_add(1));
        Ok(())
    }

    fn import_canonical_snapshot(
        &mut self,
        snapshot: &SymbolTableSnapshot,
    ) -> Result<(), &'static str> {
        // Each meaning is checked against the dictionary holding the earlier
        // ones, so a batch that contradicts itself refuses like an alias. The
        // persistent roots copy in O(1); a refusal leaves `self` unchanged.
        let mut staged = self.clone();
        for (symbol, text) in &snapshot.entries {
            staged.import_canonical_meaning(*symbol, text)?;
        }
        *self = staged;
        Ok(())
    }

    fn checked_additions_since(
        &self,
        previous: &Self,
    ) -> Result<SymbolTableSnapshot, &'static str> {
        let mut entries = Vec::new();
        for changed in previous.by_symbol.diff(&self.by_symbol) {
            match changed {
                im::ordmap::DiffItem::Add(id, text) => entries.push((*id, text.clone())),
                im::ordmap::DiffItem::Update { .. } | im::ordmap::DiffItem::Remove(_, _) => {
                    return Err("candidate binding erased or replaced a frozen meaning")
                }
            }
        }
        Ok(SymbolTableSnapshot { entries })
    }
}
