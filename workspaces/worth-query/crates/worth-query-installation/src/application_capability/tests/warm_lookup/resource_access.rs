//! Reuse the real installed capability/principal fixture for resource purposes.
use super::*;
use crate::facade::{
    WorthQueryInstalledGraphAuthorizationRequirement as Authorization,
    WorthQueryResourceAccessPurpose as Purpose,
};
use worth_query_declaration::facade::{application_query::*, application_resource::*};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};

#[derive(Clone, Copy)]
pub(super) struct ReadIntent;
pub(super) struct Parameters;
pub(super) struct QueryResult;
struct IdentitySlot;
impl worth_query_declaration::facade::application_schema::RequiredApplicationFieldValue
    for PrincipalIdentity
{
}
worth_query_structured_value_binding!(pub(super) InputBinding for ReadIntent {identity: "worth.test.resource.access-input"});
worth_query_structured_value_binding!(pub(super) ParametersBinding for Parameters {identity: "worth.test.resource.parameters"});
worth_query_structured_value_binding!(pub(super) ResultBinding for QueryResult {identity: "worth.test.resource.query-result"});
worth_query_portable_type!(IdentitySlot => "worth.test.resource.identity-slot");
worth_query_application_query!(pub(super) AccessQuery for Schema,
    identity "worth.test.resource.access-query", parameters ParametersBinding,
    result ResultBinding, scope Principal => "Principal", name "ResourceAccess");
impl PrincipalBinding {
    fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        Grant,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        principal_binding()
    }
}
worth_query_query_binding!(pub(super) Access for ReadIntent, schema Schema,
    identity "worth.test.resource.access-binding", input InputBinding,
    query AccessQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal PrincipalBinding, mapping Grant, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalFacts, PrincipalIdentity, u64, ReadOnly,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
    principal_field principal_identity_field(), limits results 1);

pub(super) fn query(
) -> ApplicationQueryDefinition<Schema, AccessQuery, Parameters, QueryResult, Principal> {
    let principal = ApplicationEntityRef::<Schema, Principal>::from_schema_identifier("Principal");
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        AccessQuery,
        Principal,
        QueryResult,
        ResultBinding,
    >::new(principal)
    .field(ApplicationQueryResultFieldRef::<
        AccessQuery,
        IdentitySlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("identity", principal_identity_field()))
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccessQuery::reference())
        .root(principal)
        .scope(principal)
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
worth_query_portable_type!(Resource => "worth.test.governed-resource");
worth_query_portable_type!(Provider => "worth.test.governed-resource-provider");
impl ApplicationResourceBinding<Schema> for Resource {
    type Access = Access;
    type InputBinding = U64ApplicationValueBinding;
    type ResultBinding = U64ApplicationValueBinding;
    type FailureBinding = U64ApplicationValueBinding;
    type Provider = Provider;
    const IDENTITY: &'static str = "GovernedResource";
    fn scope(_: &u64) -> <Access as ApplicationQueryBinding<Schema>>::ScopeBinding {
        ReadIntent.into_scope()
    }
    fn parameters(_: &u64) -> ApplicationQueryParameterSet<AccessQuery> {
        ApplicationQueryParameterSet::new()
    }
}

pub(super) fn definition() -> ApplicationResourceDefinition<Schema, Resource> {
    let mut disclosure = ApplicationQueryDisclosureContract::governed_by(
        "resource",
        ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
    );
    for facet in ApplicationResourceDisclosureFacet::ALL {
        disclosure = disclosure.disclose_resource_by(
            ApplicationResourceDisclosureRef::<Resource>::new(facet),
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap(),
            ApplicationQueryInfluenceContract::permit(facet.required_influences().iter().copied()),
        );
    }
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
        disclosure,
    )
    .unwrap()
}

fn installed() -> crate::facade::WorthQueryInstalledApplicationSchema<Schema> {
    let declaration = Schema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth-query-tests",
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

#[test]
fn installed_resource_purposes_select_exact_action_and_disclosure_facets() {
    let schema = installed();
    let resource = schema.installed_resource::<Resource>().unwrap();
    for purpose in Purpose::ALL {
        let contract = resource.access_contract(purpose);
        let rows = contract.graph_obligations();
        assert_eq!(
            rows.subject_kind(),
            crate::facade::WorthQueryInstalledGraphObligationSubjectKind::ApplicationResource
        );
        assert_eq!(contract.purpose(), purpose);
        assert_eq!(contract.disclosure_rules().len(), purpose.facets().len());
        for rule in contract.disclosure_rules() {
            let ApplicationQueryDisclosureSelector::Resource(meaning) = rule.selector() else {
                panic!("real resource facet required")
            };
            assert!(purpose.facets().contains(&meaning.facet()));
        }
        let capability_rows = rows
            .rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.authorization_requirement(),
                    Some(Authorization::Capabilities(_))
                )
            })
            .count();
        assert_eq!(capability_rows, usize::from(!purpose.facets().is_empty()));
        assert!(rows.rows().iter().all(|row| !matches!(
            row.selection_basis(),
            crate::facade::WorthQueryInstalledGraphObligationSelectionBasis::ApplicationQueryGraph(
                _
            )
        )));
    }
    assert!(resource
        .access_contract(Purpose::Close)
        .disclosure_rules()
        .is_empty());
    assert_eq!(
        Purpose::CloseDetail.action(),
        ApplicationResourceAction::Close
    );
    assert_ne!(
        resource
            .access_contract(Purpose::Close)
            .graph_obligations()
            .identity(),
        resource
            .access_contract(Purpose::CloseDetail)
            .graph_obligations()
            .identity()
    );
    let foreign = installed();
    let foreign = foreign.installed_resource::<Resource>().unwrap();
    assert!(!foreign.owns_access_contract(resource.access_contract(Purpose::Close)));
}
