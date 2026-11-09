//! The real schema declaration includes its own typed Source, Index and Parent.
use super::*;
use worth_query_declaration::facade::authentication::*;

pub trait ModelSchema: ApplicationSchema {
    const MODE: u8;
    type Model: ParentModel;
}
pub struct ParentSchema<const MODE: u8>;
impl ModelSchema for ParentSchema<0> {
    const MODE: u8 = 0;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<1> {
    const MODE: u8 = 1;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<2> {
    const MODE: u8 = 2;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<3> {
    const MODE: u8 = 3;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<4> {
    const MODE: u8 = 4;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<5> {
    const MODE: u8 = 5;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<6> {
    const MODE: u8 = 6;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<7> {
    const MODE: u8 = 7;
    type Model = ParentRecords<false>;
}
impl ModelSchema for ParentSchema<8> {
    const MODE: u8 = 8;
    type Model = ParentRecords<true>;
}
impl ModelSchema for ParentSchema<9> {
    const MODE: u8 = 9;
    type Model = ParentRecords<false>;
}
impl<const MODE: u8> ApplicationSchema for ParentSchema<MODE>
where
    Self: ModelSchema,
{
    const OWNER: &'static str = "worth.query.tests.managed-parent";
    const NAME: &'static str = "ParentSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let policy = policy::<Self>();
        let declared = <Self as ModelSchema>::Model::storage::<Self>()?
            .principal_binding(principal_binding::<Self>())
            .policy(policy)
            .ability(ability::<Self>())
            .ability_policy(
                ability::<Self>(),
                policy,
                [
                    ApplicationAuthorizationPathBuilder::from_principal(principal::<Self>())
                        .allow(principal::<Self>()),
                ],
            )
            .application_query(queries::source::<Self>())
            .application_query_binding::<queries::SourceBinding<Self>>()
            .application_query(queries::parent::<Self>())
            .application_query_binding::<queries::ParentBinding<Self>>()
            .managed_index(ApplicationManagedIndexDefinition::<
                Self,
                queries::ModelIndex<Self>,
            >::new());
        super::decision_read::declare(declared).build()
    }
}
use worth_query_declaration::facade::application_schema::fragment::{self, *};
mod records;
pub use records::*;
mod profile;
pub use profile::*;
pub struct PrincipalBinding;
pub fn principal_binding<S: ModelSchema>() -> ApplicationPrincipalBindingRef<
    S,
    PrincipalBinding,
    Mapping<S>,
    Principal<S>,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                S::Model::mapping_identity(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                S::Model::mapping_status(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(mapping_target::<S>()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                principal_identity::<S>(),
            ),
        },
    )
}
pub struct CloseAssessment;
pub struct ManagedParentPolicy;
pub fn ability<S: ModelSchema>() -> ApplicationAbilityRef<S, CloseAssessment, Principal<S>> {
    ApplicationAbilityRef::from_schema_identifiers("CloseAssessment", "Principal")
}
pub fn policy<S: ModelSchema>() -> ApplicationPolicyRef<S, ManagedParentPolicy> {
    ApplicationPolicyRef::from_schema_identifier("ManagedParentPolicy")
}
