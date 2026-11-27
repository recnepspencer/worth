//! The real symbol dictionary's transitions as a conformance trace for the
//! proved Lean model (`proofs/lean/relational_symbol_commit`).
//!
//! A `StringInterner` imports canonical meanings one at a time and in
//! batches (including symbol and text aliases, a self-contradicting batch
//! and the exhausted symbol); a `RuntimeSymbolTable` owner's pending
//! candidate reserves texts held by the candidate, by the owner, by neither
//! (a fresh identity) and by the owner under a symbol the candidate already
//! reserved for another text; and whole commits reserve their touched texts
//! and publish them, including one refused at the exhausted frontier. Every
//! step records the dictionaries before and after, both maps and the
//! frontier. Texts are decimal numbers so the model's texts are naturals.
//! Each candidate is published or dropped before the next begins, so the
//! shared pending directory never answers for another candidate.
//! With `WORTH_SYMBOL_COMMIT_TRACE=<path>` the steps are written as NDJSON,
//! which `lake exe conformance <path>` checks against the proved
//! `importMeaning`, `importAll`, `reserve` and `commit`.
use std::fmt::Write as _;

use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;

use crate::runtime::state::subsystems::services::PendingSymbolMeanings;
use crate::runtime::RuntimeSymbolTable;
use crate::symbols::data::{StringInterner, Symbol, SymbolTableSnapshot};

const LAST: u32 = u32::MAX - 1;

fn text(value: &str) -> u64 {
    value.parse().expect("trace texts are decimal numbers")
}

/// Both maps and the frontier as JSON, after asserting the maps are
/// inverse and every symbol is below the frontier.
fn dict(symbols: &StringInterner) -> String {
    let by_symbol: Vec<(u32, u64)> = symbols
        .retained_entries()
        .map(|(symbol, value)| (symbol.0, text(value.as_str())))
        .collect();
    let mut by_value: Vec<(u32, u64)> = symbols
        .snapshot()
        .entries
        .iter()
        .map(|(symbol, value)| (symbol.0, text(value.as_str())))
        .collect();
    by_value.sort_unstable();
    assert_eq!(by_symbol, by_value, "one meaning per symbol and per text");
    let next = symbols.identity_frontier();
    assert!(by_symbol.iter().all(|(symbol, _)| *symbol < next));
    let pairs = |rows: Vec<String>| rows.join(",");
    format!(
        r#"{{"by_symbol":[{}],"by_value":[{}],"next":{next}}}"#,
        pairs(
            by_symbol
                .iter()
                .map(|(s, t)| format!("[{s},{t}]"))
                .collect()
        ),
        pairs(by_value.iter().map(|(s, t)| format!("[{t},{s}]")).collect()),
    )
}

fn snapshot(entries: &[(u32, &str)]) -> SymbolTableSnapshot {
    SymbolTableSnapshot {
        entries: entries
            .iter()
            .map(|(symbol, value)| (Symbol(*symbol), (*value).into()))
            .collect(),
    }
}

fn json_entries(entries: &[(u32, &str)]) -> String {
    let rows: Vec<String> = entries.iter().map(|(s, t)| format!("[{s},{t}]")).collect();
    format!("[{}]", rows.join(","))
}

#[derive(Default)]
struct Trace {
    lines: String,
    steps: usize,
}

impl Trace {
    fn push(&mut self, kind: &str, body: String) {
        let _ = writeln!(
            self.lines,
            r#"{{"step":{},"kind":"{kind}",{body}}}"#,
            self.steps
        );
        self.steps += 1;
    }

