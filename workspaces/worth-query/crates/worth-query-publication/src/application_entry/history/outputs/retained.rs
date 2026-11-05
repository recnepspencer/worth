//! Typed original tree and refusal custody, retained by the document.
use super::*;
use crate::application_entry::{
    WorthQueryApplicationProgramOutputHandle, WorthQueryApplicationProgramOutputSettlement,
    WorthQueryOutputDemandControls,
};
use std::sync::{Arc, Mutex};
pub(super) use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationFixedOutputRoot, ApplicationOutputGraphShape,
    ApplicationProgramDefinition,
};
pub(super) use worth_query_execution::{
    facade::{
        application_contribution::{
            WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
        },
        application_installation::WorthQueryProgramApplicationRuntime,
        primary_graph::{
            WorthQueryApplicationReadObservation, WorthQueryApplicationRequiredOutputConnection,
        },
        product::WorthQueryProductBranch,
    },
    publication_boundary::document_history::{
        WorthQueryHistoryOutputProfile, WorthQueryHistoryOutputSource,
    },
};

pub(super) type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
pub(super) type Demand<S, R> =
    <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
pub(super) type Source<S, R> = <<Demand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source;
pub(super) type RootQuery<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::Query;
pub(super) type Value<S, R> = <<Source<S, R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// The original configured demand and the same output trees survive frontend loss.
/// It stores no admitted query or permission frame.
pub struct WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) application: WorthQueryProgramApplicationRuntime<S, P>,
    pub(super) profile: WorthQueryHistoryOutputProfile,
    pub(super) demand: Demand<S, R>,
    pub(super) controls: WorthQueryOutputDemandControls,
    pub(super) slots: Mutex<std::collections::BTreeMap<[u8; 32], Arc<OutputSlot<S, P, R>>>>,
}

pub(super) struct OutputSlot<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    state: Mutex<SlotState<S, P, R>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<S, P, R> OutputSlot<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn new(memory: Option<worth_execution::ExecutionMemoryReservation>) -> Self {
        Self {
            state: Mutex::new(SlotState::new()),
            _memory: memory,
        }
    }

    pub(super) fn try_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, SlotState<S, P, R>>, WorthQueryHistoryOutputDenial> {
        match self.state.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::WouldBlock) => Err(WorthQueryHistoryOutputDenial::Busy),
            Err(std::sync::TryLockError::Poisoned(poison)) => {
                let mut state = poison.into_inner();
                // Authored unwinding cannot resume a possibly partial tree.
                // Keep its original owners available only for actual disposal.
                state.terminal = true;
                Ok(state)
            }
        }
    }
}

pub(super) struct SlotState<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) original: Option<(
        WorthQueryProductBranch,
        Arc<WorthQueryApplicationReadObservation>,
    )>,
    pub(super) source: Option<WorthQueryHistoryOutputSource>,
    pub(super) tree: Option<WorthQueryApplicationProgramOutputHandle<S, P, R>>,
    pub(super) settlement: Option<WorthQueryApplicationProgramOutputSettlement<RootQuery<S, R>>>,
    pub(super) read_cleanup:
        Option<worth_query_execution::facade::primary_graph::WorthQueryOutputReadCleanup>,
    pub(super) managed_cleanup: Option<(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitOutcome,
        worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup,
    )>,
    pub(super) terminal: bool,
    pub(super) selected: bool,
}

impl<S, P, R> SlotState<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn new() -> Self {
        Self {
            original: None,
            source: None,
            tree: None,
            settlement: None,
            read_cleanup: None,
            managed_cleanup: None,
            terminal: false,
            selected: false,
        }
    }

    pub(super) fn empty(&self) -> bool {
        self.original.is_none()
            && self.source.is_none()
            && self.tree.is_none()
            && self.settlement.is_none()
            && self.read_cleanup.is_none()
            && self.managed_cleanup.is_none()
    }
}
