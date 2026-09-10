//! Proof-bearing lifecycle for bound operation projections.
//!
//! Query owns transition admission while ordinary live resources retain
//! registration, continuation, delivery, and teardown mechanics.

mod capability_generation;
mod cleanup_pending;
mod conditional_attempt;
mod conditional_core;
mod counters;
mod denial;
mod lifecycle_basis;
mod lifecycle_close;
mod operational_owner;
mod promotion;
mod promotion_outcome;
mod promotion_preflight;
mod rebind;
mod receipt;
mod refresh;
mod refresh_work;
mod replacement;
mod source;

pub(in crate::domain_installation::operation_execution) use operational_owner::{
    WorthQueryOperationalProjection, WorthQueryOperationalProjectionProof,
};
pub(in crate::domain_installation::operation_execution) use promotion_preflight::{
    admit_projection_promotion_core, WorthQueryProjectionCoreStop,
};
pub(crate) use refresh::refresh_granular_source;
pub(crate) use source::validate_live_source_authority;
pub(in crate::domain_installation::operation_execution) use source::WorthQueryProjectionLifecycleSource;
pub(in crate::domain_installation::operation_execution) use states::WorthQueryLiveProjectionPhase;
mod states;
mod terminal_states;
mod transition_admission;
mod transition_denial;
mod transition_states;
mod transition_terminal;
mod workflow_promotion;
mod workflow_promotion_outcome;
mod workflow_rebind;
mod workflow_replacement;
mod workflow_states;
mod workflow_terminal_states;
mod workflow_transition_states;
mod workflow_transition_terminal;

pub use capability_generation::WorthQueryBoundCapabilityGeneration;
pub use cleanup_pending::WorthQueryProjectionCleanupWork;
pub use counters::WorthQueryProjectionPromotionCounters;
pub use denial::{WorthQueryProjectionPromotionDenialKind, WorthQueryProjectionPromotionStop};
pub use lifecycle_close::{
    WorthQueryProjectionLifecycleCloseCause, WorthQueryProjectionLifecycleCloseReceipt,
    WorthQueryProjectionLifecycleTransitionCounters,
};
pub use promotion_outcome::WorthQueryProjectionPromotionOutcome;
pub use rebind::{
    WorthQueryProjectionRebindOutcome, WorthQueryRebindCleanupRetryOutcome,
    WorthQueryRebindRollbackOutcome,
};
pub use receipt::WorthQueryLiveProjectionReceipt;
pub use refresh::{
    WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshAuthorityStop,
    WorthQueryLiveProjectionRefreshError,
};
pub use refresh_work::WorthQueryLiveProjectionRefreshWork;
pub use replacement::{
    WorthQueryProjectionReplacementOutcome, WorthQueryReplacementCleanupRetryOutcome,
    WorthQueryReplacementRollbackOutcome,
};
pub use states::{
    WorthQueryAuthorityRevalidationDomainProjection, WorthQueryCurrentDomainProjection,
    WorthQueryLiveBoundDomainProjection, WorthQueryRebindRequiredDomainProjection,
    WorthQueryStaleReadableDomainProjection,
};
pub use terminal_states::{
    WorthQueryCancelledDomainProjection, WorthQueryDisposedDomainProjection,
    WorthQueryProjectionCancellationOutcome, WorthQueryProjectionCancellationStop,
    WorthQueryProjectionDisposalOutcome, WorthQueryProjectionDisposalStop,
    WorthQueryProjectionPriorTransitionEvidence,
};
pub use transition_denial::{
    WorthQueryProjectionTransitionDenialKind, WorthQueryProjectionTransitionWork,
};
pub use transition_states::{
    WorthQueryProjectionTransitionStop, WorthQueryRebindCleanupPendingDomainProjection,
    WorthQueryReboundDomainProjection, WorthQueryReplacedDomainProjection,
    WorthQueryReplacementCleanupPendingDomainProjection,
};
pub use transition_terminal::{
    WorthQueryTransitionedProjectionCancellationOutcome, WorthQueryTransitionedProjectionCloseStop,
    WorthQueryTransitionedProjectionDisposalOutcome,
};
pub use workflow_promotion_outcome::{
    WorthQueryWorkflowProjectionPromotionOutcome, WorthQueryWorkflowProjectionPromotionStop,
};
pub use workflow_rebind::{
    WorthQueryWorkflowProjectionRebindOutcome, WorthQueryWorkflowRebindCleanupRetryOutcome,
    WorthQueryWorkflowRebindRollbackOutcome,
};
pub use workflow_replacement::{
    WorthQueryWorkflowProjectionReplacementOutcome,
    WorthQueryWorkflowReplacementCleanupRetryOutcome, WorthQueryWorkflowReplacementRollbackOutcome,
};
pub use workflow_states::{
    WorthQueryAuthorityRevalidationWorkflowProjection, WorthQueryCurrentWorkflowProjection,
    WorthQueryLiveBoundWorkflowProjection, WorthQueryRebindRequiredWorkflowProjection,
    WorthQueryStaleReadableWorkflowProjection,
};
pub use workflow_terminal_states::{
    WorthQueryCancelledWorkflowProjection, WorthQueryDisposedWorkflowProjection,
    WorthQueryWorkflowProjectionCancellationOutcome, WorthQueryWorkflowProjectionCancellationStop,
    WorthQueryWorkflowProjectionDisposalOutcome, WorthQueryWorkflowProjectionDisposalStop,
};
pub use workflow_transition_states::{
    WorthQueryRebindCleanupPendingWorkflowProjection, WorthQueryReboundWorkflowProjection,
    WorthQueryReplacedWorkflowProjection, WorthQueryReplacementCleanupPendingWorkflowProjection,
    WorthQueryWorkflowProjectionTransitionStop,
};
pub use workflow_transition_terminal::{
    WorthQueryTransitionedWorkflowProjectionCancellationOutcome,
    WorthQueryTransitionedWorkflowProjectionCloseStop,
    WorthQueryTransitionedWorkflowProjectionDisposalOutcome,
};
