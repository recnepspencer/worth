use worth_query_decl::facade::application_schema::fragment::{self, *};
#[path = "schema/records.rs"]
mod storage_model;
use super::*;
pub use storage_model::*;
pub trait Scenario:
    ApplicationSchema + ApplicationSchemaComposition<Contributions = (Contribution,)>
{
    const MODE: u8;
}
pub struct Model<const MODE: u8>;
impl<const MODE: u8> Fragment for Model<MODE> {
    const NAME: &'static str = "Model";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl<const MODE: u8> Scenario for Model<MODE> {
    const MODE: u8 = MODE;
}
impl<const MODE: u8> ApplicationSchemaComposition for Model<MODE> {
    type Contributions = (Contribution,);
}
impl<const MODE: u8> ApplicationSchema for Model<MODE> {
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
        fragment::declare(Self)
    }
}

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
                ExternalId::at("ExternalId"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at(
                "Status",
            )),
            target: ApplicationPrincipalTargetRequirement::from_relation(MapsTo::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(Id::at("Id")),
        },
    )
}
pub struct Contribution;
impl<S: Scenario> ApplicationSchemaContribution<S> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("authored-ownership.contribution.v1");
    type Members = (OwnershipModel, Contracts);
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
        contracts.conditional::<readiness::Readiness>()?;
        Ok(())
    }
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<lifecycle::CreateBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::RetireBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::RestoreBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<lifecycle::PlainBinding<S>, _>(lifecycle_handlers::Handler)?;
        setup.handler::<producer::Binding<S>, _>(producer::Provider)?;
        setup.producer::<producer::Producer<S>>(producer::Provider)?;
        setup.conditional::<readiness::Readiness>(())
    }
}

impl fragment::Relation for MapsTo {
    const NAME: &'static str = "MapsTo";
    type To = Node;
}
impl fragment::Relation for Peer {
    const NAME: &'static str = "Peer";
    type To = Node;
}
