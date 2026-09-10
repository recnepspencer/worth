use worth_query::facade::domain;

use super::conditional_node_contract::node;
use super::installed_operation_fixture::{
    conditional_controlled_workspace, conditional_workspace, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

#[test]
fn reconstituted_runtime_handles_reenter_the_retained_conditional_source() {
    let declaration = node(
        "reconstituted-conditional-source",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut workspace =
        conditional_controlled_workspace("reconstituted-conditional-source", declaration).unwrap();
    let prior = workspace.domain(GeometryDomain).unwrap();
    workspace.advance_domain_installation_generation().unwrap();
    workspace.advance_domain_installation_generation().unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    assert!(installed.installation_generation() > prior.installation_generation());
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    assert!(world.family(ReadFamily).bind(&prior, ReadVertex).is_err());
    let bound = world
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let executed = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(executed.conditional_provenance().len(), 1);
    assert_eq!(
        executed.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged,
    );
    assert_eq!(executed.counters().conditional_compute_contacts, 1);
    assert_eq!(executed.counters().executor_contacts, 1);
}

#[test]
fn rebuilt_conditional_lookup_retains_the_exact_installed_authority() {
    let declaration = node(
        "rebuilt-conditional-index",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut workspace = conditional_workspace("rebuilt-conditional-index", declaration).unwrap();

    let installed = workspace.domain(GeometryDomain).unwrap();
    let before_rebuild = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();

    let report = workspace.rebuild_conditional_execution_index();
    assert_eq!(report.authoritative_installations(), 1);
    assert_eq!(report.rebuilt_lookup_entries(), 1);
    assert!(report.exact_index_parity());

    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    before_rebuild.same_installation_with(&bound).unwrap();
    let executed = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();

    assert_eq!(executed.conditional_provenance().len(), 1);
    assert_eq!(
        executed.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(
        executed.conditional_provenance()[0]
            .declaration()
            .identity(),
        "rebuilt-conditional-index"
    );
    assert_eq!(executed.counters().conditional_compute_contacts, 1);
    assert_eq!(executed.counters().executor_contacts, 1);
}
