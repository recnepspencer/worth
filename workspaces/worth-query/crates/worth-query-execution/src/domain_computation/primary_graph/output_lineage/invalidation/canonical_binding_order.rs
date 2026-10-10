//! The fixture names are declared independently of Rust build identity.
use crate::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FactPosting;
use std::{any::TypeId, collections::BTreeMap};

struct A;
struct B;
struct C;
struct D;

fn reverse_bindings() -> [(TypeId, &'static str); 2] {
    let declared = [
        (TypeId::of::<A>(), "alpha-binding"),
        (TypeId::of::<B>(), "bravo-binding"),
        (TypeId::of::<C>(), "charlie-binding"),
        (TypeId::of::<D>(), "delta-binding"),
    ];
    for (i, left) in declared.iter().enumerate() {
        for right in &declared[i + 1..] {
            if left.0 > right.0 {
                return [*left, *right];
            }
        }
    }
    panic!("fixture needs a declared-order inversion in this build");
}

#[test]
fn postings_and_prior_output_visits_follow_declared_binding_order() {
    let bindings = reverse_bindings();
    assert!(bindings[0].0 > bindings[1].0);
    assert!(bindings[0].1 < bindings[1].1);
    let (mut lineage, seed) = super::super::registry_fixture::recorded_settlement();
    lineage.by_source.clear();
    let mut postings = im::OrdSet::new();
    for (binding, name) in bindings.into_iter().rev() {
        let mut source = seed.source().clone();
        source.output_binding = lineage.fixture_binding(binding, name);
        lineage.by_source.insert(source.clone(), BTreeMap::new());
        postings.insert(FactPosting {
            settlement: super::super::RecordedSettlementIdentity::retain(
                &source,
                super::super::ProductCoordinate {
                    occurrence: seed.address().0,
                    generation: 1,
                },
                0,
            ),
            ordinal: 0,
        });
    }
    let expected = [bindings[0].1, bindings[1].1];
    let posting_names: Vec<_> = postings
        .iter()
        .map(|p| p.settlement.source().output_binding.as_str())
        .collect();
    assert_eq!(posting_names, expected);
    // checkpoint_prior_outputs walks this exact owner index's keys and charges
    // the selected source's partition navigation before the next source visit.
    let visits: Vec<_> = lineage
        .by_source
        .keys()
        .map(|source| source.output_binding.as_str())
        .collect();
    assert_eq!(visits, expected);
}
