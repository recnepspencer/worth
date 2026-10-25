use super::fixture_values::PositiveLength;
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, ApplicationRelationCardinality,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
    ApplicationRelationRef, RelationalInvariantWorkBudget,
};
use worth_query_decl::facade::{worth_query_aspect, worth_query_field};

use super::{Body, Metre, TopologyLengthBinding, TopologyModelBinding};

worth_query_aspect!(
    pub PlanarPosition for Schema: TopologyModelBinding, Body;
    identity = AspectIdentity(0x9174_1011),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub BodyKey for Schema: TopologyModelBinding, Body, PlanarPosition:
    String => worth_query_decl::facade::application_schema::StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub PositionX for Schema: TopologyModelBinding, Body, PlanarPosition:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);
worth_query_field!(
    pub PositionY for Schema: TopologyModelBinding, Body, PlanarPosition:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarSuccessor;

impl PlanarSuccessor {
    pub const fn reference<Schema>() -> ApplicationRelationRef<Schema, Self, Body, Body>
    where
        Schema: TopologyModelBinding,
    {
        ApplicationRelationRef::from_schema_identifiers(
            "PlanarSuccessor",
            "Body",
            "Body",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(
                    Some(1),
                    Some(1),
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ),
                ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
            )
            .reaches_at_least(2, const { &[Body::reference::<Schema>().name()] }),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UndeclaredPlanarRelation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarDiscoverySource;

impl PlanarDiscoverySource {
    pub const fn reference<Schema>() -> ApplicationRelationRef<Schema, Self, Body, Body>
    where
        Schema: TopologyModelBinding,
    {
        ApplicationRelationRef::from_schema_identifiers(
            "PlanarDiscoverySource",
            "Body",
            "Body",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(None, None, None, None, None, None),
                ApplicationRelationDeletionPolicy::CascadeDeleteRelations,
            ),
        )
    }
}

impl UndeclaredPlanarRelation {
    pub const fn reference<Schema>() -> ApplicationRelationRef<Schema, Self, Body, Body>
    where
        Schema: TopologyModelBinding,
    {
        ApplicationRelationRef::from_schema_identifiers(
            "UndeclaredPlanarRelation",
            "Body",
            "Body",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(None, None, None, None, None, None),
                ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositivePlanarTurn;

impl<Schema> ApplicationInvariantMarkerIdentity<Schema> for PositivePlanarTurn
where
    Schema: TopologyModelBinding,
{
    const IDENTIFIER: &'static str = "PositivePlanarTurn";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

// Eight rings of three bodies/successors; price root comparisons and six
// coordinate reads per body (18 aspect + 18 kind visits), with 4x headroom.
const PLANAR_SCOPE_RINGS: u64 = 8;
const PLANAR_SCOPE_BODIES: u64 = PLANAR_SCOPE_RINGS * 3;
const PLANAR_SCOPE_SUCCESSORS: u64 = PLANAR_SCOPE_BODIES;
const PLANAR_CHECKED_HEADROOM: u64 = 4;
pub(crate) const PLANAR_CHECKED_WORK: u64 = PLANAR_CHECKED_HEADROOM
    * (PLANAR_SCOPE_BODIES * 3 * PLANAR_SCOPE_BODIES
        + PLANAR_SCOPE_BODIES * 6 * (18 + 18)
        + PLANAR_SCOPE_SUCCESSORS * 8)
    * super::planar_invariant::PLANAR_CANDIDATE_WORK;

pub fn planar_turn_invariant<Schema>() -> ApplicationInvariantDefinition<Schema, PositivePlanarTurn>
where
    Schema: TopologyModelBinding,
{
    ApplicationInvariantDefinition::new(
        PositivePlanarTurn::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Schema::planar_invariant_work()),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [
                ApplicationInvariantGroup::SchemaCompliance,
                ApplicationInvariantGroup::RelationIntegrity,
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Body".to_owned()),
                ApplicationInvariantScopeTarget::Relation("PlanarSuccessor".to_owned()),
            ],
            [ApplicationInvariantScopeTarget::Entity("Body".to_owned())],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
