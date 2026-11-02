//! Owned observation of controls captured from an admitted kernel context.
use super::fixed_backing::{BuildingFixedBacking, OwnedFixedBacking};
use super::{CancellationToken, ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial};
use crate::{ExecutionAllocationDenial, MapKernelStop, WorkCeilingDenial};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionInterruptionViewDenial {
    Request(WorkCeilingDenial),
    Backing(ExecutionAllocationDenial),
}

/// Move-only observation of the admitted scope's original cancellation lineage
/// and absolute deadline. Retained token backing keeps its original memory
/// charge until final drop. This view grants no execution or allocation.
/// It is `Send + Sync + 'static`: callers may share the same view in an `Arc`,
/// keeping its single backing charge until the final shared owner drops.
///
/// ```compile_fail,E0599
/// use worth_execution::ExecutionInterruptionView;
/// fn execute(view: ExecutionInterruptionView) { view.run_native(|_| ()); }
/// ```
/// ```compile_fail,E0599
/// use worth_execution::ExecutionInterruptionView;
/// fn allocate(view: ExecutionInterruptionView) { view.reserve_memory(1); }
/// ```
/// ```compile_fail,E0599
/// use worth_execution::ExecutionInterruptionView;
/// fn inspect_work(view: ExecutionInterruptionView) { view.charged_work(); }
/// ```
/// ```compile_fail,E0599
/// use worth_execution::ExecutionInterruptionView;
/// fn duplicate(view: ExecutionInterruptionView) { let _copy = view.clone(); }
/// ```
pub struct ExecutionInterruptionView {
    tokens: OwnedFixedBacking<CancellationToken>,
    deadline: Option<Instant>,
}

impl ExecutionInterruptionView {
    /// Observe the original signals and OS-clock deadline, independent of scope
    /// lifetime. Work, memory and nested computation stops remain caller-owned.
    pub fn check_live(&self) -> Result<(), MapKernelStop> {
        if self
            .tokens
            .elements()
            .iter()
            .any(CancellationToken::is_cancelled)
        {
            Err(MapKernelStop::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Err(MapKernelStop::DeadlineElapsed)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Debug for ExecutionInterruptionView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionInterruptionView")
            .field("cancellation_signals", &self.tokens.elements().len())
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}

/// Only an already-active descendant may fund a leased projection. A supplied
/// child below the current ancestor must first enter its own existing scope.
pub(crate) fn interruption_backing_lease<'authority>(
    original: Option<&ExecutionResourceLease<'authority>>,
    leased_context: bool,
) -> Result<Option<ExecutionResourceLease<'authority>>, ExecutionInterruptionViewDenial> {
    match (original, leased_context) {
        (Some(original), true) => original
            .current_descendant()
            .map(Some)
            .ok_or_else(unrelated),
        (None, false) => Ok(None),
        (Some(_), false) | (None, true) => Err(unrelated()),
    }
}

pub(crate) fn unrelated() -> ExecutionInterruptionViewDenial {
    ExecutionInterruptionViewDenial::Request(WorkCeilingDenial::Admission(
        crate::backend::request_stop(LeaseDenial::UnrelatedNestedLease),
    ))
}

pub(crate) struct InterruptionViewBuilder {
    tokens: BuildingFixedBacking<CancellationToken>,
    deadline: Option<Instant>,
}

impl InterruptionViewBuilder {
    pub(crate) fn allocate(
        lease: Option<&ExecutionResourceLease<'_>>,
        accounted: bool,
        count: usize,
        deadline: Option<Instant>,
    ) -> Result<Self, ExecutionInterruptionViewDenial> {
        use ExecutionInterruptionViewDenial as Denial;
        let bytes = BuildingFixedBacking::<CancellationToken>::payload_quote(count)
            .map_err(Denial::Backing)?;
        let reservation = if accounted {
            Some(
                ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                    .map_err(|cause| Denial::Request(WorkCeilingDenial::Admission(cause)))?,
            )
        } else {
            None
        };
        let tokens =
            BuildingFixedBacking::allocate_reserved(count, reservation).map_err(Denial::Backing)?;
        Ok(Self { tokens, deadline })
    }

    pub(crate) fn push(
        &mut self,
        token: CancellationToken,
    ) -> Result<(), ExecutionInterruptionViewDenial> {
        self.tokens
            .push(token)
            .map_err(ExecutionInterruptionViewDenial::Backing)
    }

    pub(crate) fn seal(self) -> Result<ExecutionInterruptionView, ExecutionInterruptionViewDenial> {
        Ok(ExecutionInterruptionView {
            tokens: self
                .tokens
                .seal()
                .map_err(ExecutionInterruptionViewDenial::Backing)?,
            deadline: self.deadline,
        })
    }
}
