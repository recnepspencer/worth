use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field,
};
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledGraphObligationKind,
    WorthQueryInstalledGraphObligationOwner, WorthQueryInstalledGraphObligationSelectionBasis,
    WorthQueryInstalledGraphObligationTerminalRequirement, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_host::facade::inspect_installed_graph_obligations;

worth_query_application_schema! {
    pub schema HostileConsumerSchema {
        owner: hostile_consumer,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Record::reference())
                .aspect(Record::reference(), RecordFacts::reference())
                .field(Record::reference(), RecordIdentity::reference())
                .application_query(record_query_definition())
        }
    }
}

worth_query_entity!(pub Record for HostileConsumerSchema);
worth_query_aspect!(pub RecordFacts for HostileConsumerSchema, Record; identity = AspectIdentity(0x91611041), revision = AspectContractRevision(1),);
worth_query_field!(
    pub RecordIdentity for HostileConsumerSchema, Record, RecordFacts:
    u64, read_only, equality
);

struct RecordQueryParameters;
struct RecordQueryResult;
struct RecordIdentitySlot;
worth_query_declaration::worth_query_structured_value_binding!(RecordQueryParametersBinding for RecordQueryParameters { identity: "RecordQueryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(RecordQueryResultBinding for RecordQueryResult { identity: "worth.query.test.host.record_query.result.v1" });
worth_query_declaration::worth_query_portable_type!(
    RecordQueryResult => "worth.query.test.host.record_query.result.v1"
);
worth_query_declaration::worth_query_portable_type!(
    RecordIdentitySlot => "worth.query.test.host.record_query.identity_slot.v1"
);

worth_query_application_query!(
    RecordQuery for HostileConsumerSchema,
    identity "RecordQuery",
    parameters RecordQueryParametersBinding,
    result RecordQueryResultBinding,
    scope Record => "Record",
    name "record_query"
);

fn record_query_definition() -> ApplicationQueryDefinition<
    HostileConsumerSchema,
    RecordQuery,
    RecordQueryParameters,
    RecordQueryResult,
    Record,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        HostileConsumerSchema,
        RecordQuery,
        Record,
        RecordQueryResult,
        RecordQueryResultBinding,
    >::new(Record::reference())
    .field(ApplicationQueryResultFieldRef::<
        RecordQuery,
        RecordIdentitySlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("identity", RecordIdentity::reference()))
    .build();
    ApplicationQueryDefinitionBuilder::declare(RecordQuery::reference())
        .root(Record::reference())
        .scope(Record::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("hostile-consumer query declaration should be valid")
}

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
        .application_query(RecordQuery::reference())
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
