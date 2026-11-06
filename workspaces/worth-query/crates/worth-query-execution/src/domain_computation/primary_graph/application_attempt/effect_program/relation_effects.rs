use worth_query_installation::facade::ApplicationOperationProgramTarget;
use worth_query_installation::facade::ApplicationRelationRef;
use worth_query_installation::facade::OperationLinks;
use worth_query_installation::facade::OperationReads;
use worth_query_installation::facade::OperationUnlinks;

use super::WorthQueryApplicationAttemptDenial;
use super::WorthQueryApplicationEffectEntity;
use super::WorthQueryApplicationEffectProgramBuilder;
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
        self.unlink_at_target(
            &ApplicationOperationProgramTarget::Unlink {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
            },
            from,
            to,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn unlink_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.native_effects()
            .unlink_at_target(target, from.borrowed(), to.borrowed())
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
        self.link_at_target(
            &ApplicationOperationProgramTarget::Link {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
            key,
            from,
            to,
        )
    }

    /// Typed writers and installed bodies share the same relation effect admission.
    pub(in crate::domain_computation::primary_graph) fn link_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.link_at_target_index(target, key, from, to).map(|_| ())
    }

    pub(super) fn link_at_target_index<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        let mut key = Some(key);
        self.native_effects().link_at_target_index(
            target,
            &mut || key.take().expect("one admitted key conversion").into(),
            from.borrowed(),
            to.borrowed(),
        )
    }

    pub fn unlink<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        observed: WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationUnlinks<Operation>,
    {
        self.unlink_at_observation(
            &ApplicationOperationProgramTarget::Unlink {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
            observed,
        )
    }
    fn unlink_at_observation<Relation, From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        observed: WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.native_effects()
            .unlink_at_observation(target, observed.matching_relations)
    }
}
