//! Crate-boundary internal port for `WorthQueryApplicationEffectProgram`
//! transition and authority methods consumed by `worth-query-execution-query`
//! and `worth-query-execution`. Never exported on `worth-query-execution::facade` or Host.

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIdentities,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::mvcc::{
    RelationalRequiredOutputPublication, RequiredOutputProducingImplementation,
};
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::dependency_identity::WorthQueryProducerIdentityDenial;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryProducerInvariantRequirement,
};
use crate::ports::{EffectProgramOriginalInputRuntimePort, ProducerIdentityRuntimePort};

pub trait EffectProgramMethodsInternalPort<Schema, Operation, Input, Scope>: Sized {
    fn refuse_stale_home_authority(
        &self,
        current: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;

    fn belongs_to_application(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool;

    fn with_client_observation(self) -> Self;

    fn with_producer_required_invariants(
        self,
        requirements: &'static [WorthQueryProducerInvariantRequirement],
    ) -> Self;

    fn bind_mutation_handler_input<Binding>(
        self,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>;

    fn matches_workflow_authority_binding<Binding>(
        &self,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
    ) -> bool
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>;

    fn retain_admitted_source_input<Binding>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static;

    #[allow(clippy::too_many_arguments)]
    fn coupled_producer_identities<OutputBinding: 'static>(
        &mut self,
        runtime: &dyn ProducerIdentityRuntimePort,
        execution: &ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        implementation: RequiredOutputProducingImplementation,
        predecessor: Option<&RelationalRequiredOutputPublication>,
        policy: ExecutionAllocationPolicy<'_, '_>,
        retain_output_field_evidence: bool,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial>;

    fn retain_original_program(
        self,
        support: &WorthQueryInstalledProgramSupport<Schema>,
        revision: &ApplicationProgramRevision,
    ) -> Result<Self, WorthQueryApplicationCommitDenial>;

    fn validate_current_authority(&self) -> Result<(), WorthQueryOperationAuthorizationDenial>;
}

impl<Schema, Operation, Input, Scope>
    EffectProgramMethodsInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    fn refuse_stale_home_authority(
        &self,
        current: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::refuse_stale_home_authority(self, current)
    }

    fn belongs_to_application(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        Self::belongs_to_application(self, runtime_authority, binding_identity)
    }

    fn with_client_observation(self) -> Self {
        Self::with_client_observation(self)
    }

    fn with_producer_required_invariants(
        self,
        requirements: &'static [WorthQueryProducerInvariantRequirement],
    ) -> Self {
        Self::with_producer_required_invariants(self, requirements)
    }

    fn bind_mutation_handler_input<Binding>(
        self,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        Self::bind_mutation_handler_input(self, identities, execution)
    }

    fn matches_workflow_authority_binding<Binding>(
        &self,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
    ) -> bool
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        Self::matches_workflow_authority_binding::<Binding>(self, idempotency)
    }

    fn retain_admitted_source_input<Binding>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn EffectProgramOriginalInputRuntimePort<Schema>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Input: serde::Serialize + Send + Sync + 'static,
    {
        Self::retain_admitted_source_input::<Binding>(self, phase, runtime, input)
    }

    fn coupled_producer_identities<OutputBinding: 'static>(
        &mut self,
        runtime: &dyn ProducerIdentityRuntimePort,
        execution: &ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        implementation: RequiredOutputProducingImplementation,
        predecessor: Option<&RelationalRequiredOutputPublication>,
        policy: ExecutionAllocationPolicy<'_, '_>,
        retain_output_field_evidence: bool,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial> {
        Self::coupled_producer_identities::<OutputBinding>(
            self,
            runtime,
            execution,
            declared_key,
            implementation,
            predecessor,
            policy,
            retain_output_field_evidence,
        )
    }

    fn retain_original_program(
        self,
        support: &WorthQueryInstalledProgramSupport<Schema>,
        revision: &ApplicationProgramRevision,
    ) -> Result<Self, WorthQueryApplicationCommitDenial> {
        Self::retain_original_program(self, support, revision)
    }

    fn validate_current_authority(&self) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::validate_current_authority(self)
    }
}
