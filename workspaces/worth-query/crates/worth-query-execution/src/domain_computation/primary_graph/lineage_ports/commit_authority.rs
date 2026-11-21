//! Admission constructor and program downcast on `WorthQueryApplicationCommitAuthorityBinding`.

use std::any::Any;
use std::sync::Arc;

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationIdempotencyBinding,
    WorthQueryRetainedGovernedInput,
};
use crate::domain_computation::primary_graph::OriginalProgramMeaning;

pub(in crate::domain_computation::primary_graph) trait CommitAuthorityBindingExt: Sized {
    fn from_admission<Schema, Operation, Input, Scope>(
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static;

    fn original_program(&self) -> Option<&OriginalProgramMeaning>;
}

impl CommitAuthorityBindingExt for WorthQueryApplicationCommitAuthorityBinding {
    fn from_admission<Schema, Operation, Input, Scope>(
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static,
    {
        let retained_governed_input = match admission.original_input() {
            Some((input, _identity)) => Some(WorthQueryRetainedGovernedInput::from_admission(
                input,
                admission.governed_input_identity().copied(),
                admission.original_scope(),
                admission.original_correction_disclosure(),
                admission.original_retained_input(),
                admission.original_material_input(),
            )),
            None => admission.shared_capability_input().map(|input| {
                WorthQueryRetainedGovernedInput::from_admission(
                    input,
                    admission.governed_input_identity().copied(),
                    admission.original_scope(),
                    admission.original_correction_disclosure(),
                    admission.original_retained_input(),
                    admission.original_material_input(),
                )
            }),
        };
        Self::from_parts(
            admission.original_mutation_binding(),
            admission.runtime_authority(),
            *admission.installed_operation_identity_bytes(),
            admission.allowed_graph_contract().aftermath().cloned(),
            admission.operation_scope_binding().clone(),
            idempotency_binding,
            retained_governed_input,
            admission
                .original_program()
                .map(|program| Arc::clone(program) as Arc<dyn Any + Send + Sync>),
            admission.original_inspection_disclosure().cloned(),
        )
    }

    fn original_program(&self) -> Option<&OriginalProgramMeaning> {
        self.original_program_any()
            .and_then(|program| program.downcast_ref::<OriginalProgramMeaning>())
    }
}
