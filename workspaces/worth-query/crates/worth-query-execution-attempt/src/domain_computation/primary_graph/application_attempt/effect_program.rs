pub mod aspect_removal;
pub mod body_external_emission;
pub mod case_transition;
pub mod content;
pub mod core;
pub use core::EffectEntityBorrowedExt;
pub mod effect_growth;
pub mod emission;
pub mod entity_creation;
pub mod entity_retirement;
pub mod entity_selection;
pub mod field_observation;
pub mod forgery;
pub mod installed_reentry;
pub mod model;
pub mod optional_field_authoring;
pub mod ordinary_field_authoring;
pub mod original_input;
pub mod output_correspondence;
pub mod persistent_names;
pub mod relation_effects;
pub mod relation_initialization;
pub mod same_commit_names;
pub mod target_admission;
pub mod write_index;

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
pub use model::{WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission};
pub use model::{
    WorthQueryApplicationCreatedRelation, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationEffectProgramBuilder,
};
pub use model::{WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect};

use super::effect_validation::{canonical_key, denial};
use super::read_set::WorthQueryCompleteApplicationReadSet;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityKey;
pub use output_correspondence::{
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
            persistent_names: Default::default(),
            creation_partition: None,
            content_attachments: Default::default(),
            content_denial: None,
        }
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(crate) fn handler_checkpoint(
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

    pub(crate) fn initialize_body_field<Entity>(
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
            synchronous_outputs: Vec::new(),
            authored_stages: Vec::new(),
            producer_round_limits: crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::default_producer_round_bounds(),
            read_set: self.read_set,
            effects: self.effects,
            content_attachments: self.content_attachments,
            conditional_definition: self.conditional_definition,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Application,
            persistent_names: self.persistent_names,
            output_correspondence: self.output_correspondence,
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        })
    }

    /// Intrinsic uniqueness is a native write obligation, not supplier read authority.

    pub(crate) fn validate_target<Entity>(
        &self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        entity: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        core::validate_target(&self.program, target.borrowed(), entity)
    }

    pub(crate) fn admit_program_target(
        &self,
        target: &ApplicationOperationProgramTarget,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        core::admit_target(
            self.read_set.admission.allowed_graph_contract(),
            self.read_set.admission.operation(),
            target,
        )
    }

    pub(crate) fn field_locator(
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
pub mod inline_type_budget;

pub mod fragment;

pub mod producer_round_limits;

pub mod conditional_definition;

pub mod operation_body;
