mod body_external_emission;
mod conditional_definition;
mod effect_growth;
mod emission;
mod entity_creation;
mod entity_retirement;
mod entity_selection;
mod model;
mod operation_body;
mod optional_field_authoring;
mod ordinary_field_authoring;
mod original_input;
pub(super) mod output_correspondence;
mod relation_effects;
mod target_admission;
mod write_index;

#[cfg(test)]
mod external_payload_tests;
#[cfg(test)]
mod outbox_persistence_tests;

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationProgramTarget, ApplicationScalarValueBinding,
    DeclaredApplicationFieldValue, OperationCreates, OperationDeletes, OperationWrites,
};
use worth_relational::facade::transactions::EntityReference;
use write_index::FieldWriteIndex;

pub(in crate::domain_computation::primary_graph::application_attempt) use model::WorthQueryApplicationOptionalFieldWrite;
pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};
pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};
pub use model::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEffectProgramBuilder,
};

use super::effect_validation::{canonical_key, denial};
use super::read_set::WorthQueryCompleteApplicationReadSet;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityKey;
pub(in crate::domain_computation::primary_graph) use output_correspondence::{
    OutputRoleUse, WorthQueryApplicationOutputCorrespondence, WorthQueryCheckpointOutputRole,
};
pub use output_correspondence::{
    WorthQueryApplicationOutputEntity, WorthQueryApplicationOutputFamilyEntry,
    WorthQueryApplicationOutputPosture, WorthQueryApplicationOutputProjectionDenial,
    WorthQueryApplicationTypedOutputCorrespondence,
};
use target_admission::installed_contract_admits_program_target;

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
        WorthQueryApplicationEffectProgramBuilder {
            read_set: self,
            layout,
            program: Arc::new(()),
            effects: Vec::new(),
            field_write_positions: FieldWriteIndex::default(),
            keys: BTreeSet::new(),
            conditional_definition: None,
            output_correspondence: Default::default(),
            creation_partition: None,
        }
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
        self.initialize_encoded_field(
            target,
            &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus::new(
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            std::borrow::Cow::Owned(value),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn initialize_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, worth_foundational::facade::AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.initialize_encoded_field(target, field, value)
    }

    fn initialize_encoded_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: std::borrow::Cow<'_, worth_foundational::facade::AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.reserve_effects(0)?;
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
        let Some(WorthQueryApplicationRealizedEffect::CreateEntity { .. }) =
            self.effects.get(created_effect)
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        let WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. } =
            &mut self.effects[created_effect]
        else {
            unreachable!("validated created effect changed before field retention");
        };
        fields.insert(locator, value.into_owned());
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
        self.delete_at_target(
            &ApplicationOperationProgramTarget::Delete {
                entity: entity.name().into(),
            },
            target,
        )
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
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        self.output_correspondence.validate_effects(&self.effects)?;
        self.validate_body_external_emission()?;
        Ok(WorthQueryApplicationEffectProgram {
            read_set: self.read_set,
            effects: self.effects,
            conditional_definition: self.conditional_definition,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Application,
            output_correspondence: self.output_correspondence,
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        })
    }

    /// Intrinsic uniqueness is a native write obligation, not supplier read authority.
    pub(in crate::domain_computation::primary_graph) fn observe_unique_write_dependencies(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.read_set
            .observe_unique_write_values(execution, &self.effects, allocation)
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
