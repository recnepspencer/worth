use super::RuntimeWorldBranchAdmissionDenial;
use worth_execution::report::Disposition::*;
use worth_execution::report::RetryHint::WithFreshAuthority;

crate::denial_report::world_denial_report!(RuntimeWorldBranchAdmissionDenial, "branch_admission", {
    Self::HomeCreationRequestRequired => ("home_creation_request_required", InvalidInput, CorrectInput, "Use a Home branch creation request."),
    Self::MemoryCreationRequestRequired => ("memory_creation_request_required", InvalidInput, CorrectInput, "Use a memory branch creation request."),
    Self::ProgramSourceRequired => ("program_source_required", InvalidInput, CorrectInput, "Branch creation requires an admitted program source."),
    Self::ForeignProgramSource => ("foreign_program_source", Authorization, Reauthorize, "The program source belongs to another owner."),
    Self::ProgramSourceMismatch => ("program_source_mismatch", Conflict, CorrectInput, "The program source does not match the selected basis."),
    Self::CancelledBeforeEffect => ("cancelled_before_effect", Cancelled, Final, "Branch admission was cancelled before effects."),
    Self::DeadlineBeforeEffect => ("deadline_before_effect", DeadlineElapsed, Final, "The branch admission deadline elapsed before effects."),
    Self::OwnerUnavailable => ("owner_unavailable", Unavailable, Retry(WithFreshAuthority), "The World owner is unavailable."),
    Self::ForeignOwner => ("foreign_owner", Authorization, Reauthorize, "The branch selection belongs to another World owner."),
    Self::StaleIncarnation => ("stale_incarnation", Conflict, Retry(WithFreshAuthority), "The branch selection belongs to a stale incarnation."),
    Self::RetiredBranch => ("retired_branch", NotFound, Final, "The product branch is retired."),
    Self::DuplicateName => ("duplicate_name", Conflict, CorrectInput, "The product branch name is already in use."),
    Self::CapacityExhausted => ("capacity_exhausted", Exhausted, Final, "The World branch capacity is exhausted."),
    Self::IdentityExhausted => ("identity_exhausted", Exhausted, Escalate, "The World cannot issue another branch identity."),
    Self::PlansOmitted => ("plans_omitted", InvalidInput, CorrectInput, "Branch creation requires explicit plans for both components."),
    Self::UnsupportedRetainedSourcePlan => ("unsupported_retained_source_plan", InvalidInput, CorrectInput, "Retained branch creation requires fork destinations for both components."),
    Self::CustodyCapacityExhausted => ("custody_capacity_exhausted", Exhausted, Final, "The World has no space for another component custody record."),
    Self::StaleSourceHead => ("stale_source_head", Conflict, Retry(WithFreshAuthority), "The source branch head changed after observation."),
    Self::ForkSourceChanged => ("fork_source_changed", Conflict, Retry(WithFreshAuthority), "The component fork source no longer matches the admitted basis."),
    Self::EmptyNativeForkSource => ("empty_native_fork_source", InvalidInput, CorrectInput, "Native requires a published, nonempty fork source."),
    Self::HistoryEntryUnavailable => ("history_entry_unavailable", Unavailable, Final, "The selected history entry is unavailable."),
});

impl std::error::Error for RuntimeWorldBranchAdmissionDenial {}
