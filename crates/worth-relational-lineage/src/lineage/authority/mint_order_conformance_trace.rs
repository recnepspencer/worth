//! `dependency_order` on real birth requests as a conformance trace for the
//! proved Lean model (`proofs/lean/persistent_naming`, `MintOrder`).
//!
//! Each graph's births declare same-occurrence references to one another
//! (repeated, to themselves, and to a name outside the commit included);
//! records are allocated against label order. Each line records the births
//! in canonical order, every birth's in-commit dependencies as canonical
//! positions, and the order `dependency_order` mints them in. With
//! `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` the lines are written to
//! `<dir>/mint_order.ndjson`, which `lake exe conformance` checks against
//! the proved `dependencyOrder`.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::contracts::identity::data::primitives::{LineageId, PartitionId};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::{
    InstalledPersistentNameSchemeRegistry, LineageRecordTarget, PersistentNameBirthRequest,
    PersistentNameDiscriminatorReference, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameReferenceBasis,
};

use super::{canonical_order, dependency_order, pending_identity};

fn target(slot: u64) -> LineageRecordTarget {
    LineageRecordTarget::Entity(EntityId::new(PartitionId::main(), slot, 1))
}

fn request(label: usize) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: "worth.trace.mint-order.v1".to_owned(),
        occurrence_key: "occ".to_owned(),
        role_path: format!("r{label}"),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: vec![label as u8],
        holder: PersistentNameHolder::Lineage(LineageId(0)),
    }
}

fn reference(identity: PersistentNameIdentity) -> PersistentNameDiscriminatorReference {
    PersistentNameDiscriminatorReference {
        identity,
        basis: PersistentNameReferenceBasis::SameOccurrence {
            role: "r".to_owned(),
        },
    }
}

/// One graph's trace line: `deps[label]` lists the labels it references.
fn trace_graph(deps: &[&[usize]]) -> String {
    let registry = InstalledPersistentNameSchemeRegistry::new();
    let births = deps.len();
    let outside = pending_identity(&registry, &request(99));
    let mut requests: Vec<_> = (0..births).map(request).collect();
    let ids: Vec<_> = requests
        .iter()
        .map(|birth| pending_identity(&registry, birth))
        .collect();
    for (label, wanted) in deps.iter().enumerate() {
        let wanted = wanted.iter().map(|d| ids[*d]).chain([outside]);
        requests[label].referenced_names = wanted.map(reference).collect();
    }
    // Records are allocated against label order: allocation never decides.
    let slot = |label: usize| (births - label) as u64;
    let named: BTreeMap<_, _> = requests
        .into_iter()
        .enumerate()
        .map(|(label, birth)| (target(slot(label)), birth))
        .collect();
    let position: BTreeMap<_, _> = canonical_order(&registry, &named)
        .iter()
        .enumerate()
        .map(|(at, (record, _))| (*record, at))
        .collect();
    let mut by_position = vec![Vec::new(); births];
    for (label, wanted) in deps.iter().enumerate() {
        by_position[position[&target(slot(label))]] =
            wanted.iter().map(|d| position[&target(slot(*d))]).collect();
    }
    let order: Vec<usize> = dependency_order(&registry, &named)
        .iter()
        .map(|(record, _)| position[record])
        .collect();
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        (0..births).collect::<Vec<_>>(),
        "every birth mints once"
    );
    format!(r#"{{"kind":"mint_order","births":{births},"deps":{by_position:?},"order":{order:?}}}"#)
}

#[test]
fn mint_order_follows_dependencies_then_canonical_order_against_the_proved_model() {
    let graphs: [&[&[usize]]; 5] = [
        // A diamond and a chain against canonical order.
        &[&[3], &[0], &[0], &[], &[1, 2], &[]],
        // A two-birth cycle, a birth stuck behind it, and free births.
        &[&[], &[2], &[1], &[1], &[0]],
        // Repeated and self references count once and never.
        &[&[1, 1, 0], &[], &[0, 1, 1], &[2, 2]],
        // No births, and births without in-commit dependencies.
        &[],
        &[&[], &[], &[]],
    ];
    let mut trace = String::new();
    for deps in graphs {
        let _ = writeln!(trace, "{}", trace_graph(deps));
    }
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/mint_order.ndjson"), trace).unwrap();
    }
}
