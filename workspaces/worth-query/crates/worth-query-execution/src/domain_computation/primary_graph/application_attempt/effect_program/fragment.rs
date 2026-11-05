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

impl<S: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
{
    pub fn fragment_candidate<
        'a,
        F: worth_query_declaration::facade::application_schema::fragment::Fragment,
        O,
        I,
        Scope,
    >(
        &'a self,
        candidate: &'a mut Builder<S, O, I, Scope>,
        marker: F,
    ) -> Result<
        crate::domain_computation::primary_graph::fragment_view::View<
            'a,
            F,
            crate::domain_computation::primary_graph::fragment_view::Writing<'a>,
        >,
        FragmentDenial,
    > {
        if !candidate.read_set.admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(Denial::new(
                Kind::ForeignApplication,
                candidate.read_set.admission.operation(),
            )
            .into());
        }
        let definition = self
            .installed_schema
            .value_bindings()
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        Ok(
            crate::domain_computation::primary_graph::fragment_view::View::writing(
                definition, candidate,
            ),
        )
    }
}

impl<S, E> EffectEntity<S, E> {
    pub fn for_fragment(&self) -> CandidateEntity<E> {
        CandidateEntity {
            erased: erase(self),
            marker: PhantomData,
        }
    }
}

impl<S, O, I, Scope> Builder<S, O, I, Scope> {
    pub(super) fn preflight_record(
        &mut self,
        target: &EffectEntity<S, ()>,
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
            self.admit_program_target(&Target::Write {
                entity: locus.entity().to_owned(),
                aspect: locus.aspect().to_owned(),
                field: locus.field().to_owned(),
            })?;
            let locator = self.field_locator(locus.entity(), locus.aspect(), locus.field())?;
            if let EntityReference::Existing(entity) = target.reference {
                if self.effects.iter().enumerate().any(|(index, effect)| {
                    matches!(effect, Effect::RemoveAspect { entity_id, contract, .. }
                        if *entity_id == entity && contract.key() == locator.aspect().aspect_key())
                        && !superseded_removals.contains(&index)
                }) {
                    self.require_parent_not_removed(locus.entity(), entity, &locator)?;
                }
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
    fn case_record(
        &mut self,
        target: &ErasedCandidate,
        aspect: &AspectDefinition,
        group: &worth_query_declaration::facade::application_schema::fragment::GroupDefinition,
        cases: &[&AspectDefinition],
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        self.write_case_record(target, aspect, group, cases, fields)
    }
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
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        let native = restore::<S>(target);
        self.preflight_record(&native, definition, &[])?;
        // Resolve the complete encoded row before retaining its first effect.
        let prepared = prepare_fields(definition, fields)?;
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
pub(super) fn restore<S>(target: &ErasedCandidate) -> EffectEntity<S, ()> {
    EffectEntity {
        reference: target.reference.clone(),
        entity: target.entity.clone(),
        created_effect: target.created_effect,
        program: Arc::clone(&target.program),
        _marker: PhantomData,
    }
}

pub(super) fn prepare_fields(
    definition: &AspectDefinition,
    mut fields: BTreeMap<String, AspectValue>,
) -> Result<
    Vec<(
        &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        AspectValue,
    )>,
    Denial,
> {
    let mut prepared = Vec::new();
    prepared
        .try_reserve(definition.fields().len())
        .map_err(|_| Denial::new(Kind::AllocationDenied, definition.path()))?;
    for field in definition.fields() {
        let locus = field.recipe().locus();
        let path = format!("{}.{}", definition.path(), locus.field());
        let value = fields
            .remove(&path)
            .ok_or_else(|| Denial::new(Kind::InvalidEffectValue, &path))?;
        prepared.push((locus, value));
    }
    if let Some((path, _)) = fields.first_key_value() {
        return Err(Denial::new(Kind::InvalidEffectValue, path));
    }
    Ok(prepared)
}
