use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::primary_graph::{
    WholeResultLiveStageForTest, WorthQueryApplicationLiveCloseOutcome,
    WorthQueryApplicationProjection, WorthQueryApplicationWholeResultLiveLease,
    WorthQueryApplicationWholeResultLiveOutcome, WorthQueryApplicationWholeResultLiveSnapshot,
    WorthQueryLiveRuntimeKeepalive, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionDenialKind,
    WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryApplicationLiveNextDenial, WorthQueryApplicationRequest};

type Binding<Schema, Intent> = <Intent as ApplicationQueryIntent<Schema>>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
type WholeResultLiveLease<'application, Schema, Intent> = WorthQueryApplicationWholeResultLiveLease<
    'application,
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Principal<Schema, Intent>,
    PrincipalIdentity<Schema, Intent>,
    Scope<Schema, Intent>,
>;

/// An open whole-result live query subscription preserving exact negative,
/// related-record, scope-path, and row invalidation evidence.
///
/// Inspect the initial snapshot via [`Self::initial`] / [`Self::take_initial`],
/// call [`Self::next`] (or [`Self::next_in_advancement`]) with a fresh request
/// to poll for updates, and end it with [`Self::close`].
pub struct WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    application: LiveApplication<'application, Schema>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    lease: WholeResultLiveLease<'application, Schema, Intent>,
}

/// The runtime a subscription polls: borrowed, or kept alive by the subscription.
enum LiveApplication<'application, Schema> {
    Borrowed(&'application WorthQueryPrimaryGraphApplicationRuntime<Schema>),
    Kept(WorthQueryLiveRuntimeKeepalive<Schema>),
}

impl<Schema> LiveApplication<'_, Schema> {
    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        match self {
            Self::Borrowed(runtime) => runtime,
            Self::Kept(keepalive) => keepalive.application(),
        }
    }
}

/// A whole-result live subscription that keeps its runtime alive instead of
/// borrowing it, so it can be stored beside the runtime's owner.
///
/// Every operation of the borrowed subscription applies unchanged. While it is
/// open the runtime's clean close refuses `RuntimeKeepaliveRetained`; closing
/// or dropping it releases the hold.
pub type WorthQueryOwnedApplicationWholeResultLiveSubscription<Schema, Intent> =
    WorthQueryApplicationWholeResultLiveSubscription<'static, Schema, Intent>;

