mod candidate_reservation;
mod candidate_retained_representation;
mod conditional_definition;
mod emission;
mod entity_selection;
mod model;
mod optional_field_authoring;
mod ordinary_field_authoring;
pub(super) mod output_correspondence;
mod relation_effects;
mod reservation_charging;
mod target_admission;

#[cfg(test)]
#[path = "effect_program/candidate_retained_representation_tests.rs"]
mod candidate_retained_representation_tests;
#[cfg(test)]
mod external_payload_tests;
#[cfg(test)]
mod outbox_persistence_tests;

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationProgramTarget, ApplicationScalarValueBinding,
    DeclaredApplicationFieldValue, OperationCreates, OperationDeletes, OperationWrites,
};
use worth_relational::facade::transactions::EntityReference;

pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};
pub use model::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEffectProgramBuilder,
};
pub(super) use model::{
    WorthQueryApplicationOptionalFieldWrite, WorthQueryApplicationRealizedEffect,
};

use super::effect_validation::{canonical_key, denial};
use super::read_set::WorthQueryCompleteApplicationReadSet;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityKey;
pub(in crate::domain_computation::primary_graph) use candidate_reservation::WorthQueryCandidateValidatorWorkAdmission;
use candidate_reservation::{CandidateItemKind, WorthQueryCandidateReservation};
use candidate_retained_representation as retained_representation;
pub use output_correspondence::{
    Create as WorthQueryCreateOutput, Preserve as WorthQueryPreserveOutput,
    Retire as WorthQueryRetireOutput, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputEntity, WorthQueryApplicationOutputPosture,
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationOutputRole,
};
use target_admission::installed_contract_admits_program_target;
use worth_query_declaration::facade::{
    application_operation::ApplicationCandidateRequirements,
    domain_computation::WorthQuerySemanticScaleAxis,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Root-scoped ordinary reads cannot advance to mutation authoring:
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::{
    ///     WorthQueryCompleteApplicationReadSet, WorthQueryOrdinaryApplicationRead,
    /// };
    ///
    /// fn ordinary_read_cannot_author_effects<Schema, Operation, Input, Scope>(
    ///     reads: WorthQueryCompleteApplicationReadSet<
    ///         Schema, Operation, Input, Scope, WorthQueryOrdinaryApplicationRead,
    ///     >,
    /// ) {
    ///     let _ = reads.begin_effect_program();
    /// }
    /// ```
    pub fn begin_effect_program(
        self,
    ) -> WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope> {
        let layout = Arc::clone(&self.lease.layout);
        let emission_retained_bytes_ceiling = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        WorthQueryApplicationEffectProgramBuilder {
            read_set: self,
            layout,
            program: Arc::new(()),
            effects: Vec::new(),
            keys: BTreeSet::new(),
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            candidate_reservation: None,
            output_correspondence: Default::default(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn begin_reserved_effect_program(
        self,
        requested: ApplicationCandidateRequirements,
        ceiling: ApplicationCandidateRequirements,
    ) -> Result<
        WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let envelope = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope();
        let reservation = WorthQueryCandidateReservation::admit(
            requested,
            ceiling,
            envelope.scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems),
            envelope.resource_ceiling(
                WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
            ),
            envelope.scale_ceiling(WorthQuerySemanticScaleAxis::WorkItems),
        )?;
        let capacity = reservation.total_items();
        let layout = Arc::clone(&self.lease.layout);
        let emission_retained_bytes_ceiling =
            envelope.resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        Ok(WorthQueryApplicationEffectProgramBuilder {
            read_set: self,
            layout,
            program: Arc::new(()),
            effects: Vec::with_capacity(capacity),
            keys: BTreeSet::new(),
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
            candidate_reservation: Some(reservation),
            output_correspondence: Default::default(),
        })
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn handler_checkpoint(
        &self,
    ) -> Result<
        (),
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    > {
        self.read_set
            .admission
            .publication_request()
            .interruption()
            .map_or(Ok(()), Err)
    }

    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        let target = ApplicationOperationProgramTarget::Create {
            entity: entity.name().to_string(),
        };
        self.admit_program_target(&target)?;
        let kind = self.layout.entity_kind(entity.name()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                entity.name(),
            )
        })?;
        let key = key.into_string();
        if self.keys.contains(&(kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                entity.name(),
            ));
        }
        let retained_representation_bytes =
            retained_representation::created_entity(&key, entity.name())
                .ok_or_else(candidate_representation_denial)?;
        self.charge_candidate_representation(
            CandidateItemKind::Create,
            retained_representation_bytes,
            0,
        )?;
        self.keys.insert((kind, key.clone()));
        let reference =
            EntityReference::Created(worth_relational::facade::transactions::CreatedEntityRef {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: kind,
                client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
            });
        let created_effect = self.effects.len();
        self.effects
            .push(WorthQueryApplicationRealizedEffect::CreateEntity {
                kind,
                key,
                fields: BTreeMap::new(),
            });
        Ok(WorthQueryApplicationEffectEntity {
            reference,
            entity: entity.name().to_string(),
            created_effect: Some(created_effect),
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }

    pub fn initialize_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        let value = Field::Binding::encode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                field.field(),
            )
        })?;
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        })?;
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let Some(created_effect) = target.created_effect else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        let Some(WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. }) =
            self.effects.get(created_effect)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        let retained_representation_bytes =
            retained_representation::field_entry(&locator, &value, !fields.contains_key(&locator))
                .ok_or_else(candidate_representation_denial)?;
        let replaced_representation_bytes = fields
            .get(&locator)
            .map_or(0, retained_representation::value);
        self.charge_candidate_representation(
            CandidateItemKind::Write,
            retained_representation_bytes,
            replaced_representation_bytes,
        )?;
        let WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. } =
            &mut self.effects[created_effect]
        else {
            unreachable!("validated created effect changed before field retention");
        };
        fields.insert(locator, value);
        Ok(())
    }

    pub fn delete_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationDeletes<Operation>,
    {
        self.validate_target(target, entity.name())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Delete {
            entity: entity.name().to_string(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity.name(),
            ));
        };
        self.charge_candidate_item(CandidateItemKind::Delete)?;
        self.effects
            .push(WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id });
        Ok(())
    }

    pub fn finish(
        self,
    ) -> Result<
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                    self.read_set.admission.operation(),
                )
            })?;
        self.output_correspondence.validate_effects(&self.effects)?;
        let validator_work_admission = self.candidate_reservation.as_ref().map_or_else(
            WorthQueryCandidateValidatorWorkAdmission::unreserved_internal,
            WorthQueryCandidateReservation::validator_work_admission,
        );
        Ok(WorthQueryApplicationEffectProgram {
            read_set: self.read_set,
            effects: self.effects,
            emission_retained_bytes: self.emission_retained_bytes,
            emission_retained_bytes_ceiling: self.emission_retained_bytes_ceiling,
            conditional_definition: self.conditional_definition,
            validator_work_admission,
            output_correspondence: self.output_correspondence,
        })
    }

    fn validate_target<Entity>(
        &self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        entity: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if Arc::ptr_eq(&target.program, &self.program) && target.entity == entity {
            Ok(())
        } else {
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ))
        }
    }

    fn admit_program_target(
        &self,
        target: &ApplicationOperationProgramTarget,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let contracts = self.read_set.admission.allowed_graph_contract();
        if installed_contract_admits_program_target(contracts, target) {
            Ok(())
        } else {
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                self.read_set.admission.operation(),
            ))
        }
    }

    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
        self.layout
            .field_locator(entity, aspect, field)
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    field,
                )
            })
    }
}

fn candidate_representation_denial() -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
        "candidate retained representation overflowed",
    )
}
