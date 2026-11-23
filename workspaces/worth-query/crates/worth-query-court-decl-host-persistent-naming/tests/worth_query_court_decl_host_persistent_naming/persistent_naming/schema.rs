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
    const OWNER: &'static str = "persistent-naming";
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
/// The declared persistent-name scheme every `Member` birth is minted under.
pub const MEMBER_SCHEME: &str = "worth.tests.persistent-naming.member.v1";
/// The declared persistent-name scheme every `Seam` birth is minted under.
pub const SEAM_SCHEME: &str = "worth.tests.persistent-naming.seam.v1";
impl Registration<Schema> for Registrations {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        use worth_query_decl::facade::application_naming::ApplicationNameSchemeBuilder;
        let scheme = ApplicationNameSchemeBuilder::<Schema, Member>::new(MEMBER_SCHEME)
            .role_order(&["authored", "piece", "merged"])
            .role_depends_on("piece", "authored")
            .split_discriminator::<outputs::MemberCut>()
            .require_authored_occurrence()
            .build()
            .expect("the member scheme is well-formed");
        let seam = ApplicationNameSchemeBuilder::<Schema, Seam>::for_relation(
            SEAM_SCHEME,
            Seam::reference(),
        )
        // A seam may tell itself apart by a member authored beside it.
        .role_order(&["authored", "seam", "seam_piece"])
        .role_depends_on("seam", "authored")
        .split_discriminator::<outputs::SeamSide>()
        .require_authored_occurrence()
        .build()
        .expect("the seam scheme is well-formed");
        declare(schema)
            .application_name_scheme::<Member>(scheme)?
            .application_name_scheme::<Seam>(seam)
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
    outputs::sculpt::declare(refusals::declare(probe::declare(delete::declare(query::member::declare(
        query::model::declare(edit::declare(navigation::declare(schema))),
    )))))
    .application_history(
        ApplicationHistoryDefinition::<Schema, History>::for_binding(
            ApplicationQueryDisclosureContract::public(),
        )
        .model_domain(ApplicationHistoryModelDomain::OwnedModelGraph)
        .navigation::<navigation::Navigation>()
        .owns(ModelMember::reference())
        .owns(Child::reference())
        .operation::<edit::model::Binding>("Edit model")
        .operation::<delete::Binding>("Delete member")
        .operation::<probe::Binding>("Probe names")
        .operation::<outputs::sculpt::Binding>("Sculpt members")
        .owned_operation::<edit::member::Binding>("Edit member"),
    )
}
