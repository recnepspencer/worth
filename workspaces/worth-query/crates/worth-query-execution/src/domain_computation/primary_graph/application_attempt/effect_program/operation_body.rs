//! Installed body effects use native target, footprint and reservation checks.
use super::{
    denial, ApplicationOperationProgramTarget, EntityReference, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn reserve_body_signing_slot(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, ()>,
        spec: &worth_query_declaration::facade::application_schema::ApplicationOperationBodySigningSpec,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        use super::super::signed_creation::{DeferredSigningSlot, SignedCreationState};
        let field = spec.provenance();
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })?;
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let effect = target.created_effect.ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            )
        })?;
        let Some(super::WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. }) =
            self.effects.get(effect)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        if fields.contains_key(&locator) || self.read_set.signed_creation.is_some() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
                field.field(),
            ));
        }
        let representation_capacity = self
            .candidate_reservation
            .as_mut()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                    field.field(),
                )
            })?
            .reserve_deferred_initializer()?;
        self.read_set.signed_creation = Some(SignedCreationState {
            slot: DeferredSigningSlot {
                effect,
                field: locator,
                spec: spec.clone(),
                representation_capacity,
            },
            authentication: None,
            sealed: None,
        });
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn write_body_input_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.write_body_value(target, field, std::borrow::Cow::Owned(value))
    }
    pub(in crate::domain_computation::primary_graph) fn write_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.write_body_value(target, field, std::borrow::Cow::Borrowed(value))
    }

    fn write_body_value<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.field(),
            ));
        };
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        if self.layout.field_is_optional(field.entity(), &locator) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.field(),
            ));
        }
        self.write_required_value(field, entity_id, locator, value)
    }

    pub(in crate::domain_computation::primary_graph) fn write_body_optional_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Option<AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.field(),
            ));
        };
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        if !self.layout.field_is_optional(field.entity(), &locator) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.field(),
            ));
        }
        self.write_optional_value(field, entity_id, locator, value)
    }
}
