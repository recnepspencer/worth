//! The canonical package receiver keeps ordinary Source and Parent basis contracts distinct.
use super::*;
use crate::application_query::*;
use crate::application_schema::*;
use crate::portable_identity::WorthQueryPortableTypeIdentity as Id;
crate::worth_query_application_query!(ParentQuery for TestSchema,
    identity "worth.tests.source-authorization.parent.v1", parameters AccountQueryParametersBinding,
    result AccountQueryResultBinding, scope Account => "Account", name "indexed_account");
#[derive(worth_foundational::facade::WorthValue)]
struct Probe {
    prefix: String,
}
crate::worth_query_value_binding!(ProbeBinding for Probe {
    identity: "worth.tests.source-authorization.probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});
fn members_with_source(
    ability: bool,
    basis: ApplicationQueryBasisSupport,
) -> Vec<ApplicationSchemaMember> {
    let mut source = query("AccountId", "id").into_parts();
    source.authorization = if ability {
        ApplicationQueryAuthorizationRequirement::Ability {
            ability: "ViewAccount".into(),
            scope_entity: "Account".into(),
        }
    } else {
        ApplicationQueryAuthorizationRequirement::Public
    };
    let binding = ApplicationQueryBindingReference::from_untrusted_parts(
        ApplicationQueryBindingReferenceParts {
            binding: Id::declared("worth.tests.source-authorization.binding.v1"),
            input: Id::declared("worth.tests.source-authorization.input.v1"),
            query_name: source.name.clone(),
            query: source.query_type.clone(),
            parameters: source.parameter_type.clone(),
            result: source.result_type.clone(),
            scope: source.scope_type.clone(),
            scope_entity: "Account".into(),
            scope_aspect: "AccountFacts".into(),
            scope_field: "AccountId".into(),
            scope_value: Id::declared("worth.rust.u64"),
            scope_resolution: ApplicationQueryScopeResolutionMode::PrincipalIdentity,
            scope_writable: false,
            principal_binding: "Principal".into(),
        },
    );
    let probe = ApplicationManagedIndexProbeMeaning::of::<ProbeBinding>();
    let index = ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
        WorthQueryPortableManagedIndexParts {
            semantic_version: 3,
            name: "AccountIndex".into(),
            provider: "worth.tests.source-authorization.provider.v1".into(),
            provider_version: 1,
            structure: ApplicationManagedIndexSourceStructure::BinarySourceTree,
            membership: ApplicationManagedIndexMembership::CheckedSummarySourceTree,
            source: ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(binding),
            entity: "Account".into(),
            probe: probe.clone(),
        },
    );
    let mut parent = source.clone();
    parent.name = "indexed_account".into();
    parent.query_type = Id::declared("worth.tests.source-authorization.parent.v1");
    let account = ApplicationEntityRef::<TestSchema, Account>::from_schema_identifier("Account");
    let field = ApplicationFieldRef::<
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_identifiers("Account", "AccountFacts", "AccountId");
    parent.result_shape = ApplicationQueryResultShapeBuilder::<
        TestSchema,
        ParentQuery,
        Account,
        AccountResult,
        AccountQueryResultBinding,
    >::new(account)
    .field(ApplicationQueryResultFieldRef::<
        ParentQuery,
        AccountIdSlot,
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::new("id", field))
    .build()
    .into_erased();
    parent.authorization = ApplicationQueryAuthorizationRequirement::Public;
    parent.basis_support = basis;
    parent.parameters = vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
        "probe".into(),
        probe.parts().scalar_family,
        probe.parts().binding.clone(),
        None,
        None,
    )];
    parent.managed_index = Some(ApplicationQueryManagedIndexMeaning::from_untrusted_parts(
        WorthQueryPortableQueryManagedIndexParts {
            semantic_version: 1,
            index: "AccountIndex".into(),
            probe_parameter: "probe".into(),
        },
    ));
    let mut members = dependencies();
    if ability {
        members.push(ApplicationSchemaMember::Ability {
            ability: "ViewAccount".into(),
            scope_entity: "Account".into(),
        });
        members.push(ApplicationSchemaMember::AbilityPolicy {
            ability: "ViewAccount".into(),
            scope_entity: "Account".into(),
            policy: "AccountVisibility".into(),
            paths: vec![],
        });
    }
    members.extend([
        ApplicationSchemaMember::ApplicationQuery {
            definition: ErasedApplicationQueryDefinition::from_untrusted_parts(source),
        },
        ApplicationSchemaMember::ManagedIndex { definition: index },
        ApplicationSchemaMember::ApplicationQuery {
            definition: ErasedApplicationQueryDefinition::from_untrusted_parts(parent),
        },
    ]);
    members
}
#[test]
fn ability_source_keeps_its_ordinary_pinned_contract_but_requires_current_only_parent() {
    let current = members_with_source(true, ApplicationQueryBasisSupport::current_only());
    assert_eq!(validate_application_query_members(&current), Ok(()));
    let ApplicationSchemaMember::ApplicationQuery { definition: source } =
        &current[current.len() - 3]
    else {
        unreachable!()
    };
    assert!(
        source.basis_support().pinned(),
        "the existing ordinary Source contract was preserved"
    );
    let pinned = members_with_source(true, ApplicationQueryBasisSupport::current_and_pinned());
    assert!(matches!(
        validate_application_query_members(&pinned),
        Err(ApplicationSchemaDeclarationDenial::InvalidManagedIndex {
            cause: ApplicationManagedIndexDefinitionDenial::UnsupportedParentBasis,
            ..
        })
    ));
    let public = members_with_source(false, ApplicationQueryBasisSupport::current_and_pinned());
    assert_eq!(
        validate_application_query_members(&public),
        Ok(()),
        "existing Public Source pinned Parents stay supported"
    );
}
