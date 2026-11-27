//! Canonical import installs meanings one at a time over a staged copy.
use super::{StringInterner, StringInternerOwnerPort as _};
use crate::symbols::data::{Symbol, SymbolTableSnapshot};

fn snapshot(entries: &[(u32, &str)]) -> SymbolTableSnapshot {
    SymbolTableSnapshot {
        entries: entries
            .iter()
            .map(|(symbol, text)| (Symbol(*symbol), (*text).into()))
            .collect(),
    }
}

#[test]
fn canonical_import_refuses_aliases_and_contradictions_without_partial_effect() {
    let mut symbols = StringInterner::default();
    symbols
        .import_canonical_snapshot(&snapshot(&[(3, "kept")]))
        .unwrap();
    let before = symbols.clone();
    for refused in [
        snapshot(&[(9, "fresh"), (3, "renamed")]),
        snapshot(&[(9, "fresh"), (4, "kept")]),
        snapshot(&[(9, "fresh"), (9, "other")]),
        snapshot(&[(9, "fresh"), (10, "fresh")]),
        snapshot(&[(9, "fresh"), (u32::MAX, "last")]),
    ] {
        assert!(symbols.import_canonical_snapshot(&refused).is_err());
        assert_eq!(symbols, before, "{refused:?} left no partial meaning");
    }
    let repeated = snapshot(&[(9, "fresh"), (9, "fresh"), (3, "kept")]);
    symbols.import_canonical_snapshot(&repeated).unwrap();
    assert_eq!(symbols.resolve(Symbol(9)), Some("fresh"));
    assert_eq!(symbols.symbol("kept"), Some(Symbol(3)));
    assert_eq!(symbols.identity_frontier(), 10);
}
