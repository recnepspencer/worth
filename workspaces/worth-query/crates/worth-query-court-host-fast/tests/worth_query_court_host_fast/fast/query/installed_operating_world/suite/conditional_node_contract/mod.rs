pub(crate) use worth_query_worlds::facade::query::suite::conditional_node_contract::*;

#[test]
fn installed_conditional_meaning_binds_only_after_exact_lowering_is_present() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = conditional_workspace(
        request_execution,
        "conditional-lowering-not-installed",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .expect("the fixture installs the exact Phase 10 lowering before binding");
    assert!(!bound.binding_identity().is_empty());
}
