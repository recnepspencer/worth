//! Genuine public authoring/installation for the ResourceRead admission court.
#[path = "resource_fixture/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_declaration::facade::{
    application_query::*, application_resource::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_installation::facade::*;

use worth_query_declaration::facade::application_schema::fragment::{self, *};
#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = NoBindings;
    type Members = (ResourceModel, Contracts);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "resource_admission_court";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl Schema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<Schema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(schema
            .principal_binding(PrincipalBinding::reference())
            .application_query(query())
            .application_resource(definition()))
    }
}

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        Mapping,
        Principal,
        <PrincipalIdentity as RecordSelection>::Value,
        <PrincipalIdentity as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    MappingIdentity::at("MappingIdentity"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(Target::reference()),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

#[derive(Clone, Copy)]
pub struct ReadIntent;
pub struct Parameters;
pub struct ReadResult;
struct Slot;
worth_query_structured_value_binding!(pub InputBinding for ReadIntent {identity:"worth.court.resource.input"});
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"worth.court.resource.parameters"});
worth_query_structured_value_binding!(pub ReadResultBinding for ReadResult {identity:"worth.court.resource.read-result"});
worth_query_portable_type!(Slot => "worth.court.resource.slot");
worth_query_application_query!(pub Query for Schema, identity "worth.court.resource.query",
    parameters ParametersBinding, result ReadResultBinding, scope Principal => "Principal", name "ResourceScope");
worth_query_query_binding!(pub Access for ReadIntent, schema Schema,
    identity "worth.court.resource.access", input InputBinding, query Query,
    parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ReadResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope PrincipalIdentity,
    principal_field PrincipalIdentity::at("PrincipalIdentity"), limits results 1);

fn query() -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, ReadResult, Principal> {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Principal,
        ReadResult,
        ReadResultBinding,
    >::new(Principal::reference())
    .field(ApplicationQueryResultFieldRef::<Query, Slot, _, _>::new(
        "identity",
        PrincipalIdentity::at("PrincipalIdentity"),
    ))
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
pub struct Resource;
pub struct Provider;
worth_query_portable_type!(Resource => "worth.court.resource");
worth_query_portable_type!(Provider => "worth.court.resource.provider");
impl ApplicationResourceBinding<Schema> for Resource {
    type Access = Access;
    type InputBinding = U64ApplicationValueBinding;
    type ResultBinding = U64ApplicationValueBinding;
    type FailureBinding = U64ApplicationValueBinding;
    type Provider = Provider;
    const IDENTITY: &'static str = "Resource";
    fn scope(_: &u64) -> <Access as ApplicationQueryBinding<Schema>>::ScopeBinding {
        ReadIntent.into_scope()
    }
    fn parameters(_: &u64) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
}
fn definition() -> ApplicationResourceDefinition<Schema, Resource> {
    ApplicationResourceDefinition::request_response(
        ApplicationResourceRequestResponseProfile {
            retry_max_attempts: 2,
            retry_delay_milliseconds: 1,
            timeout_milliseconds: 5,
        },
        ApplicationResourceLimits {
            maximum_live: 2,
            maximum_retained_results: 2,
            ingress_slots_per_request: 2,
            maximum_input_bytes: 32,
            maximum_frame_bytes: 32,
        },
        ApplicationResourceAuthorization::public(),
        ApplicationQueryDisclosureContract::public(),
    )
    .unwrap()
}
pub fn installed() -> WorthQueryInstalledApplicationSchema<Schema> {
    let declaration = Schema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "resource_admission_court",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
