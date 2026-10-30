//! Author a whole-parent removal from the complete same-program decision closure.
use super::*;
use worth_foundational::facade::{AbsenceLaw, AspectKey, AspectShape, PortableAspectContractBasis};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityMarkerIdentity,
    OperationRemovesAspects,
};
use worth_relational::facade::identity::EntityId;
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub fn remove_aspect<Entity, Aspect>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
        Aspect:
            ApplicationAspectMarkerIdentity<Schema, Entity> + OperationRemovesAspects<Operation>,
    {
        self.reserve_effects(1)?;
        let entity = Entity::IDENTIFIER;
        self.validate_target(target, entity)?;
        self.admit_program_target(&ApplicationOperationProgramTarget::RemoveAspect {
            entity: entity.into(),
            aspect: aspect.name().into(),
        })?;
        let EntityReference::Existing(id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ));
        };
        let key = AspectKey::new(aspect.name()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                aspect.name(),
            )
        })?;
        let contract = self.layout.aspect_contract(entity, &key).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                aspect.name(),
            )
        })?;
        if contract.identity() != aspect.identity()
            || contract.revision() != aspect.revision()
            || contract.absence() != AbsenceLaw::Optional
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                aspect.name(),
            ));
        }
        let AspectShape::Struct(shape) = contract.shape() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                aspect.name(),
            ));
        };
        self.require_aspect_unwritten(id, &key)?;
        let mut fields = Vec::new();
        fields.try_reserve(shape.fields().len()).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::AllocationDenied,
                aspect.name(),
            )
        })?;
        for field in shape.fields() {
            let locator = self.field_locator(entity, aspect.name(), field.key().as_str())?;
            // Exact completed-body routes preserve every original value/absence and full generation.
            self.require_field_observation(id, &locator)?;
            fields.push(locator);
        }
        if fields.is_empty() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                aspect.name(),
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
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
                key.as_str(),
            ))
        } else {
            Ok(())
        }
    }
    pub(super) fn require_parent_not_removed(
        &self,
        id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if self.effects.iter().any(|effect| matches!(effect, model::WorthQueryApplicationRealizedEffect::RemoveAspect { entity_id, contract, .. }
            if *entity_id == id && contract.key() == locator.aspect().aspect_key())) {
            Err(denial(WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep, locator.aspect().aspect_key().as_str()))
        } else { Ok(()) }
    }
}
