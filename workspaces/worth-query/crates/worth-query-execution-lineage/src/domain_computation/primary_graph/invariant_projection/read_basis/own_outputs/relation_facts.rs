//! Relation and adjacency facts through a successor's own-output view (#277):
//! the one owner every applied fact reader (the lease's completion, a carried
//! computation's re-observation) uses, so a handler, its recorded fact and a
//! re-read all see the same served list.
use super::OwnOutputOverlay;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::{identity::EntityId, runtime::RelationalAdjacencyDirection};

impl OwnOutputOverlay {
    /// `fact`, read at the current image, as the successor's view serves it:
    /// a relation or adjacency list the chain's links touch is current minus
    /// covered relations plus the covered relations of `head` (the same key
    /// read at the head pre-pin, read only when touched). Then every served
    /// relation the chain did not make must not reach a record the chain
    /// created (`coherent_endpoints`).
    pub fn serve_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fact: Fact,
        head: impl FnOnce() -> Result<Fact, Denial>,
    ) -> Result<Fact, Denial> {
        let served = match touched_list(&fact) {
            Some((anchor, kind, direction)) if self.touches_adjacency(anchor, kind, direction) => {
                self.merge_fact(fact, head()?)
            }
            _ => fact,
        };
        self.coherent_endpoints(execution, served)
    }

    /// Whether a recorded relation or adjacency `fact` still equals the list
    /// the view serves at `applied` (current minus covered, plus covered at
    /// the head pre-pin). `None` for any other fact, or one over a list the
    /// chain's links do not touch: the caller compares it as it would anyway.
    pub fn served_fact_equal(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        applied: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        fact: &Fact,
    ) -> Result<Option<bool>, Denial> {
        use crate::domain_computation::primary_graph::application_attempt::fact::image::FactReadImage;
        match touched_list(fact) {
            Some((anchor, kind, direction)) if self.touches_adjacency(anchor, kind, direction) => {}
            _ => return Ok(None),
        }
        let image = |applied| FactReadImage {
            runtime,
            before: snapshot,
            applied: Some(applied),
        };
        let (current, head) = (image(applied), image(self.head()));
        let relations = self.relations();
        Ok(Some(match fact {
            Fact::Relation {
                relation_kind,
                from,
                to,
                matching_relations,
            } => {
                let served = relations.merge(
                    current.relations(execution, *relation_kind, *from, *to)?,
                    head.relations(execution, *relation_kind, *from, *to)?,
                    |id| (*id, *from, *to),
                );
                &served == matching_relations
            }
            Fact::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations: recorded,
            } => {
                let read = |image: &FactReadImage<'_>| {
                    image.adjacency(
                        execution,
                        *relation_kind,
                        *anchor,
                        *direction,
                        *maximum_work_units,
                    )
                };
                match (read(&current)?, read(&head)?) {
                    (Some(current), Some(head)) => {
                        &relations.merge(current, head, |relation| {
                            (relation.relation_id, relation.from, relation.to)
                        }) == recorded
                    }
                    // A bounded read past its limit is no served list.
                    _ => false,
                }
            }
            _ => return Ok(None),
        }))
    }

    /// Whether the part of `fact`'s list that Native's record of the chain
    /// does not speak for reads the same at the head pre-pin as at `applied`
    /// (#277). A chain delete changes its record and that record's relations.
    /// The relations it retires are covered and served from the head; what is
    /// left is the residue: the raw list at each pin, minus covered
    /// relations, kept where the anchor or an endpoint is a record the chain
    /// wrote. Two raw reads at two pins, never the served merge, so a
    /// difference is a change at a chain record that nothing certifies. A
    /// bounded read past its limit on either side proves nothing.
    pub fn uncovered_residue_equal(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        applied: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        fact: &Fact,
    ) -> Result<bool, Denial> {
        use crate::domain_computation::primary_graph::application_attempt::fact::image::FactReadImage;
        let image = |applied| FactReadImage {
            runtime,
            before: snapshot,
            applied: Some(applied),
        };
        let (current, head) = (image(applied), image(self.head()));
        let relations = self.relations();
        Ok(match fact {
            Fact::Relation {
                relation_kind,
                from,
                to,
                ..
            } if self.covers(*from) || self.covers(*to) => {
                let residue = |image: &FactReadImage<'_>| -> Result<Vec<_>, Denial> {
                    let mut list = image.relations(execution, *relation_kind, *from, *to)?;
                    list.retain(|id| !relations.covers(*id));
                    Ok(list)
                };
                residue(&current)? == residue(&head)?
            }
            Fact::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                ..
            } => {
                let anchored = self.covers(*anchor);
                let residue = |image: &FactReadImage<'_>| -> Result<Option<Vec<_>>, Denial> {
                    let list = image.adjacency(
                        execution,
                        *relation_kind,
                        *anchor,
                        *direction,
                        *maximum_work_units,
                    )?;
                    Ok(list.map(|mut list| {
                        list.retain(|relation| {
                            !relations.covers(relation.relation_id)
                                && (anchored
                                    || self.covers(relation.from)
                                    || self.covers(relation.to))
                        });
                        list
                    }))
                };
                match (residue(&current)?, residue(&head)?) {
                    (Some(current), Some(head)) => current == head,
                    _ => false,
                }
            }
            // No chain record on the list: nothing a chain delete could reach.
            _ => true,
        })
    }

    fn merge_fact(&self, current: Fact, head: Fact) -> Fact {
        let relations = self.relations();
        match (current, head) {
            (
                Fact::Relation {
                    relation_kind,
                    from,
                    to,
                    matching_relations,
                },
                Fact::Relation {
                    matching_relations: head_matching,
                    ..
                },
            ) => Fact::Relation {
                relation_kind,
                from,
                to,
                matching_relations: relations
                    .merge(matching_relations, head_matching, |id| (*id, from, to)),
            },
            (
                Fact::Adjacency {
                    relation_kind,
                    anchor,
                    direction,
                    maximum_work_units,
                    relations: current,
                },
                Fact::Adjacency {
                    relations: head_relations,
                    ..
                },
            ) => Fact::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations: relations.merge(current, head_relations, |relation| {
                    (relation.relation_id, relation.from, relation.to)
                }),
            },
            (current, _) => current,
        }
    }

    /// A served relation the chain did not make whose endpoint is a record
    /// the chain created (covered, absent at the head pre-pin) is another
    /// member's effect on the chain's records: mark the view incoherent and
    /// fail, so the round stops typed. Covered relations already come from
    /// the head image. One charged point read per covered endpoint.
    fn coherent_endpoints(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fact: Fact,
    ) -> Result<Fact, Denial> {
        let relations = self.relations();
        let endpoints: Vec<EntityId> = match &fact {
            Fact::Relation {
                from,
                to,
                matching_relations,
                ..
            } if matching_relations.iter().any(|id| !relations.covers(*id)) => vec![*from, *to],
            Fact::Adjacency {
                relations: served, ..
            } => served
                .iter()
                .filter(|relation| !relations.covers(relation.relation_id))
                .flat_map(|relation| [relation.from, relation.to])
                .collect(),
            _ => Vec::new(),
        };
        for endpoint in endpoints {
            match self.created_by_chain(execution, endpoint) {
                Ok(false) => {}
                Ok(true) => {
                    self.mark_incoherent(endpoint);
                    return Err(Denial::new(
                        Kind::MissingAuthoritativeFact,
                        "own-output relation endpoint",
                    ));
                }
                Err(cause) => {
                    return Err(Denial::relational_read_stopped(
                        cause,
                        "own-output relation endpoint",
                    ));
                }
            }
        }
        Ok(fact)
    }
}

/// The adjacency list a relation or adjacency fact reads.
fn touched_list(
    fact: &Fact,
) -> Option<(
    EntityId,
    worth_relational::facade::identity::KindId,
    RelationalAdjacencyDirection,
)> {
    match fact {
        Fact::Relation {
            relation_kind,
            from,
            ..
        } => Some((
            *from,
            *relation_kind,
            RelationalAdjacencyDirection::Outgoing,
        )),
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            ..
        } => Some((
            *anchor,
            *relation_kind,
            match direction {
                WorthQueryApplicationAdjacencyDirection::Outgoing => {
                    RelationalAdjacencyDirection::Outgoing
                }
                WorthQueryApplicationAdjacencyDirection::Incoming => {
                    RelationalAdjacencyDirection::Incoming
                }
            },
        )),
        _ => None,
    }
}
