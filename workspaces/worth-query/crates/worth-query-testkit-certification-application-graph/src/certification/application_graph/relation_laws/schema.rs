//! The same graph vocabulary with separately authored relation meaning.
pub use std::marker::PhantomData;
pub use worth_query_decl::facade::application_schema::fragment::{self, *};
#[path = "schema/records.rs"]
mod records;
pub use records::*;
#[path = "schema/policy_models.rs"]
mod policy_models;
pub use policy_models::*;
pub use worth_query_host::facade::declaration::{application_schema::*, authentication::*};

pub trait LawPolicy: Clone + std::fmt::Debug + Send + Sync + 'static {
    type Model: GraphModel;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>;
    const INTEGRITY: ApplicationRelationIntegrity;
    const RELATION_PAYLOAD: bool = false;
    const DERIVED_RELATIONS: bool = false;
    const LOGICAL_RELATIONS: bool = false;
    const CUSTOM_DERIVED_OWNER: bool = false;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    fn observation_members(
        schema: ApplicationSchemaDeclarationBuilder<GraphSchema<Self>>,
    ) -> ApplicationSchemaDeclarationBuilder<GraphSchema<Self>> {
        schema
    }
}
#[derive(Clone, Debug)]
pub struct Acyclic;
impl LawPolicy for Acyclic {
    type Model = AcyclicRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    // Self endpoints are permitted here so acyclicity itself must judge them.
    // compiled-example: begin
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().acyclic();
    // compiled-example: end
}
#[derive(Clone, Debug)]
pub struct Unrestricted;
impl LawPolicy for Unrestricted {
    type Model = UnrestrictedRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
}

#[derive(Clone, Debug)]
pub struct Orbit;
impl LawPolicy for Orbit {
    type Model = OrbitRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    // compiled-orbit-example: begin
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit();
    // compiled-orbit-example: end
}
#[derive(Clone, Debug)]
pub struct NoSelfOrbit;
impl LawPolicy for NoSelfOrbit {
    type Model = NoSelfOrbitRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
            .orbit();
}

#[derive(Clone, Debug)]
pub struct Symmetric;
impl LawPolicy for Symmetric {
    type Model = SymmetricRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().symmetric();
}
#[derive(Clone, Debug)]
pub struct DirectedUnique;
impl LawPolicy for DirectedUnique {
    type Model = DirectedUniqueRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
}
#[derive(Clone, Debug)]
pub struct NormalizedUnique;
impl LawPolicy for NormalizedUnique {
    type Model = NormalizedUniqueRecords;
    fn schema_declaration(
    ) -> Result<ApplicationSchemaDeclaration<GraphSchema<Self>>, ApplicationSchemaDeclarationDenial>
    {
        fragment::declare(GraphSchema::<Self>(PhantomData))
    }

    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .unique(ApplicationRelationUniquenessScope::NormalizedSymmetricEdge);
}
#[path = "schema/paired_unique.rs"]
pub mod paired_unique;
pub use paired_unique::SymmetricDirectedUnique;

pub trait GraphSchemaBinding: ApplicationSchema {
    type Model: GraphModel;
    const INTEGRITY: ApplicationRelationIntegrity;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity;
}
#[derive(Clone, Debug)]
pub struct GraphSchema<L: LawPolicy>(pub PhantomData<fn() -> L>);
impl<L: LawPolicy> GraphSchemaBinding for GraphSchema<L> {
    type Model = L::Model;
    const INTEGRITY: ApplicationRelationIntegrity = L::INTEGRITY;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity = L::FEEDS_INTEGRITY;
}
impl<L: LawPolicy> ApplicationSchema for GraphSchema<L> {
    const OWNER: &'static str = "worth.query.relation-law-court";
    const NAME: &'static str = "RelationGraph";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        L::schema_declaration()
    }
}
impl<L: LawPolicy> Fragment for GraphSchema<L> {
    const NAME: &'static str = "RelationGraph";
    type Bindings = NoBindings;
    type Members = (super::program::GraphContribution<L>,);
}
impl<L: LawPolicy> ApplicationSchemaComposition for GraphSchema<L> {
    type Contributions = (super::program::GraphContribution<L>,);
}

pub struct Precedes;

pub struct MappingTarget;
impl MappingTarget {
    pub fn reference<S: GraphSchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, ExternalMapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "ExternalMapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct GraphPrincipal;
impl GraphPrincipal {
    pub fn reference<S: GraphSchemaBinding>() -> ApplicationPrincipalBindingRef<
        S,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "GraphPrincipal",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalKey::at("PrincipalKey"),
                ),
            },
        )
    }
}

#[path = "schema/partition.rs"]
pub mod partition;
pub use partition::{CrossPartition, Isolated};

pub struct Feeds;

#[path = "schema/minimum.rs"]
pub mod minimum;
pub use minimum::{LeafAndPrincipalMinimumTwo, LeafMinimumTwo, MinimumTwo};

#[path = "schema/payload.rs"]
pub mod payload;
pub use payload::{EdgeFacts, EdgeWeight, Payload};

#[path = "schema/model.rs"]
mod model;
pub use model::*;

#[path = "schema/extension_payload.rs"]
mod extension_payload;
pub use extension_payload::ExtensionPayload;

#[path = "schema/deletion.rs"]
pub mod deletion;
pub use deletion::DeletionProtected;
