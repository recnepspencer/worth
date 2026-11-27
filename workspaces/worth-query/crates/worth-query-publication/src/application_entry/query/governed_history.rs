//! One governed request reads the record and aspect history of the subjects
//! its SubjectRead discloses: capture, open and bounded paging in a single
//! advancement, every step admitting the read and metadata capabilities afresh
//! for the request's principal. Nothing here widens disclosure: the SubjectRead
//! keeps its own governance and the history metadata contract its own.
use super::super::{WorthQueryApplicationRequest, WorthQueryGovernedReadAccess};
use super::*;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::{ApplicationHistoryQueryBinding, ApplicationQueryScopeBinding},
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationHistoryCaptureLimits,
    WorthQueryApplicationHistoryDenial, WorthQueryApplicationHistoryPage,
    WorthQueryApplicationHistoryPageLimits, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::WorthQueryApplicationHistoryInstallationDenial;

type ScopeEntity<S, I> = <<<I as ApplicationQueryIntent<S>>::Binding as ApplicationQueryBinding<
    S,
>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;

/// Bounds of one governed history read. The walk stops after
/// `maximum_pages` pages even when the history continues.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryGovernedHistoryLimits {
    pub capture: WorthQueryApplicationHistoryCaptureLimits,
    pub page: WorthQueryApplicationHistoryPageLimits,
    pub maximum_pages: NonZeroUsize,
}

/// Whether a governed history read reached the end of the selected history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGovernedHistoryCompleteness {
    Complete,
    /// `maximum_pages` pages were read and the history continues.
    PageLimitReached,
}

/// Why a governed history read stopped. A refusal after the first page means
/// `on_page` already saw the earlier pages; a caller that must expose all or
/// nothing buffers them (the Host's `record_history` returns no partial log).
#[derive(Debug)]
pub enum WorthQueryGovernedHistoryDenial {
    /// The history query is not installed in this program's schema.
    Installation(WorthQueryApplicationHistoryInstallationDenial),
    /// The SubjectRead, a capability or the principal was refused exactly as
    /// `execute_governed` refuses them.
    Request(WorthQueryApplicationRequestQueryDenial),
    /// Native capture, metadata disclosure, opening or paging refused.
    History(WorthQueryApplicationHistoryDenial),
}

impl WorthQueryGovernedHistoryDenial {
    /// Whether the walk stopped at a retired history gap: history before it
    /// is no longer kept. Distinct from every work, capacity, policy or
    /// branch refusal, so an audit view can say "earlier history retired"
    /// without reading `Debug` text.
    pub const fn is_retired_history_gap(&self) -> bool {
        match self {
            Self::History(cause) => cause.is_retired_history_gap(),
            Self::Installation(_) | Self::Request(_) => false,
        }
    }
}

