use worth_query_installation::facade::{
    ApplicationOperationProgramTarget, ApplicationRelationRef, OperationLinks, OperationReads,
    OperationUnlinks,
};

use super::{
    canonical_key, denial, CandidateItemKind, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::WorthQueryObservedApplicationRelation;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn unlink_observed<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Operation> + OperationUnlinks<Operation>,
    {
        use worth_relational::facade::transactions::EntityReference;
        self.validate_target(from, relation.from())?;
        self.validate_target(to, relation.to())?;
        let (EntityReference::Existing(from_id), EntityReference::Existing(to_id)) =
            (&from.reference, &to.reference)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                relation.name(),
            ));
        };
        let kind = self
            .layout
            .relation(relation.name())
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    relation.name(),
                )
            })?
            .kind;
        let observed =
            self.read_set
                .relation_observation(relation.name(), kind, *from_id, *to_id)?;
        self.unlink(relation, observed)
    }

    pub fn link<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Operation>,
    {
        self.validate_target(from, relation.from())?;
        self.validate_target(to, relation.to())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Link {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        })?;
        let kind = self
            .layout
            .relation(relation.name())
            .map(|layout| layout.kind)
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    relation.name(),
                )
            })?;
        let key = canonical_key(key.into(), relation.name())?;
        if self.keys.contains(&(kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                relation.name(),
            ));
        }
        let retained_representation_bytes =
            super::retained_representation::relation(&key, &from.reference, &to.reference)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        relation.name(),
                    )
                })?;
        self.charge_candidate_representation(
            CandidateItemKind::Link,
            retained_representation_bytes,
            0,
        )?;
        self.keys.insert((kind, key.clone()));
        self.effects
            .push(WorthQueryApplicationRealizedEffect::CreateRelation {
                kind,
                key,
                from: from.reference.clone(),
                to: to.reference.clone(),
            });
        Ok(())
    }

    pub fn unlink<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        observed: WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationUnlinks<Operation>,
    {
        self.admit_program_target(&ApplicationOperationProgramTarget::Unlink {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        })?;
        self.charge_candidate_items(CandidateItemKind::Unlink, observed.matching_relations.len())?;
        self.effects
            .extend(observed.matching_relations.into_iter().map(|relation_id| {
                WorthQueryApplicationRealizedEffect::DeleteRelation { relation_id }
            }));
        Ok(())
    }
}
