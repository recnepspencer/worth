use super::*;
use worth_query_decl::facade::application_schema::fragment::*;
mod records;
pub use records::*;
#[derive(Clone, Copy, Debug)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "owned-model";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = (Contribution,);
}
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("owned-model.contribution.v1");
    type Members = (
        Storage,
        navigation::grant::GrantStorage,
        Actor,
        Registrations,
    );
}
impl Contribution {
    pub fn reference() -> ApplicationSchemaContributionRef<Schema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Registrations;
impl Registration<Schema> for Registrations {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(declare(schema))
    }
}
#[derive(Clone, Copy)]
pub struct Actor;
impl Actor {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "Actor",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalId::at("ExternalId"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    Status::at("Status"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalId::at("PrincipalId"),
                ),
            },
        )
    }
}
impl Registration<Schema> for Actor {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}
fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema;
    query::member::declare(query::model::declare(edit::declare(navigation::declare(
        schema,
    ))))
    .declared_operation::<ownership_change::ChangeOwnership>(
        ownership_change::ChangeOwnership::reference()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    )
    .application_mutation_binding::<ownership_change::Binding>()
    .declared_operation::<generation::RecreateMember>(
        generation::RecreateMember::reference()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    )
    .application_mutation_binding::<generation::Binding>()
    .application_history(
        ApplicationHistoryDefinition::<Schema, History>::for_binding(
            ApplicationQueryDisclosureContract::public(),
        )
        .model_domain(ApplicationHistoryModelDomain::OwnedModelGraph)
        .navigation::<navigation::Navigation>()
        .owns(ModelMember::reference())
        .owns(Child::reference())
        .operation::<edit::model::Binding>("Edit model")
        .operation::<ownership_change::Binding>("Move ownership")
        .owned_operation::<edit::member::Binding>("Edit member"),
    )
    .application_history(
        ApplicationHistoryDefinition::<Schema, ResultLimitedHistory>::for_binding(
            ApplicationQueryDisclosureContract::public(),
        )
        .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
        .owns(ModelMember::reference())
        .owns(Child::reference())
        .operation::<edit::model::Binding>("Edit model")
        .owned_operation::<edit::member::Binding>("Edit member"),
    )
}
