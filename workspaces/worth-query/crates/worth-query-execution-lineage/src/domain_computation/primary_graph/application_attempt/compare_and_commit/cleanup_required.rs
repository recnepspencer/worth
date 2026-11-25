//! Business evidence and the real failed cleanup owner travel together.
use super::WorthQueryApplicationCommitOutcome;
pub(crate) mod internal_port;
pub mod managed;
use super::super::business_original as world_association;
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryProductRuntime, WorthQueryMutationCleanupFailure,
};
pub use managed::WorthQueryApplicationManagedCleanup;
use managed::{ErasedCleanupRequiredOwner, ErasedManagedCleanupState, ManagedCleanupState};

/// Original business evidence and actual failed native cleanup travel together.
#[derive(Debug)]
#[must_use = "retain business evidence and native cleanup custody"]
pub struct WorthQueryApplicationCleanupRequired {
    parts: Option<(
        WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationManagedCleanup,
    )>,
    reserved_state: Option<ErasedManagedCleanupState>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryApplicationCleanupRequired {
    pub(crate) fn reserve(
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Box<Self>, worth_execution::ExecutionAllocationDenial> {
        let bytes = std::mem::size_of::<Self>() as u64
            + std::mem::size_of::<Option<ManagedCleanupState>>() as u64;
        let funding = policy.reserve_memory(bytes)?;
        policy.check_live()?;
        Ok(Box::new(Self {
            parts: None,
            reserved_state: Some(ErasedManagedCleanupState::new(None)),
            _funding: funding,
        }))
    }
    pub(crate) fn fill(
        mut self: Box<Self>,
        outcome: WorthQueryApplicationCommitOutcome,
        failure: WorthQueryMutationCleanupFailure,
        runtime: WorthQueryProductRuntime,
        original: Option<super::RetainedOriginalCleanupCommit>,
        bridge: Box<dyn std::any::Any + Send + Sync>,
    ) -> Box<Self> {
        assert!(
            self.parts.is_none(),
            "cleanup envelope is filled only once per handoff"
        );
        let mut state = self
            .reserved_state
            .take()
            .expect("cleanup state was reserved before effects");
        *state = Some(ManagedCleanupState {
            business_original: None,
            history: None,
            publication: None,
            journal_denial: None,
            failure,
            original,
            runtime,
            bridge,
            last_denial: None,
        });
        self.parts = Some((
            outcome,
            WorthQueryApplicationManagedCleanup {
                state,
                return_owner: None,
            },
        ));
        self
    }
    pub(crate) fn from_parts(
        outcome: WorthQueryApplicationCommitOutcome,
        mut cleanup: WorthQueryApplicationManagedCleanup,
    ) -> Box<Self> {
        let mut owner = cleanup
            .return_owner
            .take()
            .expect("split cleanup retains its preallocated envelope")
            .into_boxed();
        assert!(
            owner.parts.is_none(),
            "split envelope has no duplicate business owner"
        );
        owner.parts = Some((outcome, cleanup));
        owner
    }
    pub(crate) fn retain_for_home(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        history: std::sync::Arc<crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory>,
    ) {
        let (outcome, cleanup) = self.parts.as_mut().expect("filled native cleanup");
        cleanup.retain_for_home(request, history, world_association::from_outcome(outcome));
    }
    /// Business evidence retains its native outcome independently of failed cleanup.
    pub fn outcome(&self) -> &WorthQueryApplicationCommitOutcome {
        &self
            .parts
            .as_ref()
            .expect("only filled cleanup envelopes are published")
            .0
    }
    /// Borrows the actual native cleanup owner without transferring authority.
    pub fn cleanup(&self) -> &WorthQueryApplicationManagedCleanup {
        &self
            .parts
            .as_ref()
            .expect("only filled cleanup envelopes are published")
            .1
    }
    /// Transfers both owners without manufacturing business completion.
    pub fn into_parts(
        mut self: Box<Self>,
    ) -> (
        WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationManagedCleanup,
    ) {
        let (outcome, mut cleanup) = self
            .parts
            .take()
            .expect("published cleanup retains both owners");
        cleanup.return_owner = Some(ErasedCleanupRequiredOwner::from_box(self));
        (outcome, cleanup)
    }
}
