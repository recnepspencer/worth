//! Completion may record facts on a proof, but cannot replace the proof.
use super::{WorthQueryAdmittedApplicationOperation, WorthQueryAdmittedOperation};
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, SealedComputationRetention,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

pub struct ReadCompletionAdmission<'admission> {
    core: &'admission mut WorthQueryAdmittedOperation,
}

impl std::ops::Deref for ReadCompletionAdmission<'_> {
    type Target = WorthQueryAdmittedOperation;

    fn deref(&self) -> &Self::Target {
        self.core
    }
}

impl ReadCompletionAdmission<'_> {
    pub fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: SealedComputationRetention,
    ) {
        self.core
            .record_completed_handler_facts(boundary, computation);
    }

    pub fn retain_execution_canonical_work(
        &mut self,
        work: WorthQueryCanonicalWorkEvidence,
    ) {
        self.core.retain_execution_canonical_work(work);
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[inline]
    pub fn read_completion_loan(
        &mut self,
    ) -> ReadCompletionAdmission<'_> {
        ReadCompletionAdmission {
            core: &mut self.core,
        }
    }
}
