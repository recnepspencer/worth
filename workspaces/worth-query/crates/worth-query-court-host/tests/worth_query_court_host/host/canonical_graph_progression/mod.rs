pub(crate) use worth_query_testkit_host::facade::host::canonical_graph_progression::*;

#[test]
fn external_host_consumer_has_one_obligation_and_one_graph_read_planning_path() {
    let declaration = HostileConsumerSchema::declaration()
        .expect("hostile-consumer application schema should declare");
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "hostile_consumer",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("hostile-consumer package should validate");
    let admitted = WorthQueryInstallationAdmissionProfile::new("host", "hostile-consumer")
        .admit(package)
        .expect("hostile-consumer package should admit");
    let installed = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("hostile-consumer package should install");
    let schema = installed
        .bind_application_schema(declaration)
        .expect("hostile-consumer schema should bind");
    let query = schema
        .certification_query(RecordQuery::reference())
        .expect("hostile-consumer query should be installed");

    let adoption =
        inspect_installed_graph_obligations("hostile-external-consumer", query.graph_obligations())
            .expect("read-only public adoption should accept installation evidence");
    assert_eq!(adoption.rows().len(), 1);
    assert_eq!(adoption.selector_index_entries(), 5);
    assert_eq!(
        adoption.rows()[0].kind(),
        WorthQueryInstalledGraphObligationKind::GraphRead
    );
    assert_eq!(
        adoption.rows()[0].required_owners(),
        &[WorthQueryInstalledGraphObligationOwner::RelationalGraph]
    );
    assert_eq!(
        adoption.rows()[0].terminal_requirement(),
        WorthQueryInstalledGraphObligationTerminalRequirement::GraphReadProduct
    );

    let installed_obligations = query.graph_obligations();
    let [installed_row] = installed_obligations.rows() else {
        panic!("the ordinary query must install exactly one obligation row")
    };
    let WorthQueryInstalledGraphObligationSelectionBasis::ApplicationQueryGraph(bound_graph) =
        installed_row.selection_basis()
    else {
        panic!("the graph-read obligation must bind the installed application-query graph")
    };
    assert_eq!(
        bound_graph.canonical_planning_basis().digest(),
        query.read_graph().canonical_planning_basis().digest(),
        "inspection and execution planning must share one canonical installed graph"
    );
}
