//! A measured core allowance plus each driven member kind's constant symbol cost.
use std::collections::{BTreeMap, BTreeSet};

// c8291a5e0b: forty owned aspects measured seven symbols each.
const OWNED_ASPECT_SYMBOL_CEILING: usize = 7;
// f0ad5db7ff: forty extension aspects measured eleven symbols each.
const EXTENSION_ASPECT_SYMBOL_CEILING: usize = 11;
// This relations probe: four ExtensionLink members measured eight symbols each.
const RELATION_SYMBOL_CEILING: usize = 8;
// The same probe's four ExtensionPayload members measured seven symbols each.
const PAYLOAD_ASPECT_SYMBOL_CEILING: usize = 7;
// This expanded probe measured 2,617 total; 752 symbols mention at least one member.
const MEASURED_CORE_SYMBOLS: usize = 1_865;
// The original fixed ceiling left 2,500 - 1,077 symbols of core headroom.
const CORE_HEADROOM: usize = 1_423;
const CORE_SYMBOL_CEILING: usize = MEASURED_CORE_SYMBOLS + CORE_HEADROOM;

#[derive(Clone, Copy)]
pub(super) struct Kind {
    prefix: &'static str,
    ceiling: usize,
}
pub(super) const OWNED_ASPECT: Kind = Kind {
    prefix: "::Fact<",
    ceiling: OWNED_ASPECT_SYMBOL_CEILING,
};
pub(super) const EXTENSION_ASPECT: Kind = Kind {
    prefix: "::ExtensionFact<",
    ceiling: EXTENSION_ASPECT_SYMBOL_CEILING,
};
pub(super) const RELATION: Kind = Kind {
    prefix: "::ExtensionLink<",
    ceiling: RELATION_SYMBOL_CEILING,
};
pub(super) const PAYLOAD_ASPECT: Kind = Kind {
    prefix: "::ExtensionPayload<",
    ceiling: PAYLOAD_ASPECT_SYMBOL_CEILING,
};
const KINDS: [Kind; 4] = [OWNED_ASPECT, EXTENSION_ASPECT, RELATION, PAYLOAD_ASPECT];

fn members(line: &str, prefix: &str) -> BTreeSet<usize> {
    line.match_indices(prefix)
        .filter_map(|(start, _)| line[start + prefix.len()..].split_once('>')?.0.parse().ok())
        .collect()
}
fn counts(definitions: &[String], kind: Kind) -> BTreeMap<usize, usize> {
    let mut counts = BTreeMap::new();
    for line in definitions {
        for member in members(line, kind.prefix) {
            *counts.entry(member).or_default() += 1;
        }
    }
    counts
}
pub(super) fn assert_kind(definitions: &[String], kind: Kind) {
    let counts: Vec<_> = counts(definitions, kind).into_values().collect();
    println!(
        "{} members={}, per-member={counts:?}, ceiling={}",
        kind.prefix,
        counts.len(),
        kind.ceiling
    );
    assert!(
        !counts.is_empty(),
        "the probe must drive {} members",
        kind.prefix
    );
    assert!(
        counts
            .iter()
            .all(|count| (1..=kind.ceiling).contains(count))
            && counts.windows(2).all(|pair| pair[0] == pair[1]),
        "{} must retain its measured constant per-member cost",
        kind.prefix
    );
    assert!(
        !definitions.iter().any(|line| line.contains(kind.prefix)
            && [
                "Application<",
                "ExtensionFirst",
                "ExtensionSecond",
                "::Records"
            ]
            .iter()
            .any(|name| line.contains(name))),
        "{} bodies must not multiply by application or fragment",
        kind.prefix
    );
}
pub(super) fn assert_total(definitions: &[String]) {
    let member_budget: usize = KINDS
        .iter()
        .map(|kind| counts(definitions, *kind).len() * kind.ceiling)
        .sum();
    // A symbol mentioning both its relation and payload is attributed once here.
    let core = definitions
        .iter()
        .filter(|line| !KINDS.iter().any(|kind| line.contains(kind.prefix)))
        .count();
    let budget = CORE_SYMBOL_CEILING + member_budget;
    println!("probe text symbols={}, core={core}/{CORE_SYMBOL_CEILING}, member budget={member_budget}, total budget={budget}", definitions.len());
    assert!(
        core <= CORE_SYMBOL_CEILING,
        "shared cores exceeded their calibrated allowance"
    );
    assert!(
        definitions.len() <= budget,
        "symbol cost exceeded the driven-member budget"
    );
}
