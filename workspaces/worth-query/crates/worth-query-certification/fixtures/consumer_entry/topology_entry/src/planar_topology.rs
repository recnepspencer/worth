use worth_query_decl::facade::application_schema::{
    fragment, ApplicationRelationMinimumReachability,
};
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, ApplicationRelationCardinality,
    ApplicationRelationDeletionPolicy, ApplicationRelationRef, RelationalInvariantWorkBudget,
};

use super::{Body, TopologyModelBinding, TopologyRecords};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarSuccessor;

impl fragment::Relation for PlanarSuccessor {
    const NAME: &'static str = "PlanarSuccessor";
    type To = Body;
    const SELF_EDGES: bool = false;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
    const ORBIT: bool = true;
    const REACHES_AT_LEAST: Option<ApplicationRelationMinimumReachability> =
        Some(ApplicationRelationMinimumReachability {
            minimum_reachable_targets: 2,
            target_kinds: &[<Body as fragment::Entity>::NAME],
        });
}
impl PlanarSuccessor {
    pub fn reference<S: TopologyModelBinding>() -> ApplicationRelationRef<S, Self, Body, Body> {
        fragment::relation(TopologyRecords, Body, Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UndeclaredPlanarRelation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarDiscoverySource;

impl fragment::Relation for PlanarDiscoverySource {
    const NAME: &'static str = "PlanarDiscoverySource";
    type To = Body;
    const SELF_EDGES: bool = false;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, None, None, None);
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations;
}
impl PlanarDiscoverySource {
    pub fn reference<S: TopologyModelBinding>() -> ApplicationRelationRef<S, Self, Body, Body> {
        fragment::relation(TopologyRecords, Body, Self)
    }
}

impl fragment::Relation for UndeclaredPlanarRelation {
    const NAME: &'static str = "UndeclaredPlanarRelation";
    type To = Body;
    const SELF_EDGES: bool = false;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, None, None, None);
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl UndeclaredPlanarRelation {
    pub fn reference<S: TopologyModelBinding>() -> ApplicationRelationRef<S, Self, Body, Body> {
        fragment::relation(TopologyRecords, Body, Self)
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

// Conservative work policy for eight triangle rings: complete component reads,
// successor lookup and six coordinate reads per body, with 4x headroom.
// This allowance refuses exhausted work; it never truncates traversal depth.
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
                ApplicationInvariantScopeTarget::Relation("PlanarCoupling".to_owned()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Body".to_owned()),
                ApplicationInvariantScopeTarget::Relation("PlanarSuccessor".to_owned()),
                ApplicationInvariantScopeTarget::Relation("PlanarCoupling".to_owned()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
