//! Admission constructor and program downcast on `WorthQueryApplicationCommitAuthorityBinding`.

use std::any::Any;
use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::CommitAuthorityMintInternalPort as _;
use worth_query_execution_lineage::facade::ports::RetainedGovernedInputInternalPort as _;

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationIdempotencyBinding,
    WorthQueryRetainedGovernedInput,
};
use crate::domain_computation::primary_graph::OriginalProgramMeaning;

pub trait CommitAuthorityBindingExt: Sized {
    fn original_program(&self) -> Option<&OriginalProgramMeaning>;
}

impl<Schema, Operation, Input, Scope>
    worth_query_execution_lineage::facade::ports::CommitAuthorityAdmissionPort
    for WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
where
    Input: Clone + Send + Sync + 'static,
{
    fn build_commit_authority_binding(
        &self,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitAuthorityBinding {
        let retained_governed_input = match self.original_input() {
            Some((input, _identity)) => Some(WorthQueryRetainedGovernedInput::from_admission(
                input,
                self.governed_input_identity().copied(),
                self.original_scope(),
                self.original_correction_disclosure(),
                self.original_retained_input(),
                self.original_material_input(),
            )),
            None => self.shared_capability_input().map(|input| {
                WorthQueryRetainedGovernedInput::from_admission(
                    input,
                    self.governed_input_identity().copied(),
                    self.original_scope(),
                    self.original_correction_disclosure(),
                    self.original_retained_input(),
                    self.original_material_input(),
                )
            }),
        };
        WorthQueryApplicationCommitAuthorityBinding::from_parts(
            self.original_mutation_binding(),
            self.runtime_authority(),
            *self.installed_operation_identity_bytes(),
            *self.operation_definition_identity(),
            self.allowed_graph_contract().aftermath().cloned(),
            self.operation_scope_binding().clone(),
            idempotency_binding,
            retained_governed_input,
            self.original_program()
                .map(|program| Arc::clone(program) as Arc<dyn Any + Send + Sync>),
            self.original_inspection_disclosure().cloned(),
        )
    }
}

impl CommitAuthorityBindingExt for WorthQueryApplicationCommitAuthorityBinding {
    fn original_program(&self) -> Option<&OriginalProgramMeaning> {
        self.original_program_any()
            .and_then(|program| program.downcast_ref::<OriginalProgramMeaning>())
    }
}
