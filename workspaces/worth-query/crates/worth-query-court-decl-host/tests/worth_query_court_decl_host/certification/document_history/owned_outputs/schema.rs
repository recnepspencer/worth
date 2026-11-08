//! Its own installed schema: model, owned member and two derived output entities.
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
    const OWNER: &'static str = "owned-outputs";
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
        ApplicationSchemaContributionIdentity::new("owned-outputs.contribution.v1");
    type Members = (
        Storage,
        navigation::grant::GrantStorage,
        Actor,
        Registrations,
        authored_member::correction::UndoRetire,
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
    let schema = predecessor::declare(family::predecessor_family::declare(schema));
    let schema = synchronous_edit::new_source::declare(synchronous_edit::declare(
        authored_member::declare(schema),
    ));
    let schema = query::declare(member_delete::declare(member_edit::declare(edit::declare(
        navigation::declare(schema),
    ))));
    from_first::declare(restricted_from_first::declare(restricted_owned::declare(
        downstream::declare(upstream::declare(restricted::declare(child::declare(
            second::declare(first::declare(permission::declare(schema))),
        )))),
    )))
    .application_history(
        ApplicationHistoryDefinition::<Schema, History>::for_binding(
            ApplicationQueryDisclosureContract::public(),
        )
        .model_domain(ApplicationHistoryModelDomain::OwnedModelGraph)
        .navigation::<navigation::Navigation>()
        .owns(ModelMember::reference())
        .owns(ChildOf::reference())
        .operation::<edit::Binding>("Edit model")
        .owned_operation::<member_edit::Binding>("Edit member")
        .operation::<member_delete::Binding>("Delete member")
        .operation::<authored_member::retire::Binding>("Retire authored member")
        .operation::<authored_member::restore::Binding>("Restore authored member"),
    )
}
