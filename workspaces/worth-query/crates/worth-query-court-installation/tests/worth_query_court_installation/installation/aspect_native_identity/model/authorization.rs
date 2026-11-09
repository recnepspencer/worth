//! Genuine declaration closure for principal-bound authorization identities.
use super::*;
use worth_query_declaration::facade::authentication::*;
mod records;
use records::{Authorization, External, Mapping, MappingTarget, Principal, PrincipalFacts};
struct PrincipalBinding;
struct ApplyAbility;
struct ApplyPolicy;
impl OperationRequiresAbility<ApplyPreserve> for ApplyAbility {}

pub(super) fn declare<S: ApplicationSchema>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
    paths: Vec<ApplicationAuthorizationPath>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let target = fragment::relation(Authorization, Mapping, MappingTarget);
    let principal_binding = ApplicationPrincipalBindingRef::<
        S,
        PrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                fragment::record_field(Authorization, Mapping, External, "ExternalId"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                fragment::record_field(Authorization, Mapping, External, "Status"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                Authorization,
                fragment::record_field(Authorization, Principal, PrincipalFacts, "Identity"),
            ),
        },
    );
    let ability = ApplicationAbilityRef::<S, ApplyAbility, Principal>::from_schema_identifiers(
        "Apply",
        "Principal",
    );
    let policy = ApplicationPolicyRef::<S, ApplyPolicy>::from_schema_identifier("ApplyPolicy");
    schema
        .principal_binding(principal_binding)
        .policy(policy)
        .ability(ability)
        .ability_policy(ability, policy, paths)
        .operation_requires_ability(operation(), ability)
}

pub struct Authorized<const MANY: bool>;
impl<const MANY: bool> ApplicationSchema for Authorized<MANY> {
    const OWNER: &'static str = "worth.query.court.authorization-size";
    const NAME: &'static str = "AuthorizationSize";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let paths = if MANY {
            (0..4097).map(|value| path([value])).collect()
        } else {
            vec![path(0..900)]
        };
        let schema = ApplicationSchemaDeclarationBuilder::from_fragment(Authorization)?.operation(
            operation()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        );
        declare(schema, paths).build()
    }
}

fn path(values: impl IntoIterator<Item = u64>) -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPath::from_untrusted_parts(
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: ApplicationAuthorizationPathEffect::Allow,
            principal_entity: "Principal".into(),
            scope_entity: "Principal".into(),
            traversals: Vec::new(),
            predicates: values
                .into_iter()
                .map(|value| {
                    ApplicationAuthorizationPredicate::from_untrusted_parts(
                        WorthQueryPortableApplicationAuthorizationPredicateParts {
                            traversal_ordinal: 0,
                            entity: "Principal".into(),
                            aspect: fragment::record_field::<_, _, _, u64, _, _>(
                                Authorization,
                                Principal,
                                PrincipalFacts,
                                "Identity",
                            )
                            .resolve()
                            .unwrap()
                            .locus()
                            .aspect()
                            .into(),
                            field: "Identity".into(),
                            value: AspectValue::UInt64(value),
                        },
                    )
                })
                .collect(),
        },
    )
}
