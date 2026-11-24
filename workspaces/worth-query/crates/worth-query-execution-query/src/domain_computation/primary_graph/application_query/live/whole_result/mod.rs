//! Whole-result live subscriptions preserving exact negative, related-record,
//! and row invalidation evidence over the installed canonical read graph.
mod delivery;
mod denial;
mod footprint;
mod lifecycle;
mod open;
pub mod outcome;
mod refresh;

pub use outcome::{
    WorthQueryApplicationWholeResultLiveOutcome, WorthQueryApplicationWholeResultLiveSnapshot,
    WorthQueryApplicationWholeResultLiveUpdate,
};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WholeResultLiveStageForTest {
    AfterInitialAuthorityBeforePoll,
    HeadAdvanced,
    AfterPollBeforePendingConfirm,
    AfterPollBeforeOverflowConfirm,
    BeforeFootprintCapture,
    AfterFootprintCaptureBeforeDeliveryConfirm,
}

pub(super) trait WholeResultStageObserver {
    fn on_stage(&mut self, stage: WholeResultLiveStageForTest);
}

pub(super) struct NoopStageObserver;

impl WholeResultStageObserver for NoopStageObserver {
    #[inline(always)]
    fn on_stage(&mut self, _stage: WholeResultLiveStageForTest) {}
}

pub(super) struct CallbackStageObserver<'a>(&'a mut dyn FnMut(WholeResultLiveStageForTest));

impl WholeResultStageObserver for CallbackStageObserver<'_> {
    #[inline(always)]
    fn on_stage(&mut self, stage: WholeResultLiveStageForTest) {
        (self.0)(stage);
    }
}

use std::marker::PhantomData;
use std::rc::Rc;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::controls::WorthQueryApplicationLiveControls;
use crate::domain_computation::{
    managed_run::WorthQueryManagedLowerExecutionBasis,
    primary_graph::{
        live_delivery::WorthQueryLiveSubscription, WorthQueryApplicationEntityIdentity,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};
use footprint::WholeResultInvalidationFootprint;

/// An open whole-result live subscription that delivers the complete freshly
/// projected query result whenever any observed negative-filter,
/// related-record, scope-path, or projected-row fact changes.
///
/// Call `next` with a fresh principal and request for each poll/delivery;
/// every call re-admits and re-authorizes against current truth before polling
/// or delivering. The lease is not `Send`; end it with `close`.
pub struct WorthQueryApplicationWholeResultLiveLease<
    'runtime,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub(super) runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) termination_failed: bool,
    pub(super) query:
        WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    pub(super) scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    pub(super) parameters: ApplicationQueryParameterSet<Query>,
    pub(super) controls: WorthQueryApplicationLiveControls,
    pub(super) opening_branch: worth_runtime_world::facade::ProductBranchIdentity,
    pub(super) opening_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    pub(super) opening_product: Option<crate::basis::WorthQueryProductBranchLease>,
    pub(super) governance: super::super::disclosure::WorthQueryApplicationQueryGovernance,
    pub(super) basis: Option<WorthQueryManagedLowerExecutionBasis>,
    pub(super) read_completion:
        Option<crate::domain_computation::provider_session::WorthQueryGraphReadCompletion>,
    pub(super) subscription: Option<WorthQueryLiveSubscription>,
    pub(super) cursor: u64,
    pub(super) footprint: Option<WholeResultInvalidationFootprint>,
    pub(super) initial: Option<WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>>,
    pub(super) _principal: PhantomData<fn() -> (Principal, PrincipalIdentity)>,
    pub(super) _thread_affinity: PhantomData<Rc<()>>,
}

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryApplicationWholeResultLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    /// Borrows the initial whole-result snapshot captured when this
    /// subscription opened, if not yet taken via [`Self::take_initial`].
    pub fn initial(
        &self,
    ) -> Option<&WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>> {
        self.initial.as_ref()
    }

    /// Takes ownership of the initial whole-result snapshot captured at open.
    pub fn take_initial(
        &mut self,
    ) -> Option<WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>> {
        self.initial.take()
    }
}
