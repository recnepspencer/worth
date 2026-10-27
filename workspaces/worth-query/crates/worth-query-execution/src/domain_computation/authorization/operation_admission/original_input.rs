//! Original typed handler input retained independently of authorization mode.
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationSharedInput,
};

use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

pub(in crate::domain_computation::authorization) fn retain_original_scope<Schema, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
        Schema,
        Scope,
    >,
    contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
) -> Result<
    Option<
        std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    if !contracts.source_inspection_disclosures().is_empty() {
        return scope
            .retain_scope_selector_funded(execution)
            .map(Some)
            .map_err(|stop| {
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Stopped(stop),
                    "original source scope retention",
                )
            });
    }
    Ok(contracts
        .aftermath()
        .is_some_and(|contract| {
            contract.authority()
                != worth_query_installation::facade::InstalledCorrectionAuthority::NotCorrectable
        })
        .then(|| scope.retain_scope_selector()))
}

pub(super) struct OriginalOperationInput<Input> {
    encoded: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    program:
        Option<std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>>,
    input: ApplicationSharedInput<Input>,
    identity: [u8; 32],
    binding_type: std::any::TypeId,
    correction: Option<
        std::sync::Arc<
            crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
        >,
    >,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn original_scope(
        &self,
    ) -> Option<
        &std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>,
    > {
        self.original_scope.as_ref()
    }
    pub(in crate::domain_computation) fn retain_original_input<Binding>(
        &mut self,
        input: ApplicationEncodedInput<Binding::InputBinding>,
        correction: Option<
            std::sync::Arc<
                crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
            >,
        >,
    ) where
        Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
        >,
        Input: Send + Sync + 'static,
    {
        // The caller has checked the sealed handler's binding and input identity.
        self.original_input = Some(OriginalOperationInput {
            encoded: std::sync::Arc::new(input.clone()),
            program: None,
            identity: *input.identity(),
            binding_type: std::any::TypeId::of::<Binding>(),
            input: input.into_shared_input(),
            correction,
        });
    }

    pub(in crate::domain_computation) fn original_mutation_binding(
        &self,
    ) -> Option<std::any::TypeId> {
        self.original_input
            .as_ref()
            .map(|input| input.binding_type)
            .or_else(|| {
                self.original_inspection_disclosure
                    .as_ref()
                    .map(|captured| captured.association().source_type())
            })
    }

    pub(in crate::domain_computation) fn retain_original_inspection_disclosure(
        &mut self,
        disclosure: Option<std::sync::Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>,
    ) {
        self.original_inspection_disclosure = disclosure;
    }

    pub(in crate::domain_computation) fn original_inspection_disclosure(
        &self,
    ) -> Option<&std::sync::Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>{
        self.original_inspection_disclosure.as_ref()
    }

    pub(in crate::domain_computation) fn original_encoded_input(
        &self,
    ) -> Option<&std::sync::Arc<dyn std::any::Any + Send + Sync>> {
        self.original_input.as_ref().map(|input| &input.encoded)
    }

    pub(in crate::domain_computation) fn retain_original_program(
        &mut self,
        program: std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>,
    ) {
        self.original_input
            .as_mut()
            .expect("program retention follows original input capture")
            .program = Some(program);
    }

    pub(in crate::domain_computation) fn original_program(
        &self,
    ) -> Option<&std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>>
    {
        self.original_input.as_ref()?.program.as_ref()
    }

    pub(in crate::domain_computation) fn original_correction_disclosure(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
        >,
    > {
        self.original_input.as_ref()?.correction.as_ref()
    }

    pub(in crate::domain_computation) fn original_input(
        &self,
    ) -> Option<(&ApplicationSharedInput<Input>, [u8; 32])> {
        self.original_input
            .as_ref()
            .map(|original| (&original.input, original.identity))
    }
}