    fn import(&mut self, symbols: &mut StringInterner, entries: &[(u32, &str)], ok: bool) {
        let before = dict(symbols);
        let result = if let [(symbol, value)] = entries {
            symbols.import_canonical_meaning(Symbol(*symbol), &(*value).into())
        } else {
            symbols.import_canonical_snapshot(&snapshot(entries))
        };
        assert_eq!(result.is_ok(), ok, "{entries:?}");
        let after = dict(symbols);
        if !ok {
            assert_eq!(before, after, "a refused import leaves no partial meaning");
        }
        let body = match entries {
            [(symbol, value)] => format!(r#""symbol":{symbol},"text":{value}"#),
            _ => format!(r#""entries":{}"#, json_entries(entries)),
        };
        let kind = if entries.len() == 1 {
            "import_meaning"
        } else {
            "import_snapshot"
        };
        self.push(
            kind,
            format!(r#""dict":{before},{body},"ok":{ok},"after":{after}"#),
        );
    }

    fn reserve(
        &mut self,
        owner: &RuntimeSymbolTable,
        cand: &mut PendingSymbolMeanings,
        value: &str,
    ) -> Option<Symbol> {
        let (owner_before, cand_before) = (owner.with_read(dict), dict(cand.interner()));
        let reserved = cand.reserve(value).ok();
        let symbol = reserved.map_or("null".to_owned(), |symbol| symbol.0.to_string());
        self.push(
            "reserve",
            format!(
                r#""owner":{owner_before},"cand":{cand_before},"text":{value},"symbol":{symbol},"owner_after":{},"cand_after":{}"#,
                owner.with_read(dict),
                dict(cand.interner()),
            ),
        );
        reserved
    }

    /// Reserve every touched text in a fresh candidate, then publish the
    /// reserved meanings in touched order.
    fn commit(&mut self, owner: &RuntimeSymbolTable, touched: &[&str]) -> bool {
        let before = owner.with_read(dict);
        let mut candidate = owner.begin_pending_meanings();
        let reserved: Result<Vec<_>, _> = touched
            .iter()
            .map(|value| {
                candidate
                    .reserve(value)
                    .map(|symbol| (symbol, (*value).into()))
            })
            .collect();
        let ok = match reserved {
            Ok(entries) => {
                let meanings = SymbolTableSnapshot { entries };
                let binding = candidate.binding();
                let publication = candidate.seal_for_publication(&meanings, &binding, []);
                publication.and_then(|prepared| prepared.publish()).is_ok()
            }
            Err(_) => false,
        };
        let touched = touched.join(",");
        self.push(
            "commit",
            format!(
                r#""owner":{before},"touched":[{touched}],"ok":{ok},"after":{}"#,
                owner.with_read(dict)
            ),
        );
        ok
    }
}

#[test]
fn symbol_dictionary_transitions_trace_against_the_proved_model() {
    let mut trace = Trace::default();
    let mut symbols = StringInterner::default();
    trace.import(&mut symbols, &[(3, "30")], true);
    trace.import(&mut symbols, &[(3, "30")], true);
    trace.import(&mut symbols, &[(5, "31")], true);
    trace.import(&mut symbols, &[(3, "32")], false);
    trace.import(&mut symbols, &[(6, "30")], false);
    trace.import(&mut symbols, &[(u32::MAX, "33")], false);
    trace.import(&mut symbols, &[(9, "40"), (9, "40"), (3, "30")], true);
    trace.import(&mut symbols, &[(10, "41"), (3, "42")], false);
    trace.import(&mut symbols, &[(11, "43"), (12, "43")], false);
    trace.import(&mut symbols, &[(LAST, "34")], true);
    trace.import(&mut symbols, &[(7, "44"), (8, "45")], true);

    let owner = RuntimeSymbolTable::default();
    owner
        .import_canonical_snapshot(&snapshot(&[(2, "20")]))
        .unwrap();
    let mut cand = owner.begin_pending_meanings();
    owner
        .import_canonical_snapshot(&snapshot(&[(4, "21")]))
        .unwrap();
    assert_eq!(trace.reserve(&owner, &mut cand, "20"), Some(Symbol(2)));
    assert_eq!(trace.reserve(&owner, &mut cand, "21"), Some(Symbol(4)));
    let fresh = trace
        .reserve(&owner, &mut cand, "22")
        .expect("a fresh identity");
    assert_eq!(trace.reserve(&owner, &mut cand, "22"), Some(fresh));
    // A peer's canonical meaning lands on the identity this candidate holds.
    owner
        .import_canonical_snapshot(&snapshot(&[(fresh.0, "23")]))
        .unwrap();
    assert_eq!(
        trace.reserve(&owner, &mut cand, "23"),
        None,
        "aliases a reservation"
    );
    drop(cand);

    assert!(trace.commit(&owner, &["20", "24", "25", "24"]));
    assert!(trace.commit(&owner, &["25", "26"]));
    assert!(trace.commit(&owner, &[]));

    let exhausted = RuntimeSymbolTable::default();
    exhausted
        .import_canonical_snapshot(&snapshot(&[(LAST, "50")]))
        .unwrap();
    let mut cand = exhausted.begin_pending_meanings();
    assert_eq!(
        trace.reserve(&exhausted, &mut cand, "50"),
        Some(Symbol(LAST))
    );
    assert_eq!(
        trace.reserve(&exhausted, &mut cand, "51"),
        None,
        "frontier exhausted"
    );
    drop(cand);
    assert!(trace.commit(&exhausted, &["50"]));
    assert!(!trace.commit(&exhausted, &["50", "52"]));

    assert_eq!(trace.steps, 23);
    if let Ok(path) = std::env::var("WORTH_SYMBOL_COMMIT_TRACE") {
        std::fs::write(path, trace.lines).unwrap();
    }
}
