use std::num::NonZeroU64;

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, ApplicationRelationCardinality,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
    ApplicationRelationRef, RelationalInvariantWorkBudget,
};
use worth_query_decl::facade::{worth_query_aspect, worth_query_field};

use super::{Body, Metre, TopologyLengthBinding, TopologySchemaBinding};

worth_query_aspect!(
    pub PlanarPosition for Schema: TopologySchemaBinding, Body;
    identity = AspectIdentity(0x9174_1011),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub BodyKey for Schema: TopologySchemaBinding, Body, PlanarPosition:
    String => worth_query_decl::facade::application_schema::StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub PositionX for Schema: TopologySchemaBinding, Body, PlanarPosition:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);
worth_query_field!(
    pub PositionY for Schema: TopologySchemaBinding, Body, PlanarPosition:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarSuccessor;

impl PlanarSuccessor {
    pub const fn reference<Schema>() -> ApplicationRelationRef<Schema, Self, Body, Body>
    where
        Schema: TopologySchemaBinding,
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
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositivePlanarTurn;

impl<Schema> ApplicationInvariantMarkerIdentity<Schema> for PositivePlanarTurn
where
    Schema: TopologySchemaBinding,
{
    const IDENTIFIER: &'static str = "PositivePlanarTurn";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn planar_turn_invariant<Schema>() -> ApplicationInvariantDefinition<Schema, PositivePlanarTurn>
where
    Schema: TopologySchemaBinding,
{
    ApplicationInvariantDefinition::new(
        PositivePlanarTurn::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(NonZeroU64::new(1024).expect("nonzero fixture budget")),
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
