//! Owner-scoped persistent-name resolution on an operation's decision reader.
//!
//! The owner of a named record is whatever the kind's declared owner chain
//! (`WorthQueryNamedKind::OwnerScope`) reaches from it; nothing is inferred
//! from a name's text. Each hop is admitted against the operation's declared
//! relation read and read as a tracked adjacency fact, so re-owning a
//! candidate (linking or unlinking a hop) stales the decision exactly as a
//! split, merge or retirement of the candidate does.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::{
    NameOwnerHop, NameOwnerHopDirection, NameOwnerScope, NameSelection, WorthQueryName,
    WorthQueryNamedKind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, RelationDirection,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};
use worth_relational::facade::identity::EntityId;

use super::naming::{NameOwnerOf, WorthQueryDecisionNameDenial, WorthQueryDecisionNameResolution};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

type Erased = WorthQueryInvariantEntityIdentity<(), ()>;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Resolves a held entity name among the live records `owner` holds, as
    /// a tracked decision read.
    ///
    /// The selection applies only to `owner`'s candidates: one resolves
    /// `Exact`, several resolve `Plural` under `AllDescendants` and refuse
    /// `Ambiguous` otherwise. An owner holding no matching live record
    /// refuses `NoCandidateInOwner` with every candidate and the owners that
    /// do hold it; resolution never falls back to another owner. The
    /// operation must declare a read of each relation on the owner chain.
    pub fn resolve_in<Entity>(
        &mut self,
        owner: &WorthQueryInvariantEntityIdentity<Schema, NameOwnerOf<Entity>>,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
    ) -> Result<WorthQueryDecisionNameResolution<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>
            + OperationReads<Operation>
            + WorthQueryNamedKind,
    {
        let hops = <Entity::OwnerScope as NameOwnerScope<Entity>>::hops();
        let Some(owner_entity) = hops.last().map(NameOwnerHop::far) else {
            return Err(WorthQueryDecisionNameDenial::OwnerScopeUndeclared);
        };
        if !self.reader.identity_is_local(owner, owner_entity) {
            return Err(WorthQueryDecisionNameDenial::ForeignIdentity);
        }
        self.resolve_scoped(Some(owner.entity_id), held, selection)
    }

    /// Every owner the kind's declared owner chain reaches from `candidate`,
    /// in ascending record order; empty when the kind declares no owner
    /// scope or the chain reaches none.
    ///
    /// Complexity: O(hops * records reached), each adjacency read charged to
    /// the reader's work budget.
    pub(super) fn name_owners<Entity>(
        &mut self,
        candidate: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<BTreeMap<EntityId, Erased>, WorthQueryDecisionNameDenial>
    where
        Entity: WorthQueryNamedKind,
    {
        let hops = <Entity::OwnerScope as NameOwnerScope<Entity>>::hops();
        if hops.is_empty() {
            return Ok(BTreeMap::new());
        }
        let mut frontier =
            BTreeMap::from([(candidate.entity_id, candidate.fragment_schema::<()>())]);
        for hop in hops {
            let mut reached = BTreeMap::new();
            for near in frontier.values() {
                for far in self.owner_hop(&hop, near)? {
                    reached.insert(far.entity_id, far);
                }
            }
            frontier = reached;
        }
        Ok(frontier)
    }

    /// The records one declared hop reaches from `near`, read as a tracked
    /// adjacency fact of the hop's relation.
    fn owner_hop(
        &mut self,
        hop: &NameOwnerHop,
        near: &Erased,
    ) -> Result<Vec<Erased>, WorthQueryDecisionNameDenial> {
        let target = ApplicationOperationDecisionReadTarget::Relation {
            relation: hop.relation().to_owned(),
            from: hop.from().to_owned(),
            to: hop.to().to_owned(),
        };
        self.admit_decision_target(&target)
            .map_err(WorthQueryDecisionNameDenial::UndeclaredDecisionTarget)?;
        let (direction, fact) = match hop.direction() {
            NameOwnerHopDirection::Forward => (
                RelationDirection::Forward,
                WorthQueryApplicationAdjacencyDirection::Outgoing,
            ),
            NameOwnerHopDirection::Reverse => (
                RelationDirection::Reverse,
                WorthQueryApplicationAdjacencyDirection::Incoming,
            ),
        };
        self.recorded_adjacency(&target, near.entity_id, fact, |reader| {
            let mut far = Vec::new();
            reader.native_read().relations(
                hop.relation(),
                hop.from(),
                hop.to(),
                near.native_identity(),
                direction,
                &mut |_, from, to| {
                    far.push(match hop.direction() {
                        NameOwnerHopDirection::Forward => to,
                        NameOwnerHopDirection::Reverse => from,
                    });
                },
            )?;
            Ok(far)
        })
        .map_err(WorthQueryDecisionNameDenial::OwnerRead)
    }
}
