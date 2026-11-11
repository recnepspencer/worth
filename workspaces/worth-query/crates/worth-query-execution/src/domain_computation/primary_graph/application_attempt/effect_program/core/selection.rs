//! Sealed read facts and their affinity select exact effect targets once.
use super::*;
use crate::domain_computation::{
    authorization::WorthQueryOperationAdmissionIdentity,
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct EffectSelection<
    'a,
> {
    layout:
        &'a crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    program: &'a Arc<()>,
    facts: &'a crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    routes: &'a super::super::super::CompletedFactRoutes,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: &'a ApplicationSchemaBindingIdentity,
    admission: WorthQueryOperationAdmissionIdentity,
    phase: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
}
impl<S, O, I, Scope> WorthQueryApplicationEffectProgramBuilder<S, O, I, Scope> {
    #[inline(always)]
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn native_selection(
        &self,
    ) -> EffectSelection<'_> {
        EffectSelection {
            layout: &self.layout,
            program: &self.program,
            facts: &self.read_set.facts,
            routes: &self.read_set.effect_routes,
            runtime: self.read_set.admission.runtime_authority(),
            binding: self.read_set.admission.binding_identity(),
            admission: self.read_set.admission.admission_identity(),
            phase: self.read_set.lease.applied_qualification(),
        }
    }
}
impl EffectSelection<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn resolve_observed_entity(
        &self,
        field: ColumnIdentity,
        encoder: &dyn FieldEncoder,
    ) -> Result<WorthQueryApplicationEffectEntity<(), ()>, WorthQueryApplicationAttemptDenial> {
        let locator =
            super::field_locator(self.layout, field.entity(), field.aspect(), field.field())?;
        let encoded = encoder.encode_value().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                field.field(),
            )
        })?;
        let expected_kind = self.layout.entity_kind(field.entity()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.entity(),
            )
        })?;
        let matches = self.facts.iter().filter_map(|fact| match fact {
            super::super::super::WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator: observed_locator,
                value: observed_value,
            } if *kind == expected_kind
                && *observed_locator == locator
                && *observed_value == encoded =>
            {
                Some(*entity_id)
            }
            _ => None,
        });
        // A source proof and the decision read may observe the same entity;
        // only distinct matching entities make this effect target ambiguous.
        let mut entities = matches.collect::<std::collections::BTreeSet<_>>();
        let Some(entity_id) = entities.pop_first() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        if !entities.is_empty() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        }
        Ok(WorthQueryApplicationEffectEntity {
            reference: EntityReference::Existing(entity_id),
            entity: field.entity().to_owned(),
            created_effect: None,
            program: Arc::clone(self.program),
            _marker: std::marker::PhantomData,
        })
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn select(
        &self,
        entity_id: EntityId,
        entity: &str,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        admission: Option<(
            WorthQueryOperationAdmissionIdentity,
            Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
        )>,
    ) -> Result<WorthQueryApplicationEffectEntity<(), ()>, WorthQueryApplicationAttemptDenial> {
        let observed = self.routes.walk_touches_entity(self.facts, entity_id)
            || self
                .facts
                .iter()
                .filter(|fact| {
                    !matches!(fact,
            super::super::super::WorthQueryApplicationObservedFact::StructuralWalk {..}
            | super::super::super::WorthQueryApplicationObservedFact::ComponentSelection {..}
            | super::super::super::WorthQueryApplicationObservedFact::Aggregate {..}
        )
                })
                .any(|fact| fact.touches_entity(entity_id));
        if runtime != self.runtime
            || binding != self.binding
            || admission.is_some_and(|(admission, phase)| {
                admission != self.admission || phase != self.phase
            })
            || !observed
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ));
        }
        Ok(WorthQueryApplicationEffectEntity {
            reference: EntityReference::Existing(entity_id),
            entity: entity.to_owned(),
            created_effect: None,
            program: Arc::clone(self.program),
            _marker: std::marker::PhantomData,
        })
    }
}
