mod aspect_removal;
mod body_external_emission;
mod case_transition;
mod conditional_definition;
mod content;
mod core;
mod effect_growth;
mod emission;
mod entity_creation;
mod entity_retirement;
mod entity_selection;
mod field_observation;
mod forgery;
mod fragment;
mod installed_reentry;
pub use installed_reentry::WorthQueryCommitEntryRefusal;
pub(crate) mod model;
mod operation_body;
mod optional_field_authoring;
mod ordinary_field_authoring;
mod original_input;
pub(crate) mod output_correspondence;
mod relation_effects;
mod relation_initialization;
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
    ApplicationEntityRef, ApplicationOperationProgramTarget, OperationCreates, OperationDeletes,
};
use worth_relational::facade::transactions::EntityReference;
use write_index::FieldWriteIndex;

pub use model::WorthQueryApplicationOptionalFieldWrite;
pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};
pub use model::{
    WorthQueryApplicationCreatedRelation, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationEffectProgramBuilder,
};
pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
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
    WorthQueryApplicationClosureEvidenceEntry, WorthQueryApplicationOutputEntity,
    WorthQueryApplicationOutputFamilyEntry, WorthQueryApplicationOutputPosture,
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationTypedClosureEvidence,
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
            case_transitions: Default::default(),
            keys: BTreeSet::new(),
            conditional_definition: None,
            output_correspondence: Default::default(),
            creation_partition: None,
            content_attachments: Default::default(),
            content_denial: None,
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

    #[inline(always)]
    pub fn initialize_field<Entity, Field>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Field::Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
        >,
    {
        self.native_effects().initialize_field(
            target.borrowed(),
            field.identity(),
            &crate::domain_computation::primary_graph::value_decode::ScalarFieldEncoder::<
                Field::Binding,
            >::new(&value),
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
        self.native_effects()
            .initialize_encoded_field(target.borrowed(), field, value)
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
        if let Some(cause) = self.content_denial {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::ValueDecode(cause),
                "content authoring",
            ));
        }
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        self.output_correspondence.validate_effects(&self.effects)?;
        self.validate_body_external_emission()?;
        Ok(WorthQueryApplicationEffectProgram {
            read_set: self.read_set,
            effects: self.effects,
            content_attachments: self.content_attachments,
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
        core::validate_target(&self.program, target.borrowed(), entity)
    }

    fn admit_program_target(
        &self,
        target: &ApplicationOperationProgramTarget,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        core::admit_target(
            self.read_set.admission.allowed_graph_contract(),
            self.read_set.admission.operation(),
            target,
        )
    }

    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
        core::field_locator(&self.layout, entity, aspect, field)
    }
}
#[cfg(test)]
#[path = "effect_program_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
