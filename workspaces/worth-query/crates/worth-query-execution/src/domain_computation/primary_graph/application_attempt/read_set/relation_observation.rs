use worth_relational::facade::identity::{EntityId, KindId};

use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
};

/// The observed links of one declared relation between two exact entities, as
/// recorded in a decision read set.
///
/// Returned by `observe_relation`; inspect it with `count` and `is_absent`, or
/// pass it to the effect program's `unlink` to remove exactly the observed
/// links.
pub struct WorthQueryObservedApplicationRelation<Schema, Relation, From, To> {
    pub(super) count: usize,
    pub(in crate::domain_computation::primary_graph::application_attempt) matching_relations:
        Vec<worth_relational::facade::identity::RelationId>,
    pub(super) _marker: std::marker::PhantomData<fn() -> (Schema, Relation, From, To)>,
}

impl<Schema, Relation, From, To> WorthQueryObservedApplicationRelation<Schema, Relation, From, To> {
    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn is_absent(&self) -> bool {
        self.count == 0
    }
}

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    /// Recovers only relation identities carried by this completed read set.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn relation_observation<
        Relation,
        From,
        To,
    >(
        &self,
        name: &str,
        kind: KindId,
        from: EntityId,
        to: EntityId,
    ) -> Result<
        WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    > {
        let matching_relations = self
            .facts
            .iter()
            .find_map(|fact| match fact {
                WorthQueryApplicationObservedFact::Relation {
                    relation_kind,
                    from: observed_from,
                    to: observed_to,
                    matching_relations,
                    ..
                } if *relation_kind == kind && *observed_from == from && *observed_to == to => {
                    Some(matching_relations.clone())
                }
                WorthQueryApplicationObservedFact::Adjacency {
                    relation_kind,
                    relations,
                    ..
                } if *relation_kind == kind => {
                    let matches = relations
                        .iter()
                        .filter(|observed| observed.from == from && observed.to == to)
                        .map(|observed| observed.relation_id)
                        .collect::<Vec<_>>();
                    (!matches.is_empty()).then_some(matches)
                }
                _ => None,
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    name,
                )
            })?;
        Ok(WorthQueryObservedApplicationRelation {
            count: matching_relations.len(),
            matching_relations,
            _marker: std::marker::PhantomData,
        })
    }
}
