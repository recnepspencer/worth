//! Native custody for the five approval authorization envelopes.
use super::WorkflowApprovalMeaning;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::RelationalRuntime,
    symbols::{RelationalFieldText, RelationalFieldTextAdmissionDenial as Native},
};

pub(in crate::domain_computation::primary_graph) struct WorkflowApprovalFieldText {
    pub(super) request: RelationalFieldText,
    pub(super) principal: RelationalFieldText,
    pub(super) lineage: RelationalFieldText,
    pub(super) support: RelationalFieldText,
    pub(super) dependencies: RelationalFieldText,
}
impl WorkflowApprovalFieldText {
    // Admission is once per preparation, not once per projection or effect visit.
    // Earlier dictionary admissions may survive a later refusal; the table owns them.
    pub(in crate::domain_computation::primary_graph) fn admit(
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &WorkflowApprovalMeaning,
    ) -> Result<Self, Denial> {
        let receive = |text: &super::super::actor::encoding::FundedActorText| {
            runtime
                .admit_field_text(execution, basis, text.bytes())
                .map_err(denial)
        };
        Ok(Self {
            request: receive(&meaning.authorization_request)?,
            principal: receive(&meaning.authorization_principal)?,
            lineage: receive(&meaning.authorization_lineage)?,
            support: receive(&meaning.authorization_support)?,
            dependencies: receive(&meaning.authorization_dependencies)?,
        })
    }
}
pub(super) fn denial(cause: Native) -> Denial {
    let subject = "workflow approval native field text";
    match cause {
        Native::Allocation(cause) => super::super::actor::storage_denial(cause, subject),
        Native::Stopped(cause) => Denial::relational_read_stopped(cause.into(), subject),
        Native::ForeignBasis => Denial::new(Kind::ForeignApplication, subject),
        Native::StaleBasis => Denial::new(Kind::SourceChanged, subject),
        Native::SymbolExhausted => Denial::new(Kind::CandidateCapacityExceeded, subject),
        Native::InvalidUtf8 | Native::InvalidSymbolDictionary => {
            Denial::new(Kind::InvalidAuthoritativeValue, subject)
        }
    }
}
