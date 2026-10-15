//! Execution-only body read; no raw-field API enters the application facade.
use super::super::super::operation_body::{BodyFieldExpectation, PreparedBodySource};
use super::super::super::{
    application_contribution::advancement_denial, HandlerExecutionDenial,
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};
use super::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding, ApplicationSchema, DecisionReader,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_inputs(
        &self,
        body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
        bindings: &[worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding],
    ) -> super::super::super::HandlerResult<
        Option<super::super::super::operation_body::PreparedBodyInputs>,
        super::super::super::WorthQueryOperationBodyDenial,
    > {
        super::super::super::operation_body::prepare_inputs(
            self.execution,
            self.identities.canonical_input(),
            body,
            bindings,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn body_scope_field(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        expected: BodyFieldExpectation<'_>,
    ) -> Result<bool, HandlerExecutionDenial> {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        self.reader.body_field(self.scope, field, expected)
    }

    pub(in crate::domain_computation::primary_graph) fn body_scope_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        self.body_source_value(field, false)
    }

    pub(in crate::domain_computation::primary_graph) fn body_principal_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        let _ = self.principal_identity();
        self.body_source_value(field, true)
    }

    fn body_source_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        principal: bool,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        if !principal {
            self.context_use.set(self.context_use.get().scope());
        }
        let execution = self.execution;
        let captured = execution.decision_stage(|context| {
            let capture = |raw: &AspectValue| {
                let work = u64::try_from(raw.semantic_byte_width()).map_err(|_| {
                    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                    ))
                })?;
                context.checkpoint(work).map_err(|cause| {
                    HandlerExecutionDenial::new(advancement_denial(
                        worth_execution::WorkCeilingDenial::Stopped(cause),
                    ))
                })?;
                let bytes = raw
                    .owned_allocation_capacity_bytes()
                    .checked_add(size_of::<AspectValue>())
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(|| {
                        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                            WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                        ))
                    })?;
                let reservation = execution.reserve(bytes).map_err(|cause| {
                    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
                })?;
                context.checkpoint(0).map_err(|cause| {
                    HandlerExecutionDenial::new(advancement_denial(
                        worth_execution::WorkCeilingDenial::Stopped(cause),
                    ))
                })?;
                Ok(PreparedBodySource::new(raw.clone(), reservation))
            };
            if principal {
                self.reader.body_principal_value(field, capture)
            } else {
                self.reader.body_value(self.scope, field, capture)
            }
        });
        captured.map_err(HandlerExecutionDenial::new)?
    }

    pub(in crate::domain_computation::primary_graph) fn body_scope_target(
        &mut self,
        body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
    ) -> Result<
        super::super::super::WorthQueryInvariantMutationTarget<
            Schema,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        HandlerExecutionDenial,
    > {
        self.context_use.set(self.context_use.get().scope());
        if body.scope_fields().is_empty() {
            self.checkpoint().map_err(HandlerExecutionDenial::new)?;
            self.reader
                .body_scope_entity(self.scope)
                .map_err(HandlerExecutionDenial::new)?;
        }
        self.reader
            .mutation_target(self.scope)
            .map_err(HandlerExecutionDenial::new)
    }

    pub(in crate::domain_computation::primary_graph) fn body_scope_requirements(
        &mut self,
        body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
    ) -> Result<
        super::super::super::WorthQueryCurrentBodyRequirementAssessment,
        HandlerExecutionDenial,
    > {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        super::super::super::operation_body::evaluate_requirements(
            self.reader,
            self.scope,
            body,
            self.execution,
        )
    }
}
