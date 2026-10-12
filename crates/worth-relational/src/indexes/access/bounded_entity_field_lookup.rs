use super::IndexAccess;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};
use crate::visibility::snapshot_states::resolve_snapshot_handle;

pub(super) mod sealed {
    pub trait Sealed {}
}

/// A sealed exact read input. Snapshot and selected-view inputs preserve their
/// existing caller accounting; neither input retains execution custody.
pub trait BoundedEntityFieldLookupRead: sealed::Sealed {
    type Denial;
    #[doc(hidden)]
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Self::Denial>;
}

impl IndexAccess<'_> {
    /// The one bounded field lookup door; the caller always lends its request.
    pub fn execute_bounded_entity_field_lookup<Read: BoundedEntityFieldLookupRead>(
        &self,
        request: Read,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Read::Denial> {
        request.execute(self, parity, execution)
    }
}

impl sealed::Sealed for BoundedEntityFieldLookupRequest {}
impl BoundedEntityFieldLookupRead for BoundedEntityFieldLookupRequest {
    type Denial = BoundedEntityFieldLookupDenial;
    fn execute(
        self,
        access: &IndexAccess<'_>,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Self::Denial> {
        access.lookup_snapshot(self, parity_mode, execution)
    }
}
impl IndexAccess<'_> {
    fn lookup_snapshot(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        crate::execution::check_request(execution).map_err(|cause| {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::Execution(cause),
                request.index_id(),
            )
        })?;
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let unavailable = || {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                request.index_id(),
            )
        };
        let snapshot =
            resolve_snapshot_handle(self.runtime, request.snapshot()).ok_or_else(unavailable)?;
        let pinned = self
            .runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[request.index_id()], execution)
            .map_err(|stop| {
                use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                let kind = match stop {
                    Pin::SnapshotUnavailable => {
                        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
                    }
                    Pin::IndexNotInstalled(_) => {
                        BoundedEntityFieldLookupDenialKind::IndexNotInstalled
                    }
                    Pin::Request(stop) => BoundedEntityFieldLookupDenialKind::Execution(
                        crate::execution::denial::kernel_stop(stop),
                    ),
                };
                BoundedEntityFieldLookupDenial::new(kind, request.index_id())
            })?;
        pinned.lookup_pinned_entity_field(request, parity_mode, execution)
    }
}

impl crate::runtime::RelationalPinnedRead {
    /// Execute the snapshot-input lane against retained definitions and generations.
    /// Its established liveness/accounting policy is unchanged.
    pub fn execute_bounded_entity_field_lookup(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        crate::execution::check_request(execution).map_err(|cause| {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::Execution(cause),
                request.index_id(),
            )
        })?;
        self.performance_access().count_query_index_attempt();
        self.lookup_pinned_entity_field(request, parity_mode, execution)
    }
    fn lookup_pinned_entity_field(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        let unavailable = || {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                request.index_id(),
            )
        };
        if request.snapshot() != self.snapshot() {
            return Err(unavailable());
        }
        let prepared = self.prepare_entity_field_lookup(
            request.index_id(),
            request.entity_kind(),
            request.field_locator(),
            execution,
        )?;
        prepared
            .collect_prepared_entity_field_lookup(
                request.value(),
                request.candidate_limit(),
                parity_mode,
                execution,
                || Ok::<_, std::convert::Infallible>(()),
            )
            .map_err(|stop| match stop {
                Stop::Lookup(denial) => denial,
                Stop::Admission(never) => match never {},
                Stop::ExactBasisRequired | Stop::AccountingOverflow => unavailable(),
            })
    }
}