type Access<S, C, O, I> = WorthQueryAdmittedApplicationCapabilityAccess<S, C, O, I>;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::Query>,
{
    /// Reads the governed record and aspect history of every subject this
    /// query discloses, newest step first, handing each page to `on_page`.
    ///
    /// `read` governs the SubjectRead and `metadata` the history's metadata
    /// disclosure contract; each is admitted for this request's principal at
    /// capture, at opening and for every page, so a revocation between pages
    /// refuses the next page (after `on_page` saw the earlier ones). A retained
    /// request (`at(..)`) reads the history visible at its pinned commit;
    /// capabilities are still admitted at the head.
    pub fn read_history_governed_in_advancement<History, RC, RO, RI, MC, MO, MI>(
        self,
        phase: &AdvancementPhase<'_>,
        read: WorthQueryGovernedReadAccess<Schema, RC, RO, RI>,
        metadata: WorthQueryGovernedReadAccess<Schema, MC, MO, MI>,
        limits: WorthQueryGovernedHistoryLimits,
        mut on_page: impl FnMut(&WorthQueryApplicationHistoryPage),
    ) -> Result<WorthQueryGovernedHistoryCompleteness, WorthQueryGovernedHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema, SubjectRead = Intent::Binding>,
        RO: ApplicationOperationMarkerIdentity<Schema> + 'static,
        RO::InputBinding: ApplicationStructuredValueBinding<Value = RI>,
        RI: ApplicationCapabilityRequest<Schema, RC, Scope = ScopeEntity<Schema, Intent>>
            + Clone
            + 'static,
        MO: ApplicationOperationMarkerIdentity<Schema> + 'static,
        MO::InputBinding: ApplicationStructuredValueBinding<Value = MI>,
        MI: ApplicationCapabilityRequest<Schema, MC, Scope = ScopeEntity<Schema, Intent>>
            + Clone
            + 'static,
    {
        use WorthQueryApplicationRequestQueryDenial as Refused;
        use WorthQueryGovernedHistoryDenial as Denial;
        let history = self
            .application
            .installed_schema()
            .installed_history_query::<History>()
            .map_err(Denial::Installation)?;
        let reader = HistoryReader {
            request: WorthQueryApplicationRequest {
                application: self.application,
                principal: self.principal,
                scope: self.scope,
                branch: self.branch,
            },
        };
        let retained = self.retained.clone();
        let subjects = self
            .with_admitted_governed_query(
                phase,
                None,
                read.capability(),
                read.operation(),
                read.input(),
                |application, _binding, _external, plan| {
                    let captured = reader
                        .admit::<Intent::Binding, _, _, _>(phase, &metadata)
                        .and_then(|metadata_access| {
                            let execution = phase
                                .checked_execution_request_for(application.product_runtime())
                                .map_err(|cause| Denial::Request(Refused::ExecutionRequest(cause)))?;
                            let capture = application
                                .admit_governed_application_history_capture(
                                    &execution,
                                    &history,
                                    plan,
                                    metadata_access,
                                    limits.capture,
                                )
                                .map_err(Denial::History)?;
                            application
                                .execute_application_history_capture(phase, capture)
                                .map(|result| result.into_parts().1)
                                .map_err(Denial::History)
                        });
                    Ok(captured)
                },
            )
            .map_err(Denial::Request)??;
        let application = reader.request.application;
        let truth = match retained.as_deref() {
            Some(observation) => application.select_application_read_observation(phase, observation),
            None => application.on_branch(reader.request.branch).select(phase),
        }
        .map_err(|cause| Denial::Request(Refused::ProductSelection(cause)))?;
        let mut cursor = application
            .open_governed_application_history(
                phase,
                &history,
                reader.request.principal,
                &subjects,
                reader.security(phase)?,
                &truth,
                reader.admit::<Intent::Binding, _, _, _>(phase, &read)?,
                reader.admit::<Intent::Binding, _, _, _>(phase, &metadata)?,
                reader.request.scope,
                limits.page,
            )
            .map_err(Denial::History)?;
        for _ in 0..limits.maximum_pages.get() {
            let page = application
                .read_governed_application_history(
                    phase,
                    &history,
                    reader.request.principal,
                    &subjects,
                    &mut cursor,
                    reader.security(phase)?,
                    &truth,
                    reader.admit::<Intent::Binding, _, _, _>(phase, &read)?,
                    reader.admit::<Intent::Binding, _, _, _>(phase, &metadata)?,
                    reader.request.scope,
                    limits.page,
                )
                .map_err(Denial::History)?;
            on_page(&page);
            if page.is_complete() {
                return Ok(WorthQueryGovernedHistoryCompleteness::Complete);
            }
        }
        Ok(WorthQueryGovernedHistoryCompleteness::PageLimitReached)
    }
}

/// The unpinned request every step of one governed history read admits
/// its capabilities and current security against.
struct HistoryReader<'application, 'principal, 'scope, Schema> {
    request: WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>,
}

impl<'application, Schema: ApplicationSchema> HistoryReader<'application, '_, '_, Schema> {
    /// The request branch's current head, against which each step's security
    /// (principal, grants, revocation) is decided.
    fn security(
        &self,
        phase: &AdvancementPhase<'_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'application, Schema>,
        WorthQueryGovernedHistoryDenial,
    > {
        self.request
            .application
            .on_branch(self.request.branch)
            .select(phase)
            .map_err(|cause| {
                WorthQueryGovernedHistoryDenial::Request(
                    WorthQueryApplicationRequestQueryDenial::ProductSelection(cause),
                )
            })
    }

    fn admit<Binding, Capability, Operation, Input>(
        &self,
        phase: &AdvancementPhase<'_>,
        access: &WorthQueryGovernedReadAccess<Schema, Capability, Operation, Input>,
    ) -> Result<Access<Schema, Capability, Operation, Input>, WorthQueryGovernedHistoryDenial>
    where
        Binding: ApplicationQueryBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + 'static,
    {
        self.request
            .admit_read_access_in_advancement::<Binding, _, _, _>(phase, access)
            .map_err(WorthQueryGovernedHistoryDenial::Request)
    }
}