impl<'application, Schema, Intent>
    WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub(super) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        lease: WholeResultLiveLease<'application, Schema, Intent>,
    ) -> Self {
        Self {
            application: LiveApplication::Borrowed(application),
            branch,
            lease,
        }
    }

    /// Moves this subscription onto `keepalive`'s custody of its own runtime.
    ///
    /// A keepalive of another runtime returns the subscription unchanged.
    pub fn into_owned(
        self,
        keepalive: WorthQueryLiveRuntimeKeepalive<Schema>,
    ) -> Result<WorthQueryOwnedApplicationWholeResultLiveSubscription<Schema, Intent>, Self> {
        if !std::ptr::eq(keepalive.application(), self.application.runtime()) {
            return Err(self);
        }
        let Self {
            application,
            branch,
            lease,
        } = self;
        match lease.into_shared(keepalive.live_backing()) {
            Ok(lease) => Ok(WorthQueryApplicationWholeResultLiveSubscription {
                application: LiveApplication::Kept(keepalive),
                branch,
                lease,
            }),
            Err(lease) => Err(Self {
                application,
                branch,
                lease,
            }),
        }
    }

    pub fn initial(
        &self,
    ) -> Option<
        &WorthQueryApplicationWholeResultLiveSnapshot<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
    > {
        self.lease.initial()
    }

    pub fn take_initial(
        &mut self,
    ) -> Option<
        WorthQueryApplicationWholeResultLiveSnapshot<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
    > {
        self.lease.take_initial()
    }

    pub fn next(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveOutcome<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
        WorthQueryApplicationLiveNextDenial,
    > {
        if !std::ptr::eq(self.application.runtime(), fresh_request.application) {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignApplication);
        }
        if self.branch != fresh_request.branch {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignBranch);
        }
        if let Some(outcome) = self.lease.observe_interruption(fresh_request.scope) {
            return Ok(outcome);
        }
        // The request's runtime is this subscription's own, checked above.
        let result = fresh_request
            .application
            .with_application_advancement(fresh_request.scope, |phase| {
                self.next_in_advancement(&phase, fresh_request)
            });
        match result {
            Ok(outcome) => outcome,
            Err(cause) => {
                use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
                if let Denial::Interrupted(_) = cause
                    && let Some(outcome) = self.lease.observe_interruption(fresh_request.scope)
                {
                    return Ok(outcome);
                }
                Err(WorthQueryApplicationLiveNextDenial::ExecutionRequest(cause))
            }
        }
    }

    pub fn next_in_advancement(
        &mut self,
        phase: &AdvancementPhase<'_>,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveOutcome<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
        WorthQueryApplicationLiveNextDenial,
    > {
        self.next_in_advancement_inner(phase, fresh_request, |lease, phase, principal, scope| {
            lease.next(phase, principal, scope)
        })
    }

    #[doc(hidden)]
    pub fn next_in_advancement_with_stage_hook_for_test(
        &mut self,
        phase: &AdvancementPhase<'_>,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        hook: &mut dyn FnMut(WholeResultLiveStageForTest),
    ) -> Result<
        WorthQueryApplicationWholeResultLiveOutcome<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
        WorthQueryApplicationLiveNextDenial,
    > {
        self.next_in_advancement_inner(phase, fresh_request, |lease, phase, principal, scope| {
            lease.next_with_stage_hook_for_test(phase, principal, scope, hook)
        })
    }

    fn next_in_advancement_inner(
        &mut self,
        phase: &AdvancementPhase<'_>,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        poll: impl FnOnce(
            &mut WholeResultLiveLease<'application, Schema, Intent>,
            &AdvancementPhase<'_>,
            &worth_query_execution::facade::primary_graph::WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal<Schema, Intent>,
                PrincipalIdentity<Schema, Intent>,
            >,
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        ) -> WorthQueryApplicationWholeResultLiveOutcome<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveOutcome<
            Query<Schema, Intent>,
            QueryResult<Schema, Intent>,
        >,
        WorthQueryApplicationLiveNextDenial,
    > {
        if !std::ptr::eq(self.application.runtime(), fresh_request.application) {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignApplication);
        }
        if self.branch != fresh_request.branch {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignBranch);
        }
        if let Some(outcome) = self.lease.observe_interruption(fresh_request.scope) {
            return Ok(outcome);
        }
        let binding = self
            .application
            .runtime()
            .installed_schema()
            .installed_query_binding::<Binding<Schema, Intent>>()
            .map_err(WorthQueryApplicationLiveNextDenial::BindingInstallation)?;
        use worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial as Selection;
        let selected = match self
            .application
            .runtime()
            .on_branch(fresh_request.branch)
            .select(phase)
        {
            Ok(selected) => selected,
            Err(Selection::Stopped(stop)) => return Ok(self.lease.end_with_read_stop(stop)),
            Err(cause @ Selection::Refused(_)) => {
                return Err(WorthQueryApplicationLiveNextDenial::ProductSelection(cause));
            }
        };
        let principal = match selected.resolve_authenticated_principal(
            phase,
            binding.principal_binding(),
            fresh_request.principal,
            fresh_request.scope,
            worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        ) {
            Ok(principal) => principal,
            Err(denial) => match denial.kind() {
                WorthQueryPrincipalResolutionDenialKind::Stopped(stop) => {
                    return Ok(self.lease.end_with_read_stop(stop));
                }
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::ForeignRuntime
                    | PrincipalRefusal::StaleInstalledSchema
                    | PrincipalRefusal::ExpiredAuthentication
                    | PrincipalRefusal::UnknownPrincipal
                    | PrincipalRefusal::DisabledPrincipal
                    | PrincipalRefusal::AmbiguousPrincipal
                    | PrincipalRefusal::MissingPrincipalTarget
                    | PrincipalRefusal::AmbiguousPrincipalTarget
                    | PrincipalRefusal::WrongPrincipalTargetKind
                    | PrincipalRefusal::StalePrincipalProof,
                ) => {
                    return Ok(self.lease.terminate_stale_principal());
                }
                _ => {
                    return Err(WorthQueryApplicationLiveNextDenial::PrincipalResolution(
                        denial,
                    ));
                }
            },
        };
        Ok(poll(
            &mut self.lease,
            phase,
            &principal,
            fresh_request.scope,
        ))
    }

    pub fn close(self) -> WorthQueryApplicationLiveCloseOutcome {
        self.lease.close()
    }
}
