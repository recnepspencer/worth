//! Stack-owned verification for one request, runtime and selected image.
use crate::domain_computation::primary_graph::{
    application_attempt::{
        FactMovement, WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure,
    },
    output_lineage::CertificationBasis,
    source_read_verification::SourceReadVerification,
};
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::RelationalRuntime,
    transactions::RelationalExecutionDenialCause,
};

pub struct RetainedFactVerification<'call, 'request, 'authority> {
    execution: &'call worth_execution::ExecutionRequest<'request, 'authority>,
    runtime: &'call RelationalRuntime,
    basis: CertificationBasis<'call>,
    control: Option<&'call dyn Fn() -> Result<(), RelationalExecutionDenialCause>>,
    reads: Option<SourceReadVerification<'call>>,
}
impl<'call, 'request, 'authority> RetainedFactVerification<'call, 'request, 'authority> {
    pub fn new(
        execution: &'call worth_execution::ExecutionRequest<'request, 'authority>,
        runtime: &'call RelationalRuntime,
        basis: CertificationBasis<'call>,
        control: Option<&'call dyn Fn() -> Result<(), RelationalExecutionDenialCause>>,
    ) -> Self {
        Self {
            execution,
            runtime,
            basis,
            control,
            reads: None,
        }
    }

    pub fn check_live(&self) -> Result<(), RelationalExecutionDenialCause> {
        if let Some(control) = self.control {
            control()?;
        }
        self.execution
            .check_live()
            .map_err(|stop| worth_execution::WorkCeilingDenial::Stopped(stop).into())
    }

    pub fn observe(
        &mut self,
        fact: &WorthQueryApplicationObservedFact,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        fact.source_currentness_with(
            self.runtime,
            self.execution,
            self.basis,
            &mut self.reads,
            self.control,
        )
    }
}
