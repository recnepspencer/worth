//! Genuine public authoring/installation for the ResourceRead admission court.
use worth_query_declaration::facade::{
    application_query::*, application_resource::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type,
    worth_query_principal_binding, worth_query_query_binding, worth_query_relation,
    worth_query_structured_value_binding,
};
use worth_query_installation::facade::*;

worth_query_application_schema! {pub(super) schema Schema {owner: resource_admission_court,
version: (1, 0), members: |schema| {schema.entity(Mapping::reference()).entity(Principal::reference())
    .aspect(Mapping::reference(), MappingFacts::reference()).aspect(Principal::reference(), PrincipalFacts::reference())
    .field(Mapping::reference(), MappingIdentity::reference()).field(Mapping::reference(), MappingStatus::reference())
    .field(Principal::reference(), PrincipalIdentity::reference())
    .relation(Target::reference(), Mapping::reference(), Principal::reference())
    .principal_binding(PrincipalBinding::reference()).application_query(query())
    .application_resource(definition())}}}
worth_query_entity!(pub(super) Mapping for Schema);
worth_query_entity!(pub(super) Principal for Schema);
worth_query_aspect!(pub(super) MappingFacts for Schema, Mapping; identity=AspectIdentity(0x9181_8011), revision=AspectContractRevision(1),);
worth_query_aspect!(pub(super) PrincipalFacts for Schema, Principal; identity=AspectIdentity(0x9181_8012), revision=AspectContractRevision(1),);
worth_query_field!(pub(super) MappingIdentity for Schema, Mapping, MappingFacts:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub(super) MappingStatus for Schema, Mapping, MappingFacts:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub(super) PrincipalIdentity for Schema, Principal, PrincipalFacts:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub(super) Target in Schema, Mapping => Principal; integrity=same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub(super) PrincipalBinding in Schema, mapping Mapping {
    identity: MappingIdentity, status: MappingStatus, target: Target => Principal, principal_identity: PrincipalIdentity});
#[derive(Clone, Copy)]
pub(super) struct ReadIntent;
pub(super) struct Parameters;
pub(super) struct ReadResult;
struct Slot;
worth_query_structured_value_binding!(pub(super) InputBinding for ReadIntent {identity:"worth.court.resource.input"});
worth_query_structured_value_binding!(pub(super) ParametersBinding for Parameters {identity:"worth.court.resource.parameters"});
worth_query_structured_value_binding!(pub(super) ReadResultBinding for ReadResult {identity:"worth.court.resource.read-result"});
worth_query_portable_type!(Slot => "worth.court.resource.slot");
worth_query_application_query!(pub(super) Query for Schema, identity "worth.court.resource.query",
    parameters ParametersBinding, result ReadResultBinding, scope Principal => "Principal", name "ResourceScope");
worth_query_query_binding!(pub(super) Access for ReadIntent, schema Schema,
    identity "worth.court.resource.access", input InputBinding, query Query,
    parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ReadResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalFacts, PrincipalIdentity, u64, ReadOnly, NoApplicationUnit,
    principal_field PrincipalIdentity::reference(), limits results 1);

fn query() -> ApplicationQueryDefinition<Schema, Query, Parameters, ReadResult, Principal> {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Principal,
        ReadResult,
        ReadResultBinding,
    >::new(Principal::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        Slot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("identity", PrincipalIdentity::reference()))
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
        .build()
        .unwrap()
}
pub(super) struct Resource;
pub(super) struct Provider;
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
pub(super) fn installed() -> WorthQueryInstalledApplicationSchema<Schema> {
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
