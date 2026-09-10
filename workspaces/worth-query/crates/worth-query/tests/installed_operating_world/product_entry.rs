use worth_query::facade::{domain, installed};

use super::{conditional_node_contract, installed_operation_fixture::conditional_workspace};

fn workspace(identity: &str) -> worth_query::facade::runtime::WorthQueryWorkspace {
    let node = conditional_node_contract::node(
        "geometry",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    conditional_workspace(identity, node).unwrap()
}

#[test]
fn installed_product_entry_requires_typed_selection_and_retains_its_observation() {
    let workspace = workspace("typed-product-entry");
    let branch = workspace.current_world();
    let default = workspace.observe_operating_world(branch).unwrap();
    let held = default.product_branch();
    let expected = held.selected_commit().clone();
    let selected = workspace.observe_operating_world(branch).unwrap();
    assert!(held.has_same_selected_occurrence(selected.product_branch()));
    assert_eq!(default.product_branch().selected_commit(), &expected);
}

#[test]
fn product_selection_from_another_runtime_fails_before_binding() {
    let first = workspace("first-product-entry");
    let second = workspace("second-product-entry");
    let branch = first.current_world();
    let denied = second
        .observe_operating_world(branch)
        .err()
        .expect("the runtime cannot adopt a foreign World selection");
    assert_eq!(denied.kind(), installed::WorthQueryOperatingWorldEntryDenialKind::Product(installed::WorthQueryOperatingWorldProductDenial::Admission(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::ForeignOwner,
    )));
}
