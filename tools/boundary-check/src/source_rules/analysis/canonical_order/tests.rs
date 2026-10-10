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

#[test]
fn manual_order_fixture_refuses_the_impl() {
    assert_eq!(
        rejects(include_str!(
            "../../../../tests/fixtures/canonical_order_rules/manual_order.rs"
        )),
        ["ordering impl on a TypeId-bearing type"]
    );
}
#[test]
fn aliased_container_fixture_refuses_the_transitive_key() {
    assert_eq!(
        rejects(include_str!(
            "../../../../tests/fixtures/canonical_order_rules/aliased_container.rs"
        )),
        ["ordered container keyed by TypeId"]
    );
}
#[test]
fn expression_fixture_refuses_each_deciding_call() {
    let found = rejects(include_str!(
        "../../../../tests/fixtures/canonical_order_rules/expression_order.rs"
    ));
    assert_eq!(
        found.len(),
        5,
        "all deciding expressions must be refused: {found:?}"
    );
    assert!(found
        .into_iter()
        .all(|reason| reason == "ordering expression names TypeId"));
}

#[test]
fn binary_search_and_partial_cmp_refuse_type_identity() {
    for source in [
        include_str!("../../../../tests/fixtures/canonical_order_rules/binary_search.rs"),
        include_str!("../../../../tests/fixtures/canonical_order_rules/partial_cmp.rs"),
    ] {
        assert_eq!(rejects(source), ["ordering expression names TypeId"]);
    }
}

#[test]
fn named_identity_lookup_orders_the_declaration() {
    assert!(rejects(include_str!(
        "../../../../tests/fixtures/canonical_order_rules/named_identity.rs"
    ))
    .is_empty());
}

#[test]
fn every_runtime_owner_is_in_scope() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/canonical_order");
    let mut governed =
        super::super::workspace_crates::discover_workspace_crates(&root, "workspaces/worth-query")
            .unwrap()
            .remove(0);
    let graph = super::super::crate_modules::parse_crate_modules(&governed).unwrap();
    for package in super::CRATES {
        governed.package = (*package).to_owned();
        assert_eq!(super::enforce(&governed, &graph).len(), 1, "{package}");
    }
    governed.package = "worth-signal".to_owned();
    assert!(super::enforce(&governed, &graph).is_empty());
}
