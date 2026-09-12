use worth_relational::facade::identity::{EntityId, KindId};

use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryObservedApplicationRelation,
};

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
