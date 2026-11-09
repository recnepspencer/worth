//! Concrete link and unlink effects share original target and contract checks.
use super::*;
use worth_relational::facade::identity::RelationId;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn link_at_target_index(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        key: &mut dyn FnMut() -> String,
        from: Target<'_>,
        to: Target<'_>,
    ) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        let ApplicationOperationProgramTarget::Link {
            relation,
            from: from_name,
            to: to_name,
        } = target
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                "relation link",
            ));
        };
        self.validate_target(from, from_name)?;
        self.validate_target(to, to_name)?;
        self.admit_program_target(target)?;
        let kind = self
            .layout
            .relation(relation)
            .map(|layout| layout.kind)
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    relation,
                )
            })?;
        let key = canonical_key(key(), relation)?;
        if self.keys.contains(&(kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                relation,
            ));
        }
        self.reserve_effects(1)?;
        self.keys.insert((kind, key.clone()));
        let position = self.effects.len();
        self.effects
            .push(WorthQueryApplicationRealizedEffect::CreateRelation {
                fields: Default::default(),
                kind,
                key,
                from: (*from.reference).clone(),
                to: (*to.reference).clone(),
            });
        Ok(position)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn unlink_at_observation(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        matching_relations: Vec<RelationId>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.admit_program_target(target)?;
        self.reserve_effects(matching_relations.len())?;
        self.effects
            .extend(matching_relations.into_iter().map(|relation_id| {
                WorthQueryApplicationRealizedEffect::DeleteRelation { relation_id }
            }));
        Ok(())
    }
}

impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn unlink_at_target(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        from: Target<'_>,
        to: Target<'_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let ApplicationOperationProgramTarget::Unlink {
            relation,
            from: source,
            to: destination,
        } = target
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                "relation unlink",
            ));
        };
        use worth_relational::facade::transactions::EntityReference;
        self.validate_target(from, source)?;
        self.validate_target(to, destination)?;
        let (EntityReference::Existing(from_id), EntityReference::Existing(to_id)) =
            (from.reference, to.reference)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                relation,
            ));
        };
        let kind = self
            .layout
            .relation(relation)
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    relation,
                )
            })?
            .kind;
        let matching = super::super::super::read_set::matching_relations(
            self.facts, relation, kind, *from_id, *to_id,
        )?;
        self.unlink_at_observation(target, matching)
    }
}
