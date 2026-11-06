//! Concrete whole-row preflight and retention through the scalar effect core.
use super::super::fragment::prepare_fields;
use super::*;
use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationRealizedEffect as Effect,
};
use crate::domain_computation::primary_graph::fragment_view::{ErasedCandidate, FragmentDenial};
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, RecordFieldDefinition, RelationDefinition,
};
use worth_query_installation::facade::ApplicationOperationProgramTarget as TargetKind;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn preflight_record(
        &mut self,
        target: Target<'_>,
        definition: &AspectDefinition,
        superseded_removals: &[usize],
    ) -> Result<(), Denial> {
        self.validate_target(target, definition.entity())?;
        let created = target.created_effect;
        if created.is_some_and(|index| {
            !matches!(self.effects.get(index), Some(Effect::CreateEntity { .. }))
        }) {
            return Err(Denial::new(Kind::ForeignEffectTarget, definition.path()));
        }
        for field in definition.fields() {
            let locus = field.recipe().locus();
            if created.is_none() && !field.writable() {
                return Err(Denial::new(Kind::UndeclaredEffect, field.path()));
            }
            self.admit_program_target(&TargetKind::Write {
                entity: locus.entity().to_owned(),
                aspect: locus.aspect().to_owned(),
                field: locus.field().to_owned(),
            })?;
            let locator = self.field_locator(locus.entity(), locus.aspect(), locus.field())?;
            if let EntityReference::Existing(entity) = target.reference {
                if self.effects.iter().enumerate().any(|(index, effect)| {
                    matches!(effect, Effect::RemoveAspect { entity_id, contract, .. }
                        if *entity_id == *entity && contract.key() == locator.aspect().aspect_key())
                        && !superseded_removals.contains(&index)
                }) {
                    self.require_parent_not_removed(locus.entity(), *entity, &locator)?;
                }
                self.require_field_observation(*entity, &locator)?;
                if self
                    .layout
                    .aspect_contract(locus.entity(), locator.aspect().aspect_key())
                    .is_none()
                {
                    return Err(Denial::new(Kind::UndeclaredEffect, definition.path()));
                }
            } else if created.is_none() {
                return Err(Denial::new(Kind::ForeignEffectTarget, definition.path()));
            }
        }
        self.reserve_effects(usize::from(created.is_none()))
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_prepared_record_field(
        &mut self,
        target: Target<'_>,
        field: &RecordFieldDefinition,
        value: Option<AspectValue>,
    ) -> Result<(), Denial> {
        let locus = field.recipe().locus();
        if target.created_effect.is_some() {
            if let Some(value) = value {
                self.initialize_encoded_field(target, locus, std::borrow::Cow::Owned(value))?;
            }
            return Ok(());
        }
        if field.presence() == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional {
            self.write_body_optional_field(target, locus, value)
        } else {
            self.write_body_value(target, locus, std::borrow::Cow::Owned(value.ok_or_else(|| Denial::new(Kind::InvalidEffectValue, locus.field()))?))
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn record(
        &mut self,
        target: &ErasedCandidate,
        definition: &AspectDefinition,
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        let native = target.borrowed();
        self.preflight_record(native, definition, &[])?;
        // Resolve the complete encoded row before retaining its first effect.
        let prepared = prepare_fields(definition, fields)?;
        if let Some(index) = native.created_effect {
            let Some(Effect::CreateEntity { fields, .. }) = self.effects.get_mut(index) else {
                return Err(Denial::new(Kind::ForeignEffectTarget, definition.path()).into());
            };
            fields.retain(|field, _| field.aspect().aspect_key().as_str() != definition.name());
        }
        for (field, value) in prepared {
            self.write_prepared_record_field(native, field, value)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn link_fragment(
        &mut self,
        from: &ErasedCandidate,
        relation: &RelationDefinition,
        to: &ErasedCandidate,
    ) -> Result<(), FragmentDenial> {
        let from = from.borrowed();
        let to = to.borrowed();
        let mut key = Some(format!("fragment-link-{}", self.effects.len()));
        self.link_at_target_index(
            &TargetKind::Link {
                relation: relation.name.clone(),
                from: relation.from.clone(),
                to: relation.to.clone(),
            },
            &mut || key.take().expect("one admitted fragment key conversion"),
            from,
            to,
        )?;
        Ok(())
    }
}
