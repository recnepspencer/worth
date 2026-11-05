//! A whole-Parent refresh keeps queue custody and received coverage, never a READ loan.
mod court_control;
mod next;
mod open;
use super::super::{predicate::RetainedPredicateProbe, selection::MaintainedWatchSelection};
use super::*;
use crate::domain_computation::{
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    managed_run::WorthQueryManagedLowerExecutionBasis,
    primary_graph::{
        application_query::derived_view::PredicateRegistration,
        live_delivery::{WorthQueryLivePredicateOpening, WorthQueryLivePredicateQueue},
        WorthQueryCommittedProductPublication,
    },
};
use std::{marker::PhantomData, num::NonZeroUsize, rc::Rc};
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition as Disposition;

/// An invalidation subscription. Each next use supplies a fresh bound request.
/// Dropping it removes its exact predicate and abandons its own queue basis.
pub struct WorthQueryManagedIndexWatch<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    state: Option<WatchState>,
    _binding: PhantomData<fn() -> (S, I)>,
    _thread: PhantomData<Rc<()>>,
}
pub(super) struct WatchState {
    registration: PredicateRegistration,
    queue: WorthQueryLivePredicateQueue,
    basis: Option<WorthQueryManagedLowerExecutionBasis>,
    probe: Arc<RetainedPredicateProbe>,
    selection: MaintainedWatchSelection,
    audience: Audience,
    maximum_results: NonZeroUsize,
    governance: Option<crate::domain_computation::authorization::WorthQueryCapabilityUseIdentity>,
}
#[derive(Eq, PartialEq)]
struct Audience {
    runtime: WorthQueryRuntimeAuthorityIdentity,
    principal: EntityId,
    scope: EntityId,
}
impl Audience {
    fn of(
        plan: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<Self, ReceiverCause> {
        let context = plan
            .retain_query_source_context()
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::GraphRead(cause)))?;
        Ok(Self {
            runtime: context.runtime(),
            principal: context.principal(),
            scope: context.scope(),
        })
    }
}
/// The notification and the refreshed read identify their own actual publications.
/// A notification at B may be served by a fresh current full Parent read at C.
pub struct WorthQueryManagedIndexWatchUpdate<Query, Row> {
    notification: WorthQueryCommittedProductPublication,
    read: WorthQueryApplicationOneShotResult<Query, Row>,
}
impl<Query, Row> WorthQueryManagedIndexWatchUpdate<Query, Row> {
    pub fn notification(&self) -> &WorthQueryCommittedProductPublication {
        &self.notification
    }
    pub fn read(&self) -> &WorthQueryApplicationOneShotResult<Query, Row> {
        &self.read
    }
    pub fn into_read(self) -> WorthQueryApplicationOneShotResult<Query, Row> {
        self.read
    }
}
pub enum WorthQueryManagedIndexWatchOutcome<Query, Row> {
    Delivered(WorthQueryManagedIndexWatchUpdate<Query, Row>),
    Pending,
    Overflow { missed_commit_batches: u64 },
    Unavailable,
    Closed,
}
/// One concrete runtime capture type shares the heavy Parent finalizer between
/// normal managed reads and opening watches; it introduces no mode monomorphs.
pub(super) enum ManagedParentCompletionCapture {
    None,
    Watch {
        state: WatchState,
        _opening: WorthQueryLivePredicateOpening,
    },
}
impl Drop for WatchState {
    fn drop(&mut self) {
        let _ = self.finish(Disposition::Abandoned);
    }
}
impl WatchState {
    fn finish(&mut self, disposition: Disposition) -> bool {
        let Some(basis) = self.basis.as_mut() else {
            return true;
        };
        if self.queue.acknowledge(basis).is_err() {
            return false;
        }
        let WorthQueryManagedLowerExecutionBasis {
            bridge,
            relational,
            product_observation,
        } = self.basis.take().expect("owned live basis");
        match bridge.finalize(disposition) {
            Ok(_) => true,
            Err(failure) => {
                self.basis = Some(WorthQueryManagedLowerExecutionBasis {
                    bridge: failure.into_basis(),
                    relational,
                    product_observation,
                });
                false
            }
        }
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> WorthQueryManagedIndexWatch<S, I> {
    fn terminate(&mut self, disposition: Disposition) -> Result<(), Denial> {
        if self
            .state
            .as_mut()
            .is_some_and(|state| !state.finish(disposition))
        {
            return Err(Denial::new(
                ReceiverCause::Refused(Refusal::TerminalReleaseMismatch),
                None,
            ));
        }
        drop(self.state.take());
        Ok(())
    }
    /// Releases the exact pending occupancy before completing its own queue basis.
    pub fn close(mut self) -> Result<(), Denial> {
        self.terminate(Disposition::Completed)
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> Drop
    for WorthQueryManagedIndexWatch<S, I>
{
    fn drop(&mut self) {
        let _ = self.terminate(Disposition::Abandoned);
    }
}
