//! Whole-aspect removal shares field closure, conflict and allocation checks.
use super::super::aspect_removal::aspect_conflict;
use super::*;
use worth_foundational::facade::{AbsenceLaw, AspectKey, AspectShape, PortableAspectContractBasis};
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, FragmentCatalog,
};
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn remove_aspect(
        &mut self,
        target: Target<'_>,
        entity: &str,
        name: &str,
        installed: fn(&FragmentCatalog) -> Option<&AspectDefinition>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.reserve_effects(1)?;
        self.layout.aspect_selection(name, installed)?;
        self.validate_target(target, entity)?;
        self.admit_program_target(&ApplicationOperationProgramTarget::RemoveAspect {
            entity: entity.into(),
            aspect: name.into(),
        })?;
        let EntityReference::Existing(id) = *target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ));
        };
        let key = AspectKey::new(name).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                name,
            )
        })?;
        let contract = self.layout.aspect_contract(entity, &key).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                name,
            )
        })?;
        if contract.absence() != AbsenceLaw::Optional {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                name,
            ));
        }
        let AspectShape::Struct(shape) = contract.shape() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                name,
            ));
        };
        self.require_aspect_unwritten(entity, id, &key)?;
        let mut fields = Vec::new();
        fields.try_reserve(shape.fields().len()).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::AllocationDenied,
                name,
            )
        })?;
        for field in shape.fields() {
            let locator = self.field_locator(entity, name, field.key().as_str())?;
            // Exact completed-body routes preserve every original value/absence and full generation.
            self.require_field_observation(id, &locator)?;
            fields.push(locator);
        }
        if fields.is_empty() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                name,
            ));
        }
        let contract = PortableAspectContractBasis::from_contract(contract);
        self.effects
            .push(model::WorthQueryApplicationRealizedEffect::RemoveAspect {
                entity: entity.into(),
                entity_id: id,
                contract,
                fields,
            });
        Ok(())
    }
    fn require_aspect_unwritten(
        &self,
        entity: &str,
        id: EntityId,
        key: &AspectKey,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        use model::WorthQueryApplicationRealizedEffect as Effect;
        let conflict = self.effects.iter().any(|e| match e {
            Effect::UpdateEntity {
                entity_id, fields, ..
            } => *entity_id == id && fields.keys().any(|f| f.aspect().aspect_key() == key),
            Effect::PatchOptionalEntityFields {
                entity_id, fields, ..
            } => *entity_id == id && fields.keys().any(|f| f.aspect().aspect_key() == key),
            Effect::RemoveAspect {
                entity_id,
                contract,
                ..
            } => *entity_id == id && contract.key() == key,
            Effect::DeleteEntity { entity_id } => *entity_id == id,
            _ => false,
        });
        if conflict {
            Err(aspect_conflict(entity, id, key))
        } else {
            Ok(())
        }
    }
}
