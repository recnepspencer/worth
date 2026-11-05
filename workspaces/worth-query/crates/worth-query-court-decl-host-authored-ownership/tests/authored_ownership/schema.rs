use super::*;
pub trait Scenario:
    ApplicationSchema + ApplicationSchemaComposition<Contributions = (Contribution,)>
{
    type Heterogeneous: heterogeneous::Profile;
    type Boundary: sheet_boundary::Profile;
    const MODE: u8;
    const COUPLED: bool = matches!(Self::MODE, 6..=12 | 14..=28 | 30 | 39 | 40 | 41 | 42 | 43 | 48 | 60 | 61 | 64 | 66 | 68 | 69);
    const TWO_BODIES: bool = matches!(
        Self::MODE,
        7 | 8 | 9 | 15..=18 | 20 | 27 | 28 | 42 | 43 | 61 | 64
    );
}
pub struct Model<const MODE: u8, H = heterogeneous::Absent, B = sheet_boundary::Absent>(
    PhantomData<(H, B)>,
);
impl<const MODE: u8, H: heterogeneous::Profile, B: sheet_boundary::Profile> Scenario
    for Model<MODE, H, B>
{
    type Heterogeneous = H;
    type Boundary = B;
    const MODE: u8 = MODE;
}
impl<const MODE: u8, H: heterogeneous::Profile, B: sheet_boundary::Profile>
    ApplicationSchemaComposition for Model<MODE, H, B>
{
    type Contributions = (Contribution,);
}
impl<const MODE: u8, H: heterogeneous::Profile, B: sheet_boundary::Profile> ApplicationSchema
    for Model<MODE, H, B>
{
    const OWNER: &'static str = "authored-ownership";
    const NAME: &'static str = "Model";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn visit_correction<V: ApplicationCorrectionVisitor<Self>>(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        match <Contribution as ApplicationSchemaContribution<Self>>::visit_correction(
            source, visitor,
        ) {
            Ok(output) => output,
            Err(visitor) => visitor.unavailable(),
        }
    }
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .contributions()
            .register::<Contribution>()?
            .build()
    }
}
worth_query_entity!(pub Mapping for S: Scenario);
worth_query_entity!(pub Node for S: Scenario);
worth_query_aspect!(pub MappingFacts for S: Scenario, Mapping; identity = AspectIdentity(0x919B3901), revision = AspectContractRevision(1));
worth_query_aspect!(pub Facts for S: Scenario, Node; identity = AspectIdentity(0x919B3902), revision = AspectContractRevision(1));
worth_query_aspect!(pub Values for S: Scenario, Node; identity = AspectIdentity(0x919B3903), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for S: Scenario, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for S: Scenario, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub Id for S: Scenario, Node, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Value for S: Scenario, Node, Values: u64 => U64ApplicationValueBinding, read_write, equality);
pub struct MapsTo;
pub struct Peer;
impl MapsTo {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Mapping, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "MapsTo",
            "Mapping",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl Peer {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "Peer",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct Actor;
pub fn actor<S: Scenario>(
) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node, u64, U64ApplicationValueBinding> {
    ApplicationPrincipalBindingRef::from_requirements(
        "Actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalId::reference(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                Status::reference(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(MapsTo::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(Id::reference()),
        },
    )
}
pub struct Components;
impl Components {
    pub fn reference<S: Scenario>() -> ApplicationComponentSetRef<S, Self, Node> {
        ApplicationComponentSetRef::new("OwnedComponents", Node::reference())
    }
}
pub struct Contribution;
impl<S: Scenario> ApplicationSchemaContribution<S> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("authored-ownership.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        let schema = lifecycle::declare(producer::declare(
            schema
                .entity(Mapping::reference())
                .entity(Node::reference())
                .aspect(Mapping::reference(), MappingFacts::reference())
                .aspect(Node::reference(), Facts::reference())
                .optional_aspect(Node::reference(), Values::reference())
                .field(Mapping::reference(), ExternalId::reference())
                .field(Mapping::reference(), Status::reference())
                .unique_field(Node::reference(), Id::reference())
                .field(Node::reference(), Value::reference())
                .relation(MapsTo::reference(), Mapping::reference(), Node::reference())
                .relation(Peer::reference(), Node::reference(), Node::reference())
                .principal_binding(actor())
                .application_query(query::definition())
                .application_query_binding::<query::Binding<S>>(),
        ))
        .correction::<correction::UndoCreate<S>>();
        let schema = if matches!(S::MODE, 13 | 14) {
            schema.components(
                ApplicationComponentSetDefinition::new(Components::reference())
                    .coupled_by(Peer::reference()),
            )
        } else {
            schema
        };
        let schema = sheet_boundary::declare(heterogeneous::declare(content::declare(
            stage_schema::declare(schema),
        )));
        let schema = parameter_inverse::declare(relation_splice::declare(schema));
        let schema = projected_memberships::schema::declare(schema);
        let schema = if S::COUPLED {
            coupled::producer::declare(schema)
        } else {
            schema
        };
        if S::MODE == 4 {
            // The inverse Retire has no correction declaration: only the
            // prepared retire_created correction can supply its anchor.
            schema.declared_operation(
                lifecycle::RetireOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
        } else {
            schema.correction::<correction::UndoRetire<S>>()
        }
    }
    fn visit_correction<V: ApplicationCorrectionVisitor<S>>(
        source: std::any::TypeId,
        visitor: V,
    ) -> Result<V::Output, V> {
        if matches!(S::MODE, 48 | 68 | 69)
            && source == std::any::TypeId::of::<parameter_inverse::Binding<S>>()
        {
            return Ok(dispatch_application_correction::<
                S,
                parameter_inverse::Inverse<S>,
                V,
            >(visitor));
        }
        if source == std::any::TypeId::of::<lifecycle::CreateBinding<S>>() {
            return Ok(dispatch_application_correction::<
                S,
                correction::UndoCreate<S>,
                V,
            >(visitor));
        }
        if S::MODE != 4 && source == std::any::TypeId::of::<lifecycle::RetireBinding<S>>() {
            return Ok(dispatch_application_correction::<
                S,
                correction::UndoRetire<S>,
                V,
            >(visitor));
        }
        Err(visitor)
    }
}
impl<S: Scenario> WorthQueryApplicationContribution<S> for Contribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<producer::Producer<S>>()?;
        if S::COUPLED {
            contracts.producer::<coupled::producer::Producer<S>>()?;
            contracts.conditional::<coupled::producer::readiness::Readiness>()?;
        }
        contracts.conditional::<readiness::Readiness>()?;
        Ok(())
    }
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if matches!(S::MODE, 30 | 39 | 40 | 41 | 60) {
            setup.handler::<relation_splice::inspection::Binding<S>, _>(
                relation_splice::inspection::Handler,
            )?;
        }
        if S::MODE == 29 {
            setup.handler::<projected_memberships::inspection::Binding<S>, _>(
                projected_memberships::inspection::Handler,
            )?;
        }
        if matches!(S::MODE, 48 | 68 | 69) {
            setup.handler::<parameter_inverse::Binding<S>, _>(parameter_inverse::Handler)?;
        }
        setup.handler::<lifecycle::CreateBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::RetireBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::RestoreBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::PlainBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<producer::Binding<S>, _>(producer::Provider)?;
        setup.producer::<producer::Producer<S>>(producer::Provider)?;
        if S::COUPLED {
            setup.handler::<coupled::producer::Binding<S>, _>(coupled::producer::Provider)?;
            setup.producer::<coupled::producer::Producer<S>>(coupled::producer::Provider)?;
            setup.conditional::<coupled::producer::readiness::Readiness>(())?;
        }
        setup.conditional::<readiness::Readiness>(())
    }
}
