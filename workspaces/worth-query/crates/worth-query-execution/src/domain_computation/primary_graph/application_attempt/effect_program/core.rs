//! Borrowed effect authoring state; runtime checks and mutation have one concrete owner.
use super::*;
use crate::domain_computation::primary_graph::value_decode::FieldEncoder;
use std::collections::BTreeSet;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::ColumnIdentity;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::transactions::EntityReference;
mod aspect_removal;
mod body_field;
mod case_transition;
mod entity;
mod field_patch;
mod field_write;
mod fragment;
mod initialization;
mod relation;
mod relation_initialization;
mod retirement;
mod selection;
pub(super) use entity::context_partition;
pub(super) use relation_initialization::CreatedRelation;

#[derive(Clone, Copy)]
pub(super) struct Target<'a> {
    pub reference: &'a EntityReference,
    pub entity: &'a str,
    pub created_effect: Option<usize>,
    pub program: &'a Arc<()>,
}
impl<S, E> WorthQueryApplicationEffectEntity<S, E> {
    #[inline(always)]
    pub(super) fn borrowed(&self) -> Target<'_> {
        Target {
            reference: &self.reference,
            entity: &self.entity,
            created_effect: self.created_effect,
            program: &self.program,
        }
    }
}
impl WorthQueryApplicationEffectEntity<(), ()> {
    #[inline(always)]
    pub(super) fn into_typed<S, E>(self) -> WorthQueryApplicationEffectEntity<S, E> {
        WorthQueryApplicationEffectEntity {
            reference: self.reference,
            entity: self.entity,
            created_effect: self.created_effect,
            program: self.program,
            _marker: std::marker::PhantomData,
        }
    }
}
pub(super) struct EffectCore<'a> {
    layout: &'a super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    program: &'a Arc<()>,
    request: &'a worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    contracts: &'a WorthQueryCompiledApplicationOperationContracts,
    operation: &'a str,
    facts: &'a crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    routes: &'a super::super::CompletedFactRoutes,
    effects: &'a mut Vec<WorthQueryApplicationRealizedEffect>,
    field_write_positions: &'a mut FieldWriteIndex,
    keys: &'a mut BTreeSet<(KindId, String)>,
    case_transitions: &'a mut super::case_transition::CaseTransitions,
    creation_partition: &'a mut Option<WorthQueryApplicationCreationPartition>,
}
impl<S, O, I, Scope> WorthQueryApplicationEffectProgramBuilder<S, O, I, Scope> {
    #[inline(always)]
    pub(super) fn native_effects(&mut self) -> EffectCore<'_> {
        EffectCore {
            layout: &self.layout,
            program: &self.program,
            request: self.read_set.admission.publication_request(),
            contracts: self.read_set.admission.allowed_graph_contract(),
            operation: self.read_set.admission.operation(),
            case_transitions: &mut self.case_transitions,
            facts: &self.read_set.facts,
            routes: &self.read_set.effect_routes,
            effects: &mut self.effects,
            field_write_positions: &mut self.field_write_positions,
            keys: &mut self.keys,
            creation_partition: &mut self.creation_partition,
        }
    }
}
impl EffectCore<'_> {
    fn validate_target(
        &self,
        target: Target<'_>,
        entity: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        validate_target(self.program, target, entity)
    }
    fn admit_program_target(
        &self,
        target: &ApplicationOperationProgramTarget,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        admit_target(self.contracts, self.operation, target)
    }
    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
        field_locator(self.layout, entity, aspect, field)
    }
    fn require_field_observation(
        &self,
        id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        require_field_observation(self.routes, self.facts, id, locator)
    }
    fn require_parent_not_removed(
        &self,
        entity: &str,
        id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        super::aspect_removal::require_parent_not_removed(self.effects, entity, id, locator)
    }
    pub(super) fn reserve_effects(
        &mut self,
        additional: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(stop) = self.request.interruption() {
            use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
            return Err(denial(
                match stop {
                    Stop::Cancelled => WorthQueryApplicationAttemptDenialKind::Cancelled,
                    Stop::DeadlineExceeded => {
                        WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
                    }
                },
                "effect program request stopped",
            ));
        }
        self.effects.try_reserve(additional).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::AllocationDenied,
                "effect storage allocation refused",
            )
        })
    }
}
pub(super) fn validate_target(
    program: &Arc<()>,
    target: Target<'_>,
    entity: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if Arc::ptr_eq(target.program, program) && target.entity == entity {
        Ok(())
    } else {
        Err(denial(
            WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            entity,
        ))
    }
}
pub(super) fn admit_target(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    operation: &str,
    target: &ApplicationOperationProgramTarget,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if installed_contract_admits_program_target(contracts, target) {
        Ok(())
    } else {
        Err(denial(
            WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
            operation,
        ))
    }
}
pub(super) fn field_locator(
    layout: &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    entity: &str,
    aspect: &str,
    field: &str,
) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
    layout
        .field_locator(entity, aspect, field)
        .cloned()
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field,
            )
        })
}
pub(super) fn require_field_observation(
    routes: &super::super::CompletedFactRoutes,
    facts: &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    id: EntityId,
    locator: &AspectFieldLocator,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    match routes.observes_field(facts, id, locator) {
        Some(true) => Ok(()),
        Some(false) => Err(WorthQueryApplicationAttemptDenial::unobserved_field_write(
            id, locator,
        )),
        None => Err(denial(
            WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
            "completed field observation backing",
        )),
    }
}

impl crate::domain_computation::primary_graph::fragment_view::ErasedCandidate {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn borrowed(
        &self,
    ) -> Target<'_> {
        Target {
            reference: &self.reference,
            entity: &self.entity,
            program: &self.program,
            created_effect: self.created_effect,
        }
    }
}
