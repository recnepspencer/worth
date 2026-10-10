use super::findings;

fn rejects(source: &str) -> Vec<&'static str> {
    let file = crate::source_syntax::parse_file(source).unwrap();
    findings(&[("src/ordinary.rs", &file.items)])
        .into_iter()
        .map(|f| f.reason)
        .collect()
}

#[test]
fn demand_key_derive_and_manual_ordering_cannot_order_build_identity() {
    assert_eq!(
        rejects("#[derive(Ord, PartialOrd)] struct DemandKey { family: std::any::TypeId }"),
        ["ordering derive on a TypeId-bearing type"]
    );
    assert_eq!(
        rejects("struct Key<A> { family: TypeId, value: A } impl<A> Ord for Key<A> { fn cmp(&self, other: &Self) -> Ordering { todo!() } }"),
        ["ordering impl on a TypeId-bearing type"]
    );
    assert_eq!(rejects("struct Key { family: TypeId } impl Ord for Key { fn cmp(&self, other: &Self) -> Ordering { todo!() } } impl PartialOrd for Key { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { todo!() } }"),
        ["ordering impl on a TypeId-bearing type", "ordering impl on a TypeId-bearing type"]);
}

#[test]
fn aliases_and_wrappers_do_not_hide_type_identity() {
    assert_eq!(rejects("use std::any::TypeId as Id; type Token = Id; struct Equality { value: Token } #[derive(Ord)] struct Key { inner: Equality }"),
        ["ordering derive on a TypeId-bearing type"]);
    assert_eq!(
        rejects(
            "use std::collections::BTreeSet as Ordered; type Keys = Ordered<std::any::TypeId>;"
        ),
        ["ordered container keyed by TypeId"]
    );
}

#[test]
fn equality_and_declared_identity_order_are_valid_counterparts() {
    assert!(rejects("#[derive(Eq, PartialEq, Hash)] struct Token { value: TypeId } #[derive(Ord, PartialOrd)] struct Family(&'static str); struct Registry { lookup: HashMap<TypeId, Family> } type Names = BTreeSet<Family>;").is_empty());
}

#[test]
fn containers_need_an_exact_named_membership_contract() {
    let source = "pub trait Roots { fn append_required_bindings(bindings: &mut BTreeSet<TypeId>); fn ordered(bindings: &mut BTreeSet<TypeId>); }";
    let file = crate::source_syntax::parse_file(source).unwrap();
    let found = findings(&[(
        super::MEMBERSHIP_ONLY
            .iter()
            .find(|(_, item, _)| *item == "append_required_bindings")
            .unwrap()
            .0,
        &file.items,
    )]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].item, "ordered");
    assert_eq!(
        rejects("struct Registry { members: BTreeMap<TypeId, usize> }"),
        ["ordered container keyed by TypeId"]
    );
}

#[test]
fn query_workspace_discovery_rejects_the_demand_key_fixture() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/canonical_order");
    let diagnostics = super::validate(&root).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0].subject().contains("lib.rs")
            && diagnostics[0].message().contains("DemandKey")
    );
}

#[test]
fn another_module_cannot_hide_a_type_id_field_with_the_same_type_name() {
    let ordered =
        crate::source_syntax::parse_file("#[derive(Ord)] struct Key { family: TypeId }").unwrap();
    let other = crate::source_syntax::parse_file("#[derive(Ord)] struct Key { name: String } impl PartialOrd for Key { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { todo!() } }").unwrap();
    let found = findings(&[
        ("src/ordered.rs", &ordered.items),
        ("src/other.rs", &other.items),
    ]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].source, "src/ordered.rs");
}
