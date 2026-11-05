//! Original handler input retained independently of authorization mode.
use super::WorthQueryAdmittedOperation;
use {crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};

pub(in crate::domain_computation::authorization) fn retain_original_scope<Schema, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
        Schema,
        Scope,
    >,
    contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
) -> Result<
    Option<
        std::sync::Arc<crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector>,
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

/// The encoded original input and its exact identities; the brand holds the
/// typed shared input beside it.
pub(super) struct ErasedOriginalInput {
    pub(super) encoded: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    pub(super) program:
        Option<std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>>,
    pub(super) identity: [u8; 32],
    pub(super) binding_type: std::any::TypeId,
    pub(super) correction: Option<
        std::sync::Arc<
            crate::domain_computation::vocabulary::application_aftermath::original::OriginalCorrectionDisclosure,
        >,
    >,
}

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn original_scope(
        &self,
    ) -> Option<
        &std::sync::Arc<crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector>,
    > {
        self.original_scope.as_ref()
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
        disclosure: Option<std::sync::Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>>,
    ) {
        self.original_inspection_disclosure = disclosure;
    }

    pub(in crate::domain_computation) fn original_inspection_disclosure(
        &self,
    ) -> Option<&std::sync::Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>>{
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
            crate::domain_computation::vocabulary::application_aftermath::original::OriginalCorrectionDisclosure,
        >,
    > {
        self.original_input.as_ref()?.correction.as_ref()
    }

    pub(super) fn original_input_identity(&self) -> Option<[u8; 32]> {
        self.original_input
            .as_ref()
            .map(|original| original.identity)
    }
}
