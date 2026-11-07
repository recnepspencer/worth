use worth_query_decl::facade::application_schema::fragment::{self, *};
#[path = "schema/records.rs"]
mod storage_model;
use super::*;
pub use storage_model::*;
#[path = "node_profile.rs"]
pub mod node_profile;
pub use node_profile::{id, node, value, Id, Node, Value};
pub trait Scenario:
    ApplicationSchema + ApplicationSchemaComposition<Contributions = (Contribution,)>
{
    type Heterogeneous: heterogeneous::Profile;
    type Boundary: sheet_boundary::Profile;
    type NodeProfile: node_profile::NodeProfile<Self>;
    const MODE: u8;
    const COUPLED: bool = matches!(Self::MODE, 6..=12 | 14..=28 | 30 | 39 | 40 | 41 | 42 | 43 | 48 | 60 | 61 | 64 | 66 | 68 | 69 | 70);
    const TWO_BODIES: bool = matches!(
        Self::MODE,
        7 | 8 | 9 | 15..=18 | 20 | 27 | 28 | 42 | 43 | 61 | 64
    );
}
pub struct Model<
    const MODE: u8,
    H = heterogeneous::Absent,
    B = sheet_boundary::Absent,
    const CONTENT: bool = false,
>(PhantomData<(H, B)>);
impl<
        const MODE: u8,
        H: heterogeneous::Profile,
        B: sheet_boundary::Profile,
        const CONTENT: bool,
    > Scenario for Model<MODE, H, B, CONTENT>
where
    node_profile::Profile<CONTENT>: node_profile::NodeProfile<Self>,
{
    type Heterogeneous = H;
    type Boundary = B;
    type NodeProfile = node_profile::Profile<CONTENT>;
    const MODE: u8 = MODE;
}
impl<
        const MODE: u8,
        H: heterogeneous::Profile,
        B: sheet_boundary::Profile,
        const CONTENT: bool,
    > ApplicationSchemaComposition for Model<MODE, H, B, CONTENT>
where
    node_profile::Profile<CONTENT>: node_profile::NodeProfile<Self>,
{
    type Contributions = (Contribution,);
}
impl<
        const MODE: u8,
        H: heterogeneous::Profile,
        B: sheet_boundary::Profile,
        const CONTENT: bool,
    > ApplicationSchema for Model<MODE, H, B, CONTENT>
where
    node_profile::Profile<CONTENT>: node_profile::NodeProfile<Self>,
{
    const OWNER: &'static str = "authored-ownership";
    const NAME: &'static str = "Model";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn visit_correction<V: ApplicationCorrectionVisitor<Self>>(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        match visit_correction::<Self, V>(source, visitor) {
            Ok(output) => output,
            Err(visitor) => visitor.unavailable(),
        }
    }
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        if CONTENT != (MODE == 43) {
            return Err(ApplicationSchemaDeclarationDenial::InvalidRecordMember {
                path: "scenario content profile".into(),
            });
        }
        B::declaration::<Self>()
    }
}

pub struct MapsTo;
pub struct Peer;
impl MapsTo {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Mapping, Node<S>> {
        ApplicationRelationRef::from_schema_identifiers(
            "MapsTo",
            "Mapping",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
impl Peer {
    pub fn reference<S: Scenario>() -> ApplicationRelationRef<S, Self, Node<S>, Node<S>> {
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
) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding> {
    ApplicationPrincipalBindingRef::from_requirements(
        "Actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalId::at("ExternalId"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at(
                "Status",
            )),
            target: ApplicationPrincipalTargetRequirement::from_relation(MapsTo::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                schema::id::<S>(),
            ),
        },
    )
}
pub struct Components;
impl Components {
    pub fn reference<S: Scenario>() -> ApplicationComponentSetRef<S, Self, Node<S>> {
        ApplicationComponentSetRef::new("OwnedComponents", schema::node::<S>())
    }
}
pub struct Contribution<const CONTENT: bool = true>;
impl<S: Scenario, const CONTENT: bool> ApplicationSchemaContribution<S> for Contribution<CONTENT> {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("authored-ownership.contribution.v1");
    type Members = (
        OwnershipModel<CONTENT>,
        super::native_boundary::BoundaryModel<S::Boundary>,
        Contracts,
    );
}
pub struct Contracts;
impl<S: Scenario> Registration<S> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok({
            let schema = lifecycle::declare(producer::declare(
                schema
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
            let schema = currentness_guards::declare(parameter_inverse::declare(
                relation_splice::declare(schema),
            ));
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
        })
    }
}
pub fn visit_correction<S: Scenario, V: ApplicationCorrectionVisitor<S>>(
    source: std::any::TypeId,
    visitor: V,
) -> Result<V::Output, V> {
    if matches!(S::MODE, 48 | 68 | 69 | 70)
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
        if matches!(S::MODE, 48 | 68 | 69 | 70) {
            setup.handler::<parameter_inverse::Binding<S>, _>(parameter_inverse::Handler)?;
        }
        if S::MODE == 70 {
            setup.handler::<foreign_aggregate::BirthBinding<S>, _>(
                foreign_aggregate::command::Handler,
            )?;
            setup.handler::<sheet_boundary::split_walk::ExtendBinding<S>, _>(
                sheet_boundary::split_walk::Handler,
            )?;
        }
        super::owned_action_feature::configure_create::<S>(setup)?;
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

impl fragment::Relation for MapsTo {
    const NAME: &'static str = "MapsTo";
    type To = NativeNode<true>;
}
impl fragment::Relation for Peer {
    const NAME: &'static str = "Peer";
    type To = NativeNode<true>;
}

pub struct OwnershipRoot<S, const CONTENT: bool = true>(pub PhantomData<fn() -> S>);
impl<S: Scenario, const CONTENT: bool> Fragment for OwnershipRoot<S, CONTENT> {
    const NAME: &'static str = "Model";
    type Bindings = NoBindings;
    type Members = (Contribution<CONTENT>,);
}
