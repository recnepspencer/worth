pub use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryResultFieldRef,
    ApplicationQueryResultShapeBuilder,
};
pub use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, Aspect, BindingSet, Entity, Fragment, Registration, Scalar},
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ValueDecodeAdmission,
};
pub use worth_query_declaration::worth_query_application_query;
pub use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledGraphObligationKind,
    WorthQueryInstalledGraphObligationOwner, WorthQueryInstalledGraphObligationSelectionBasis,
    WorthQueryInstalledGraphObligationTerminalRequirement, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub use worth_query_host::facade::inspect_installed_graph_obligations;

pub struct HostileConsumerSchema;
impl Fragment for HostileConsumerSchema {
    const NAME: &'static str = "HostileConsumerSchema";
    type Bindings = Bindings;
    type Members = (Record, RecordQuery);
}
impl ApplicationSchema for HostileConsumerSchema {
    const OWNER: &'static str = "hostile_consumer";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct Record;
impl Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = (RecordFacts,);
    type Relations = ();
}
pub struct RecordFacts;
impl Aspect for RecordFacts {
    const NAME: &'static str = "RecordFacts";
    type Row = u64;
}
impl Registration<HostileConsumerSchema> for RecordQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostileConsumerSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostileConsumerSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(record_query_definition()))
    }
}

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
    let identity: fragment::FieldSelection<_, _, _, u64> =
        fragment::select_field(HostileConsumerSchema, Record, RecordFacts, "value");
    let shape = ApplicationQueryResultShapeBuilder::<
        HostileConsumerSchema,
        RecordQuery,
        Record,
        RecordQueryResult,
        RecordQueryResultBinding,
    >::new(fragment::entity(HostileConsumerSchema, Record))
    .field(ApplicationQueryResultFieldRef::<
        RecordQuery,
        RecordIdentitySlot,
        _,
        _,
    >::new("identity", identity))
    .build();
    ApplicationQueryDefinitionBuilder::declare(RecordQuery::reference())
        .root(fragment::entity(HostileConsumerSchema, Record))
        .scope(fragment::entity(HostileConsumerSchema, Record))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("hostile-consumer query declaration should be valid")
}
