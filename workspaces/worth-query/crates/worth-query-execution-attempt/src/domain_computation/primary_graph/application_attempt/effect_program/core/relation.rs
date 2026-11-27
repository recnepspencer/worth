//! Concrete link and unlink effects share original target and contract checks.
use super::*;
use worth_relational::facade::identity::RelationId;
impl EffectCore<'_> {
    pub fn link_at_target_index(
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
        self.refuse_self_grant(kind, from, relation)?;
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
    /// Every handler, named, initialization, body-creation and fragment link
    /// reaches this funnel, so no link verb can grant its own principal a
    /// grant kind declared `forbid_self_grant`. Only an existing entity can
    /// be the admitted principal; a created endpoint never is.
    fn refuse_self_grant(
        &self,
        relation_kind: KindId,
        from: Target<'_>,
        relation: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let EntityReference::Existing(entity) = from.reference else {
            return Ok(());
        };
        let links_own_principal =
            crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(*entity)
                == self.principal;
        if links_own_principal && self.layout.forbids_self_grant(relation_kind) {
            // The operation's current authority never covers granting its
            // own principal a self-grant-forbidden grant kind.
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                format!("self-grant:{relation}"),
            ));
        }
        Ok(())
    }
    pub fn unlink_at_observation(
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
    pub fn unlink_at_target(
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
