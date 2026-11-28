//! Early authoring guards over the exact completed facts and pending effects.
use super::*;
use worth_foundational::facade::AspectKey;

use model::WorthQueryApplicationRealizedEffect as Effect;

impl EffectCore<'_> {
    pub(super) fn require_exclusive_write(
        &self,
        entity: &str,
        id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let aspect = locator.aspect().aspect_key();
        let mut groups = self
            .layout
            .aspect_exclusion_groups(entity, aspect)
            .peekable();
        if groups.peek().is_none() {
            return Ok(());
        }
        self.require_parent_not_removed(entity, id, locator)?;
        self.require_field_observation(id, locator)?;
        // A write to the active case does not change group membership.
        if self.case_presence(entity, id, aspect) == Some(true) {
            return Ok(());
        }
        for (members, _) in groups {
            if members.iter().any(|sibling| {
                sibling != aspect && self.case_presence(entity, id, sibling) != Some(false)
            }) {
                return Err(exclusion_conflict(entity, id, aspect));
            }
        }
        Ok(())
    }

    pub(super) fn require_exclusive_removal(
        &self,
        entity: &str,
        id: EntityId,
        aspect: &AspectKey,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        for (members, required) in self.layout.aspect_exclusion_groups(entity, aspect) {
            if required
                && self.case_presence(entity, id, aspect) != Some(false)
                && !members.iter().any(|sibling| {
                    sibling != aspect && self.case_presence(entity, id, sibling) == Some(true)
                })
            {
                return Err(exclusion_conflict(entity, id, aspect));
            }
        }
        Ok(())
    }

    fn case_presence(&self, entity: &str, id: EntityId, aspect: &AspectKey) -> Option<bool> {
        if self
            .field_write_positions
            .position(entity, id)
            .is_some_and(|position| match &self.effects[position] {
                Effect::UpdateEntity { fields, .. } => fields
                    .keys()
                    .any(|field| field.aspect().aspect_key() == aspect),
                Effect::PatchOptionalEntityFields { fields, .. } => {
                    fields.iter().any(|(field, write)| {
                        field.aspect().aspect_key() == aspect && write.value.is_some()
                    })
                }
                _ => false,
            })
        {
            return Some(true);
        }
        if self.effects.iter().any(|effect| {
            matches!(effect, Effect::RemoveAspect { entity_id, contract, .. }
                if *entity_id == id && contract.key() == aspect)
        }) {
            return Some(false);
        }
        self.routes.aspect_presence(self.facts, id, aspect)
    }
}

fn exclusion_conflict(
    entity: &str,
    id: EntityId,
    aspect: &AspectKey,
) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        format!(
            "{entity} {id:?}.{} requires an atomic case transition",
            aspect.as_str()
        ),
    )
}
