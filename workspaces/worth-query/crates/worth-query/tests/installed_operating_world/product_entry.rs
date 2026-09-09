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
    let default = workspace.observe_operating_world().unwrap();
    let held = default.product_branch().unwrap();
    let identity = held.branch_identity().clone();
    let expected = held.selected_commit().clone();
    let component_name = installed::WorthQueryBranchHeadIdentity::new(
        held.relational_basis_descriptor().branch_id().0.clone(),
    )
    .unwrap();

    let denied = workspace
        .observe_branch_operating_world(component_name)
        .err()
        .expect("a component name cannot select installed product authority");
    assert_eq!(
        denied.kind(),
        installed::WorthQueryOperatingWorldEntryDenialKind::Product(
            installed::WorthQueryOperatingWorldProductDenial::SelectionRequired
        )
    );
    let selected = workspace
        .observe_product_operating_world(&identity)
        .unwrap();
    assert!(held.has_same_selected_occurrence(selected.product_branch().unwrap()));
    assert_eq!(
        default.product_branch().unwrap().selected_commit(),
        &expected
    );
}

#[test]
fn product_selection_from_another_runtime_fails_before_binding() {
    let first = workspace("first-product-entry");
    let second = workspace("second-product-entry");
    let first_world = first.observe_operating_world().unwrap();
    let identity = first_world
        .product_branch()
        .unwrap()
        .branch_identity()
        .clone();
    let denied = second
        .observe_product_operating_world(&identity)
        .err()
        .expect("the runtime cannot adopt a foreign World selection");
    assert_eq!(denied.kind(), installed::WorthQueryOperatingWorldEntryDenialKind::Product(installed::WorthQueryOperatingWorldProductDenial::Admission(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::ForeignOwner,
    )));
}
