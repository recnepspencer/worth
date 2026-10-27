//! The real installed principal/query fixture supplies resource recipe authority.
use super::*;
use crate::facade::{
    WorthQueryApplicationResourceInstallationDenialKind as Kind,
    WorthQueryInstalledApplicationSchemaDenialKind as SchemaKind,
};

struct CloseAssessment;
struct ResourcePolicy;
fn close_ability() -> worth_query_declaration::facade::application_schema::ApplicationAbilityRef<
    IdentitySchema,
    CloseAssessment,
    Principal,
> {
    worth_query_declaration::facade::application_schema::ApplicationAbilityRef::from_schema_identifiers("CloseAssessment", "Principal")
}

pub(super) fn with_authorization(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<IdentitySchema>,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    IdentitySchema,
> {
    use worth_query_declaration::facade::application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationPolicyRef,
    };
    let policy = ApplicationPolicyRef::<IdentitySchema, ResourcePolicy>::from_schema_identifier(
        "ResourcePolicy",
    );
    schema
        .policy(policy)
        .ability(close_ability())
        .ability_policy(
            close_ability(),
            policy,
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .allow(Principal::reference()),
            ],
        )
}
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryParameterSet,
    },
    application_resource::*,
    application_schema::{
        validate_portable_application_schema_freshly, ApplicationReadableScalarValueBinding,
        ApplicationScalarValueBinding, ValueDecodeBounds,
        WorthQueryPortableApplicationSchemaRecord,
    },
    portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity},
};

#[derive(worth_foundational::facade::WorthValue)]
pub struct AssessmentResult {
    value: u64,
    label: String,
}
worth_query_declaration::worth_query_value_binding!(pub AssessmentResultBinding for AssessmentResult {
    identity: "worth.query.tests.resource-result.v1", scalar: String,
    canonical: ValueDecodeBounds::new(2048, 1024, 8, 4),
});
pub struct AssessmentProvider;
worth_query_portable_type!(AssessmentProvider => "worth.query.tests.resource-provider.v1");
pub struct Assessment<ResultBinding>(PhantomData<fn() -> ResultBinding>);
impl<ResultBinding: 'static> WorthQueryPortableType for Assessment<ResultBinding> {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.tests.resource.v1";
    const PORTABLE_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared("worth.query.tests.resource.v1");
}
impl<ResultBinding: ApplicationReadableScalarValueBinding>
    ApplicationResourceBinding<IdentitySchema> for Assessment<ResultBinding>
{
    type Access = PrincipalQueryBinding;
    type InputBinding = U64ApplicationValueBinding;
    type ResultBinding = ResultBinding;
    type FailureBinding = StringApplicationValueBinding;
    type Provider = AssessmentProvider;
    const IDENTITY: &'static str = "Assessment";
    fn scope(_: &u64) -> <Self::Access as ApplicationQueryBinding<IdentitySchema>>::ScopeBinding {
        PrincipalQueryInput.into_scope()
    }
    fn parameters(_: &u64) -> ApplicationQueryParameterSet<PrincipalQuery> {
        ApplicationQueryParameterSet::new()
    }
}

pub(super) fn definition(
) -> ApplicationResourceDefinition<IdentitySchema, Assessment<AssessmentResultBinding>> {
    definition_for::<AssessmentResultBinding>().unwrap()
}

fn definition_for<ResultBinding: ApplicationReadableScalarValueBinding>() -> Result<
    ApplicationResourceDefinition<IdentitySchema, Assessment<ResultBinding>>,
    ApplicationResourceDefinitionDenial,
> {
    ApplicationResourceDefinition::request_response(
        ApplicationResourceRequestResponseProfile {
            retry_max_attempts: 3,
            retry_delay_milliseconds: 2,
            timeout_milliseconds: 5,
        },
        ApplicationResourceLimits {
            maximum_live: 2,
            maximum_retained_results: 2,
            ingress_slots_per_request: 2,
            maximum_input_bytes: 32,
            maximum_frame_bytes: 2048,
        },
        ApplicationResourceAuthorization::public()
            .require(ApplicationResourceAction::Close, close_ability()),
        ApplicationQueryDisclosureContract::public(),
    )
}

