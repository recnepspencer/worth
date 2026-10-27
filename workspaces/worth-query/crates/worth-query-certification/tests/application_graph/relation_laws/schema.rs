//! The same graph vocabulary with separately authored relation meaning.
use std::marker::PhantomData;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
use worth_query_host::facade::declaration::{application_schema::*, authentication::*};

pub trait LawPolicy: Clone + std::fmt::Debug + Send + Sync + 'static {
    const INTEGRITY: ApplicationRelationIntegrity;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
}
#[derive(Clone, Debug)]
pub struct Acyclic;
impl LawPolicy for Acyclic {
    // Self endpoints are permitted here so acyclicity itself must judge them.
    // compiled-example: begin
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().acyclic();
    // compiled-example: end
}
#[derive(Clone, Debug)]
pub struct Unrestricted;
impl LawPolicy for Unrestricted {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
}

#[derive(Clone, Debug)]
pub struct Orbit;
impl LawPolicy for Orbit {
    // compiled-orbit-example: begin
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit();
    // compiled-orbit-example: end
}
#[derive(Clone, Debug)]
pub struct NoSelfOrbit;
impl LawPolicy for NoSelfOrbit {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
            .orbit();
}

#[derive(Clone, Debug)]
pub struct Symmetric;
impl LawPolicy for Symmetric {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().symmetric();
}
#[derive(Clone, Debug)]
pub struct DirectedUnique;
impl LawPolicy for DirectedUnique {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
}
#[derive(Clone, Debug)]
pub struct NormalizedUnique;
impl LawPolicy for NormalizedUnique {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .unique(ApplicationRelationUniquenessScope::NormalizedSymmetricEdge);
}
#[path = "schema/paired_unique.rs"]
mod paired_unique;
pub use paired_unique::SymmetricDirectedUnique;

pub trait GraphSchemaBinding: ApplicationSchema {
    const INTEGRITY: ApplicationRelationIntegrity;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity;
}
#[derive(Clone, Debug)]
pub struct GraphSchema<L: LawPolicy>(PhantomData<fn() -> L>);
impl<L: LawPolicy> GraphSchemaBinding for GraphSchema<L> {
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
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .contributions()
            .register::<super::program::GraphContribution<L>>()?
            .build()
    }
}
impl<L: LawPolicy> ApplicationSchemaComposition for GraphSchema<L> {
    type Contributions = (super::program::GraphContribution<L>,);
}

worth_query_entity!(pub ExternalMapping for S: GraphSchemaBinding);
worth_query_entity!(pub Principal for S: GraphSchemaBinding);
worth_query_entity!(pub Node for S: GraphSchemaBinding);
worth_query_entity!(pub Leaf for S: GraphSchemaBinding);
worth_query_aspect!(pub ExternalIdentity for S: GraphSchemaBinding, ExternalMapping; identity = AspectIdentity(0x91810101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for S: GraphSchemaBinding, Principal; identity = AspectIdentity(0x91810102), revision = AspectContractRevision(1));
worth_query_aspect!(pub LeafFacts for S: GraphSchemaBinding, Leaf; identity = AspectIdentity(0x91810104), revision = AspectContractRevision(1));
worth_query_field!(pub LeafKey for S: GraphSchemaBinding, Leaf, LeafFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_aspect!(pub NodeFacts for S: GraphSchemaBinding, Node; identity = AspectIdentity(0x91810103), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for S: GraphSchemaBinding, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for S: GraphSchemaBinding, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalKey for S: GraphSchemaBinding, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub NodeKey for S: GraphSchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_only, equality);

pub struct Precedes;
impl Precedes {
    pub const fn reference<S: GraphSchemaBinding>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers("Precedes", "Node", "Node", S::INTEGRITY)
    }
}
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
                    ExternalIdentityField::reference(),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::reference(),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalKey::reference(),
                ),
            },
        )
    }
}

#[path = "schema/partition.rs"]
mod partition;
pub use partition::{CrossPartition, Isolated};

pub struct Feeds;
impl Feeds {
    pub const fn reference<S: GraphSchemaBinding>() -> ApplicationRelationRef<S, Self, Node, Leaf> {
        ApplicationRelationRef::from_schema_identifiers("Feeds", "Node", "Leaf", S::FEEDS_INTEGRITY)
    }
}
#[path = "schema/minimum.rs"]
mod minimum;
pub use minimum::{LeafAndPrincipalMinimumTwo, LeafMinimumTwo, MinimumTwo};
