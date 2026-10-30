pub use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryResultFieldRef,
    ApplicationQueryResultShapeBuilder,
};
pub use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
pub use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field,
};
pub use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledGraphObligationKind,
    WorthQueryInstalledGraphObligationOwner, WorthQueryInstalledGraphObligationSelectionBasis,
    WorthQueryInstalledGraphObligationTerminalRequirement, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub use worth_query_host::facade::inspect_installed_graph_obligations;

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
    u64 => U64ApplicationValueBinding, read_only, equality
);

pub struct RecordQueryParameters;
pub struct RecordQueryResult;
pub struct RecordIdentitySlot;
worth_query_declaration::worth_query_structured_value_binding!(pub RecordQueryParametersBinding for RecordQueryParameters { identity: "RecordQueryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub RecordQueryResultBinding for RecordQueryResult { identity: "worth.query.test.host.record_query.result.v1" });
worth_query_declaration::worth_query_portable_type!(
    RecordQueryResult => "worth.query.test.host.record_query.result.v1"
);
worth_query_declaration::worth_query_portable_type!(
    RecordIdentitySlot => "worth.query.test.host.record_query.identity_slot.v1"
);

worth_query_application_query!(
    pub RecordQuery for HostileConsumerSchema,
    identity "RecordQuery",
    parameters RecordQueryParametersBinding,
    result RecordQueryResultBinding,
    scope Record => "Record",
    name "record_query"
);

pub fn record_query_definition() -> ApplicationQueryDefinition<
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
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("hostile-consumer query declaration should be valid")
}
