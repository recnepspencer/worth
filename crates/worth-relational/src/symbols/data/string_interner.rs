use im::{HashMap, OrdMap};

use serde::{Deserialize, Serialize};

use super::{InternedString, Symbol, SymbolTableSnapshot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringInterner {
    next_symbol: u32,
    by_value: HashMap<String, Symbol>,
    by_symbol: OrdMap<Symbol, String>,
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

    pub fn intern(&mut self, value: &str) -> Symbol {
        if let Some(symbol) = self.by_value.get(value) {
            return *symbol;
        }
        let symbol = Symbol(self.next_symbol);
        self.next_symbol += 1;
        self.by_value.insert(value.to_string(), symbol);
        self.by_symbol.insert(symbol, value.to_string());
        symbol
    }

    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.by_symbol.get(&symbol).map(String::as_str)
    }

    pub fn normalize(&mut self, value: InternedString) -> InternedString {
        match value {
            InternedString::Raw(raw) => InternedString::Symbol(self.intern(&raw)),
            symbol => symbol,
        }
    }

    pub fn snapshot(&self) -> SymbolTableSnapshot {
        let mut entries = self
            .by_value
            .iter()
            .map(|(value, symbol)| (*symbol, value.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.1.cmp(&right.1));
        SymbolTableSnapshot { entries }
    }

    pub fn restore_snapshot(&mut self, snapshot: SymbolTableSnapshot) {
        self.by_value = HashMap::new();
        self.by_symbol = OrdMap::new();
        self.next_symbol = 1;
        for (symbol, value) in snapshot.entries {
            self.by_value.insert(value.clone(), symbol);
            self.by_symbol.insert(symbol, value);
            self.next_symbol = self.next_symbol.max(symbol.0 + 1);
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
        left.intern("beta");
        left.intern("alpha");

        let mut right = StringInterner::default();
        right.intern("alpha");
        right.intern("beta");

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
        let beta = interner.intern("beta");
        let alpha = interner.intern("alpha");

        let snapshot = interner.snapshot();
        let mut restored = StringInterner::default();
        restored.restore_snapshot(snapshot);

        assert_eq!(restored.resolve(beta), Some("beta"));
        assert_eq!(restored.resolve(alpha), Some("alpha"));
        assert_eq!(restored.resolve(Symbol(9999)), None);
    }

    #[test]
    fn detached_clone_shares_prior_meaning_without_publishing_new_symbols() {
        let mut owner = StringInterner::default();
        let original = owner.intern("original");
        let mut detached = owner.clone();
        let candidate = detached.intern("candidate");

        assert_eq!(owner.resolve(original), Some("original"));
        assert_eq!(owner.symbol("candidate"), None);
        assert_eq!(detached.resolve(candidate), Some("candidate"));
        assert_eq!(detached.symbol("original"), Some(original));
    }

    #[test]
    fn persistent_maps_read_and_write_the_prior_symbol_wire_shape() {
        let mut interner = StringInterner::default();
        let alpha = interner.intern("alpha");
        let beta = interner.intern("beta");
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

    #[cfg(feature = "allocation-probes")]
    #[test]
    fn detached_clone_allocation_is_flat_in_prior_symbol_population() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("isolated_detached_clone_allocation_probe")
            .arg("--test-threads=1")
            .env("WORTH_SYMBOL_CLONE_ALLOCATION_PROBE", "1")
            .output()
            .expect("isolated symbol allocation probe starts");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(feature = "allocation-probes")]
    #[test]
    fn isolated_detached_clone_allocation_probe() {
        if std::env::var_os("WORTH_SYMBOL_CLONE_ALLOCATION_PROBE").is_none() {
            return;
        }
        let measured = |count: usize| {
            let mut interner = StringInterner::default();
            for index in 0..count {
                interner.intern(&format!("key-{index:05}"));
            }
            let last = format!("key-{:05}", count - 1);
            let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
            let detached = interner.clone();
            let stats = region.change();
            assert_eq!(detached.symbol(&last), interner.symbol(&last));
            stats
        };
        let small = measured(100);
        let large = measured(10_000);
        assert_eq!(small.allocations, large.allocations);
        assert_eq!(small.bytes_allocated, large.bytes_allocated);
    }
}
