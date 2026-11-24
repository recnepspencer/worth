use crate::history::data::coordinates::CommitId;

#[derive(Debug)]
pub enum RelationalRecordedComponentRefusal {
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    Basis(crate::branch::basis_denial::RelationalBranchBasisDenial),
    Frame(crate::branch::reference::observation::RelationalBranchObservationConstructionDenial),
    Source(crate::durability::data::recovery_stop::RelationalRecoveryStop),
    MissingPerformedCommit { commit_id: CommitId },
    CanonicalMismatch { commit_id: CommitId },
    PayloadEncoding { detail: String },
    PayloadBacking(worth_execution::ExecutionRequestArrayDenial),
}
