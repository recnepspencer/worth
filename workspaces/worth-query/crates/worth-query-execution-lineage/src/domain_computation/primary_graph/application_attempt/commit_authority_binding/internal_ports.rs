use std::any::Any;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationSharedInput;

use super::{WorthQueryApplicationCommitAuthorityBinding, WorthQueryRetainedGovernedInput};

impl crate::ports::CommitAuthorityBindingInternalPort
    for WorthQueryApplicationCommitAuthorityBinding
{
    fn retained_governed_input_carrier(&self) -> Option<&WorthQueryRetainedGovernedInput> {
        Self::retained_governed_input_carrier(self)
    }

    fn original_inspection_disclosure(
        &self,
    ) -> Option<&super::super::super::source_disclosure::OriginalSourceInspectionDisclosure> {
        Self::original_inspection_disclosure(self)
    }

    fn retain_original_inspection_disclosure(
        &self,
    ) -> Option<Arc<super::super::super::source_disclosure::OriginalSourceInspectionDisclosure>>
    {
        Self::retain_original_inspection_disclosure(self)
    }
}

impl crate::ports::RetainedGovernedInputInternalPort for WorthQueryRetainedGovernedInput {
    fn from_admission<Input>(
        input: &ApplicationSharedInput<Input>,
        governed_identity: Option<[u8; 32]>,
        original_scope: Option<&Arc<super::super::super::WorthQueryObservedScopeSelector>>,
        correction: Option<
            &Arc<crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>,
        >,
        captured: Option<&Arc<dyn Any + Send + Sync>>,
        material: Option<(
            &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
            &'static str,
            [u8; 32],
        )>,
    ) -> Self
    where
        Input: Send + Sync + 'static,
    {
        Self::from_admission(
            input,
            governed_identity,
            original_scope,
            correction,
            captured,
            material,
        )
    }

    fn material_input(
        &self,
    ) -> Option<(
        &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
        &'static str,
        [u8; 32],
    )> {
        Self::material_input(self)
    }

    fn retained_input<Binding>(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationRetainedInput<Binding>,
    >
    where
        Binding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding,
    {
        Self::retained_input(self)
    }

    fn governed_identity(&self) -> Option<[u8; 32]> {
        Self::governed_identity(self)
    }

    fn original_scope(&self) -> Option<&Arc<super::super::super::WorthQueryObservedScopeSelector>> {
        Self::original_scope(self)
    }

    fn correction(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>
    {
        Self::correction(self)
    }

    fn axis_probe(governed_identity: [u8; 32]) -> Self {
        Self::axis_probe(governed_identity)
    }
}
