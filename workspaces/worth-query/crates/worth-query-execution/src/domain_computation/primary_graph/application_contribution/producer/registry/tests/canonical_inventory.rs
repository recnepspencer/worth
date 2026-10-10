//! Producer declarations order visits; equality alone deduplicates bindings.
use super::super::output_family_inventory::declared_output_family_bindings;
use super::declared;
use std::{any::TypeId, collections::BTreeMap};

struct FirstBinding;
struct SecondBinding;

#[test]
fn binding_visits_follow_declared_producer_order_and_deduplicate_by_equality() {
    let a = TypeId::of::<FirstBinding>();
    let b = TypeId::of::<SecondBinding>();
    let (first, second) = if a > b { (a, b) } else { (b, a) };
    let mut alpha = declared(first);
    alpha.identity = "alpha-producer".into();
    alpha.operation_binding_type = first;
    let mut middle = alpha.clone();
    middle.identity = "middle-producer".into();
    let mut zulu = declared(second);
    zulu.identity = "zulu-producer".into();
    zulu.operation_binding_type = second;
    let declarations = BTreeMap::from([
        (zulu.identity.as_str(), &zulu),
        (middle.identity.as_str(), &middle),
        (alpha.identity.as_str(), &alpha),
    ]);
    let families = declared_output_family_bindings(declarations.values().copied());
    assert_eq!(
        families["family"]
            .iter()
            .map(|(binding, _)| *binding)
            .collect::<Vec<_>>(),
        [first, second],
        "alpha precedes middle and zulu; middle repeats alpha's binding"
    );
}
