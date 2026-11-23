//! A peer scope is admitted only through an owner it shares with the
//! operation's own scope under the declared governing relation.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget;
use worth_relational::facade::identity::{EntityId, KindId};

/// The declared governing relation of one peer-scope Parent read.
pub struct SharedOwnerRelation<'a> {
    pub relation: &'a str,
    pub owner: &'a str,
    pub member: &'a str,
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Whether `peer` shares an owner with the operation's own admitted scope
    /// through `relation`. Both
    /// owner sets are recorded Incoming adjacency decision facts, so an edit
    /// to either scope's ownership invalidates the decision that relied on it.
    /// Cost: O(in-degree(own) + in-degree(peer)) under the projection budget.
    pub fn shares_relation_owner(
        &mut self,
        relation: &SharedOwnerRelation<'_>,
        peer: EntityId,
    ) -> Result<bool, WorthQueryInvariantProjectionTraversalDenial> {
        let own = self
            .admitted_root
            .as_ref()
            .map(|root| root.entity)
            .ok_or_else(|| super::super::traversal::endpoint_unavailable(relation.member))?;
        let target = ApplicationOperationDecisionReadTarget::Relation {
            relation: relation.relation.to_owned(),
            from: relation.owner.to_owned(),
            to: relation.member.to_owned(),
        };
        self.admit_decision_target(&target).map_err(|denial| {
            WorthQueryInvariantProjectionTraversalDenial::new(
                super::super::WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget,
                denial.subject(),
            )
        })?;
        let member = self
            .reader
            .layout
            .relation(relation.relation)
            .map(|layout| layout.to)
            .ok_or_else(|| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    super::super::WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled,
                    relation.relation,
                )
            })?;
        let own_owners = self.owners_of(&target, relation, own, member)?;
        let peer_owners = self.owners_of(&target, relation, peer, member)?;
        Ok(own_owners.iter().any(|owner| peer_owners.contains(owner)))
    }

    fn owners_of(
        &mut self,
        target: &ApplicationOperationDecisionReadTarget,
        relation: &SharedOwnerRelation<'_>,
        anchor: EntityId,
        member: KindId,
    ) -> Result<Vec<EntityId>, WorthQueryInvariantProjectionTraversalDenial> {
        let execution = self.reader.execution;
        self.recorded_adjacency(
            target,
            anchor,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            |reader| {
                let identity = reader.endpoint::<()>(execution, anchor, member, relation.member)?;
                let mut owners = Vec::new();
                reader.native_read().relations(
                    relation.relation,
                    relation.owner,
                    relation.member,
                    identity.native_identity(),
                    RelationDirection::Reverse,
                    &mut |_, owner, _| owners.push(owner.entity_id),
                )?;
                Ok(owners)
            },
        )
    }
}
