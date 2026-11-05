//! A row preflights every scalar, then uses the ordinary field effect machinery.

use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationCreationPartition as Partition,
    WorthQueryApplicationEffectEntity as EffectEntity,
    WorthQueryApplicationEffectProgramBuilder as Builder,
    WorthQueryApplicationRealizedEffect as Effect,
};
use crate::domain_computation::primary_graph::fragment_view::{
    CandidateEntity, ErasedCandidate, FragmentDenial, WritePort,
};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, RelationDefinition,
};
use worth_query_installation::facade::ApplicationOperationProgramTarget as Target;
use worth_relational::facade::transactions::EntityReference;

impl<S, E> EffectEntity<S, E> {
    pub fn for_fragment(&self) -> CandidateEntity<E> {
        CandidateEntity {
            erased: erase(self),
            marker: PhantomData,
        }
    }
}

impl<S, O, I, Scope> Builder<S, O, I, Scope> {
    fn preflight_record(
        &mut self,
        target: &EffectEntity<S, ()>,
        definition: &AspectDefinition,
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
            self.admit_program_target(&Target::Write {
                entity: locus.entity().to_owned(),
                aspect: locus.aspect().to_owned(),
                field: locus.field().to_owned(),
            })?;
            let locator = self.field_locator(locus.entity(), locus.aspect(), locus.field())?;
            if let EntityReference::Existing(entity) = target.reference {
                self.require_parent_not_removed(locus.entity(), entity, &locator)?;
                self.require_field_observation(entity, &locator)?;
                if self.layout.field_is_optional(locus.entity(), &locator) {
                    return Err(Denial::new(Kind::UndeclaredEffect, definition.path()));
                }
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
}

impl<S, O, I, Scope> WritePort for Builder<S, O, I, Scope> {
    fn target(
        &self,
        target: &crate::domain_computation::primary_graph::WorthQueryInvariantMutationTarget<
            (),
            (),
        >,
    ) -> Result<ErasedCandidate, FragmentDenial> {
        let native = target.fragment_schema::<S>();
        Ok(erase(&self.projected_entity(&native)?))
    }

    fn create(&mut self, entity: &str, key: &str) -> Result<ErasedCandidate, FragmentDenial> {
        let native =
            self.create_entity_named_at::<()>(entity, key.to_owned(), Partition::Issued)?;
        Ok(erase(&native))
    }

    fn record(
        &mut self,
        target: &ErasedCandidate,
        definition: &AspectDefinition,
        mut fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        let native = restore::<S>(target);
        self.preflight_record(&native, definition)?;
        // Resolve the complete encoded row before retaining its first effect.
        let mut prepared = Vec::with_capacity(definition.fields().len());
        for field in definition.fields() {
            let path = format!("{}.{}", definition.path(), field.recipe().locus().field());
            let value = fields
                .remove(&path)
                .ok_or_else(|| Denial::new(Kind::InvalidEffectValue, &path))?;
            prepared.push((field.recipe().locus(), value));
        }
        for (locus, value) in prepared {
            if native.created_effect.is_some() {
                self.initialize_body_field(&native, locus, std::borrow::Cow::Owned(value))?;
            } else {
                self.write_body_input_field(&native, locus, value)?;
            }
        }
        Ok(())
    }

    fn link(
        &mut self,
        from: &ErasedCandidate,
        relation: &RelationDefinition,
        to: &ErasedCandidate,
    ) -> Result<(), FragmentDenial> {
        let from = restore::<S>(from);
        let to = restore::<S>(to);
        let key = format!("fragment-link-{}", self.effects.len());
        self.link_at_target(
            &Target::Link {
                relation: relation.name.clone(),
                from: relation.from.clone(),
                to: relation.to.clone(),
            },
            key,
            &from,
            &to,
        )?;
        Ok(())
    }
}

fn erase<S, E>(target: &EffectEntity<S, E>) -> ErasedCandidate {
    ErasedCandidate {
        reference: target.reference.clone(),
        entity: target.entity.clone(),
        created_effect: target.created_effect,
        program: Arc::clone(&target.program),
    }
}
fn restore<S>(target: &ErasedCandidate) -> EffectEntity<S, ()> {
    EffectEntity {
        reference: target.reference.clone(),
        entity: target.entity.clone(),
        created_effect: target.created_effect,
        program: Arc::clone(&target.program),
        _marker: PhantomData,
    }
}