#[test]
fn installed_resource_resolves_actual_access_and_codec_not_copied_names() {
    let schema = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let resource = schema
        .installed_resource::<Assessment<AssessmentResultBinding>>()
        .unwrap();
    assert_eq!(resource.binding_identity(), &schema.binding_identity());
    assert_eq!(resource.access_binding().query().name(), "principal_query");
    assert_eq!(resource.definition().parts().limits.maximum_live, 2);
    let actual = AssessmentResultBinding::canonical_contract().unwrap();
    assert!(resource
        .definition()
        .parts()
        .result
        .parts()
        .canonical
        .as_ref()
        .unwrap()
        .matches_contract(&actual));
    for action in [
        ApplicationResourceAction::Start,
        ApplicationResourceAction::Inspect,
        ApplicationResourceAction::Advance,
    ] {
        assert!(resource.action_requirement(action).is_none());
    }
    let close = resource
        .action_requirement(ApplicationResourceAction::Close)
        .unwrap();
    assert_eq!(close.ability(), "CloseAssessment");
    assert_eq!(close.scope_entity(), "Principal");
    assert_eq!(close.policy(), "ResourcePolicy");
    let refusal = schema
        .installed_resource::<Assessment<U64ApplicationValueBinding>>()
        .err()
        .unwrap();
    assert_eq!(
        refusal.kind(),
        Kind::BindingMeaningChanged,
        "identical resource/provider names cannot replace the declared result recipe"
    );
}

#[test]
fn cold_resource_shape_and_bounds_cannot_replace_actual_typed_schema() {
    for change_shape in [true, false] {
        let declaration = IdentitySchema::declaration().unwrap();
        let mut record =
            WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
        let member = record.members.iter_mut().find(|member| matches!(member,
            worth_query_declaration::facade::application_schema::ApplicationSchemaMember::ApplicationResource { .. })).unwrap();
        let worth_query_declaration::facade::application_schema::ApplicationSchemaMember::ApplicationResource { definition } = member
            else { unreachable!() };
        let mut parts = definition.parts().clone();
        let mut result = parts.result.parts().clone();
        let mut canonical = result.canonical.as_ref().unwrap().parts().clone();
        if change_shape {
            canonical.shape = r#"{"Primitive":"U64"}"#.to_owned();
        } else {
            canonical.maximum_wire_bytes = Some(4096);
        }
        result.canonical = Some(worth_query_declaration::facade::application_schema::ApplicationCanonicalValueReference::from_untrusted_parts(canonical));
        parts.result = ApplicationResourceValueBindingReference::from_untrusted_parts(result);
        *definition = ErasedApplicationResourceDefinition::from_untrusted_parts(parts);
        record.members.sort();
        let cold = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(record),
        )
        .unwrap();
        assert_ne!(cold.identity(), declaration.identity());
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "identity_installation_test",
            1,
            0,
        ))
        .application_schema_erased(cold)
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let index = WorthQueryInstalledPackageIndex::build(
            WorthQueryInstallationRuntimeIdentity::fresh(),
            WorthQueryInstallationGeneration::initial(),
            [admitted],
        )
        .unwrap();
        assert_eq!(
            index
                .bind_application_schema(declaration)
                .unwrap_err()
                .kind(),
            SchemaKind::SchemaMeaningChanged
        );
    }
}

#[test]
fn resource_rejects_missing_admitted_codec_and_zero_capacity_before_installation() {
    let mut definition = definition().into_erased().parts().clone();
    let mut value = definition.result.parts().clone();
    value.supports_admitted_decode = false;
    definition.result = ApplicationResourceValueBindingReference::from_untrusted_parts(value);
    assert_eq!(
        ErasedApplicationResourceDefinition::from_untrusted_parts(definition).validate(),
        Err(ApplicationResourceDefinitionDenial::UnsupportedValueProfile)
    );
    let mut definition = self::definition().into_erased().parts().clone();
    definition.limits.maximum_live = 0;
    assert_eq!(
        ErasedApplicationResourceDefinition::from_untrusted_parts(definition).validate(),
        Err(ApplicationResourceDefinitionDenial::InvalidLimits)
    );
}

#[test]
fn ordinary_query_cannot_borrow_resource_payload_disclosure_slot() {
    use worth_query_declaration::facade::application_query::{
        ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureRule,
        ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
        WorthQueryPortableApplicationQueryDisclosureParts,
    };
    let mut query = principal_query_definition().into_erased().into_parts();
    query.disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Public,
            classification: "public".to_owned(),
            capability_name: None,
            capability_type: None,
            rules: vec![ApplicationQueryDisclosureRule::from_untrusted_fields(
                ApplicationQueryDisclosureSelector::Resource(
                    ApplicationResourceDisclosureRef::<Assessment<AssessmentResultBinding>>::new(
                        ApplicationResourceDisclosureFacet::Result,
                    )
                    .meaning(),
                ),
                AspectValue::Bool(true),
                ApplicationQueryInfluenceContract::forbid_all(),
            )],
        },
    );
    let declaration = IdentitySchema::declaration().unwrap();
    let mut record =
        WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
    let member = record.members.iter_mut().find(|member| matches!(member,
        worth_query_declaration::facade::application_schema::ApplicationSchemaMember::ApplicationQuery { .. })).unwrap();
    *member = worth_query_declaration::facade::application_schema::ApplicationSchemaMember::ApplicationQuery {
        definition: worth_query_declaration::facade::application_query::ErasedApplicationQueryDefinition::from_untrusted_parts(query),
    };
    record.members.sort();
    assert_eq!(validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(record)).unwrap_err(),
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
}
