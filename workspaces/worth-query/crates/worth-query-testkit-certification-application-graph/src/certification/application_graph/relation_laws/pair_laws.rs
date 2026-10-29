//! Proposed physical edge counts prove paired symmetry and both uniqueness scopes.
pub use super::{installation::*, outcome::*, read, schema::*};
pub use std::collections::BTreeMap;
pub use worth_query_host::facade::{
    application_invariants::{EntityReference, SymmetryMode, UniquenessScope},
    primary_graph::*,
};

pub fn symmetry(
    denial: &WorthQueryApplicationCommitDenial,
    graph: &read::Graph,
    expected: &[(u64, u64)],
) {
    let Some(WorthQueryRelationLawDenial::Symmetry {
        contract_id,
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        mode,
        ..
    }) = denial.relation_law()
    else {
        panic!("original typed symmetry edge required: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:symmetry");
    assert_eq!(mode, SymmetryMode::PairedTwinRequired);
    assert!(expected
        .iter()
        .any(|&(a, b)| graph.identities[&a] == *source && graph.identities[&b] == *target));
}

pub fn uniqueness(
    denial: &WorthQueryApplicationCommitDenial,
    graph: &read::Graph,
    pair: (u64, u64),
    expected_scope: UniquenessScope,
) {
    let Some(WorthQueryRelationLawDenial::Uniqueness {
        contract_id,
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        count,
        scope,
        ..
    }) = denial.relation_law()
    else {
        panic!("original typed duplicate pair required: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:unique");
    assert_eq!(scope, expected_scope);
    assert_eq!(count, 2);
    assert!(
        (graph.identities[&pair.0] == *source && graph.identities[&pair.1] == *target)
            || (graph.identities[&pair.1] == *source && graph.identities[&pair.0] == *target)
    );
}
