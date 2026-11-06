//! Genuine declaration closure for principal-bound authorization identities.
use super::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::{worth_query_aspect, worth_query_entity, worth_query_field};

worth_query_entity!(pub Mapping for Schema: ApplicationSchema);
worth_query_entity!(pub Principal for Schema: ApplicationSchema);
worth_query_aspect!(pub External for Schema: ApplicationSchema, Mapping; identity = AspectIdentity(0x372001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema: ApplicationSchema, Principal; identity = AspectIdentity(0x372002), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema: ApplicationSchema, Mapping, External: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema: ApplicationSchema, Mapping, External: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub Identity for Schema: ApplicationSchema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
struct MappingTarget;
struct PrincipalBinding;
struct ApplyAbility;
struct ApplyPolicy;
impl OperationRequiresAbility<ApplyPreserve> for ApplyAbility {}

pub(super) fn declare<S: ApplicationSchema>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
    paths: Vec<ApplicationAuthorizationPath>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let target =
        ApplicationRelationRef::<S, MappingTarget, Mapping, Principal>::from_schema_identifiers(
            "PrincipalTarget",
            "Mapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        );
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
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalId::reference(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                Status::reference(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                Identity::reference(),
            ),
        },
    );
    let ability = ApplicationAbilityRef::<S, ApplyAbility, Principal>::from_schema_identifiers(
        "Apply",
        "Principal",
    );
    let policy = ApplicationPolicyRef::<S, ApplyPolicy>::from_schema_identifier("ApplyPolicy");
    schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .aspect(Mapping::reference(), External::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), Identity::reference())
        .relation(target, Mapping::reference(), Principal::reference())
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
        super::declaration_for::<Self, 2, 1>(false, paths)
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
                            aspect: "PrincipalFacts".into(),
                            field: "Identity".into(),
                            value: AspectValue::UInt64(value),
                        },
                    )
                })
                .collect(),
        },
    )
}
