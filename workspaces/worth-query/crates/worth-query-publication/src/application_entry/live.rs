use worth_query_declaration::facade::application_query::{
    ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveLease,
    WorthQueryApplicationLiveOutcome, WorthQueryApplicationProjection,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionDenialKind,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationRequest;

type Binding<Schema, Intent> =
    <Intent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<
        Schema,
    >>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
type LiveLease<'application, Schema, Intent> = WorthQueryApplicationLiveLease<
    'application,
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Principal<Schema, Intent>,
    PrincipalIdentity<Schema, Intent>,
    Scope<Schema, Intent>,
    <Intent as ApplicationLiveQueryIntent<Schema>>::Target,
    <Intent as ApplicationLiveQueryIntent<Schema>>::LiveCause,
>;

/// Bounds for a live query subscription: its buffer capacity, maximum results and maximum
/// work. Build them with `bounded`.
pub struct WorthQueryApplicationLiveLimits {
    pub(super) buffer_capacity: usize,
    pub(super) maximum_results: usize,
    pub(super) maximum_work: usize,
}

impl WorthQueryApplicationLiveLimits {
    pub const fn bounded(
        buffer_capacity: usize,
        maximum_results: usize,
        maximum_work: usize,
    ) -> Self {
        Self {
            buffer_capacity,
            maximum_results,
            maximum_work,
        }
    }
}

/// Why a live query subscription did not open. `RetainedBasis` means `subscribe` was called
/// on a request pinned to a retained observation.
#[derive(Debug)]
pub enum WorthQueryApplicationLiveOpenRequestDenial {
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    RetainedBasis,
    BindingInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    CapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
    CapabilityAdmission(
        worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenial,
    ),
    Limit(worth_query_installation::facade::WorthQueryApplicationQueryLimitDenial),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
    ScopeResolution(worth_query_execution::facade::primary_graph::WorthQueryEntityResolutionDenial),
    Controls(worth_query_execution::facade::primary_graph::WorthQueryApplicationLiveControlDenial),
    Open(worth_query_execution::facade::primary_graph::WorthQueryApplicationLiveOpenDenial),
}

/// Why a live subscription could not take its next result. `ForeignApplication` and
/// `ForeignBranch` mean the fresh request came from another runtime or branch.
#[derive(Debug)]
pub enum WorthQueryApplicationLiveNextDenial {
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    ForeignApplication,
    ForeignBranch,
    BindingInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
}

/// An open live query subscription. `next` takes the next result against a fresh request;
/// `close` ends it.
pub struct WorthQueryApplicationLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    lease: LiveLease<'application, Schema, Intent>,
}

impl<'application, Schema, Intent>
    WorthQueryApplicationLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub fn buffered_cause_count(&self) -> usize {
        self.lease.buffered_cause_count()
    }

    pub(super) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        lease: LiveLease<'application, Schema, Intent>,
    ) -> Self {
        Self {
            application,
            branch,
            lease,
        }
    }

    pub fn next(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationLiveOutcome<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationLiveNextDenial,
    > {
        // A request for another application or branch is refused before it can open an
        // advancement: its interruption is not this subscription's to observe.
        if !std::ptr::eq(self.application, fresh_request.application) {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignApplication);
        }
        if self.branch != fresh_request.branch {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignBranch);
        }
        // The subscription's terminal law comes before the open: an interrupted request
        // ends the subscription with its own terminal outcome, and a later call reports
        // `Closed`. Neither check reads.
        if let Some(outcome) = self.lease.observe_interruption(fresh_request.scope) {
            return Ok(outcome);
        }
        let result = self
            .application
            .with_application_advancement(fresh_request.scope, |phase| {
                self.next_in_advancement(&phase, fresh_request)
            });
        match result {
            Ok(outcome) => outcome,
            Err(cause) => {
                use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
                match cause {
                    // An interruption that lands between the check above and the open
                    // follows the same terminal law.
                    Denial::Interrupted(_) => {
                        if let Some(outcome) = self.lease.observe_interruption(fresh_request.scope)
                        {
                            return Ok(outcome);
                        }
                    }
                    Denial::Resource(_)
                    | Denial::NestedOpening
                    | Denial::ForeignPhase
                    | Denial::NestedStopped
                    | Denial::Panicked => {}
                }
                Err(WorthQueryApplicationLiveNextDenial::ExecutionRequest(cause))
            }
        }
    }

    fn next_in_advancement(
        &mut self,
        _phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationLiveOutcome<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationLiveNextDenial,
    > {
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Binding<Schema, Intent>>()
            .map_err(WorthQueryApplicationLiveNextDenial::BindingInstallation)?;
        let selected = self
            .application
            .on_branch(fresh_request.branch)
            .select()
            .map_err(WorthQueryApplicationLiveNextDenial::ProductSelection)?;
        let principal = match selected.resolve_authenticated_principal(
            binding.principal_binding(),
            fresh_request.principal,
            fresh_request.scope,
            worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        ) {
            Ok(principal) => principal,
            Err(denial) => match denial.kind() {
                WorthQueryPrincipalResolutionDenialKind::Cancelled
                | WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded => {
                    return Ok(self
                        .lease
                        .observe_interruption(fresh_request.scope)
                        .unwrap_or(WorthQueryApplicationLiveOutcome::Unavailable));
                }
                WorthQueryPrincipalResolutionDenialKind::ForeignRuntime
                | WorthQueryPrincipalResolutionDenialKind::StaleInstalledSchema
                | WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication
                | WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal
                | WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal
                | WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipal
                | WorthQueryPrincipalResolutionDenialKind::MissingPrincipalTarget
                | WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipalTarget
                | WorthQueryPrincipalResolutionDenialKind::WrongPrincipalTargetKind
                | WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof => {
                    return Ok(self.lease.terminate_stale_principal());
                }
                WorthQueryPrincipalResolutionDenialKind::PrimaryGraphNotInstalled
                | WorthQueryPrincipalResolutionDenialKind::BindingNotInstalled
                | WorthQueryPrincipalResolutionDenialKind::BranchMaterializationSuspended
                | WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable
                | WorthQueryPrincipalResolutionDenialKind::ProjectionWorkBudgetExceeded
                | WorthQueryPrincipalResolutionDenialKind::ProjectionPreparationMemoryExhausted
                | WorthQueryPrincipalResolutionDenialKind::CorruptIdentityIndex
                | WorthQueryPrincipalResolutionDenialKind::ActiveSnapshotCapacityExhausted { .. }
                | WorthQueryPrincipalResolutionDenialKind::SnapshotIdentityExhausted
                | WorthQueryPrincipalResolutionDenialKind::RetentionCapacityExhausted
                | WorthQueryPrincipalResolutionDenialKind::RetentionIdentityExhausted => return Err(WorthQueryApplicationLiveNextDenial::PrincipalResolution(denial)),
                // The provider enum is non-exhaustive outside its defining crate.
                _ => return Err(WorthQueryApplicationLiveNextDenial::PrincipalResolution(denial)),
            },
        };
        Ok(self.lease.next(&principal, fresh_request.scope))
    }

    pub fn close(self) -> WorthQueryApplicationLiveCloseOutcome {
        self.lease.close()
    }
}
