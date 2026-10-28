//! Application lanes guard the two saved references; World owns retirement.
use super::super::{
    BranchCoordinationDenial, WorthQueryAdvancementPhase, WorthQueryApplicationReadObservation,
};
use super::branches::WorthQueryApplicationProductBranches;
use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryCleanupReservationDenial, WorthQueryProductBranchOwnerCleanup,
};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMapDenial, WorkCeilingDenial};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, RuntimeWorldBranchReplacementDenial, RuntimeWorldServiceDenial,
};

/// Every variant refuses before either saved reference is retired.
#[derive(Debug)]
pub enum WorthQueryApplicationHeadReplacementDenial {
    ForeignPhase,
    ReadAdmission(WorthQueryProductBranchAdmissionDenial),
    SameOccurrence,
    CoordinationBusy,
    SizeOverflow,
    WorkCounterOverflow,
    CoordinationStorage(ExecutionOrderedMapDenial),
    CleanupStorage(ExecutionOrderedMapDenial),
    CleanupCapacity,
    CleanupBackingAllocation(std::collections::TryReserveError),
    Request(WorkCeilingDenial),
    PendingPublication,
    OutstandingExternalEffect,
    World(RuntimeWorldServiceDenial<RuntimeWorldBranchReplacementDenial>),
}

/// The old head was retired while both exact saved images were current.
/// Candidate custody remains read-only and does not promise later currentness.
/// Application cleanup is installed and discoverable through `pending_cleanup`;
/// that existing path releases application retention before yielding retries.
#[must_use = "retirement is an effect even when the request's handoff stopped"]
pub struct WorthQueryRetiredApplicationHead {
    candidate: Arc<WorthQueryApplicationReadObservation>,
    retired_head: CompositeCommitIdentity,
    handoff_stop: Option<WorkCeilingDenial>,
    _conditional: Arc<std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>>,
    _cleanup: WorthQueryProductBranchOwnerCleanup,
    _runtime_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryRetiredApplicationHead {
    /// Exact final commit of the retired old reference.
    pub const fn retired_head(&self) -> &CompositeCommitIdentity {
        &self.retired_head
    }
    /// Original-request stop after the retirement effect, if any.
    pub const fn handoff_stop(&self) -> Option<WorkCeilingDenial> {
        self.handoff_stop
    }
    /// Move the original candidate read. Pending cleanup remains runtime-owned.
    pub fn into_candidate(self) -> Arc<WorthQueryApplicationReadObservation> {
        self.candidate
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'_, Schema> {
    /// Retire the exact saved old head only while the exact saved candidate is
    /// also current. Both lanes are attempted once in occurrence order. Busy,
    /// stale, pending-publication and dispatch refusals have no retirement
    /// effect. Success retains actual pending cleanup without running it.
    pub fn retire_replaced_head(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        old: &Arc<WorthQueryApplicationReadObservation>,
        candidate: Arc<WorthQueryApplicationReadObservation>,
    ) -> Result<WorthQueryRetiredApplicationHead, WorthQueryApplicationHeadReplacementDenial> {
        use WorthQueryApplicationHeadReplacementDenial as Denial;
        let request = phase
            .execution_request_for(&self.application.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        let old_image = old
            .lifecycle_observation(self.application)
            .map_err(Denial::ReadAdmission)?;
        let candidate_image = candidate
            .lifecycle_observation(self.application)
            .map_err(Denial::ReadAdmission)?;
        let old_occurrence = old_image.lifecycle_incarnation();
        let candidate_occurrence = candidate_image.lifecycle_incarnation();
        if old_occurrence == candidate_occurrence {
            return Err(Denial::SameOccurrence);
        }
        let (first, second) = if old_occurrence < candidate_occurrence {
            (old_occurrence, candidate_occurrence)
        } else {
            (candidate_occurrence, old_occurrence)
        };
        let provider = &self.application.primary_provider;
        let first = provider
            .application_branch_commit_lane_for_request(first, request)
            .map_err(coordination)?;
        let second = provider
            .application_branch_commit_lane_for_request(second, request)
            .map_err(coordination)?;
        request.consult().map_err(Denial::Request)?;
        let _first = first.try_enter().ok_or(Denial::CoordinationBusy)?;
        request.consult().map_err(Denial::Request)?;
        let _second = second.try_enter().ok_or(Denial::CoordinationBusy)?;
        request.consult().map_err(Denial::Request)?;
        for occurrence in [old_occurrence, candidate_occurrence] {
            // This path never takes a correction command lane under a branch
            // lane, nor performs publication settlement as a hidden effect.
            if provider
                .inspect_pending_application_idempotency(occurrence)
                .is_some()
            {
                return Err(Denial::PendingPublication);
            }
            if provider.has_outstanding_dispatch_for_branch(occurrence) {
                return Err(Denial::OutstandingExternalEffect);
            }
        }
        let cleanup = self
            .application
            .product_runtime
            .prepare_replacement_cleanup(old_image, request)
            .map_err(cleanup_denial)?;
        let retired_head = old_image.selected_commit().clone();
        let conditional = self.application.bridge.conditional_operations();
        let retired = self
            .application
            .product_runtime
            .owner
            .branch_port()
            .retire_replaced_head(request, old_image, candidate_image)
            .map_err(Denial::World)?;
        let (cleanup, handoff_stop, runtime_memory) = cleanup.install(retired);
        // No provider release or component/conditional cleanup runs here:
        // those may allocate or refuse and remain explicit pending work.
        Ok(WorthQueryRetiredApplicationHead {
            candidate,
            retired_head,
            handoff_stop,
            _conditional: conditional,
            _cleanup: cleanup,
            _runtime_memory: runtime_memory,
        })
    }
}

fn coordination(cause: BranchCoordinationDenial) -> WorthQueryApplicationHeadReplacementDenial {
    use WorthQueryApplicationHeadReplacementDenial as Denial;
    match cause {
        BranchCoordinationDenial::Busy => Denial::CoordinationBusy,
        BranchCoordinationDenial::SizeOverflow => Denial::SizeOverflow,
        BranchCoordinationDenial::WorkCounterOverflow => Denial::WorkCounterOverflow,
        BranchCoordinationDenial::Request(cause) => Denial::Request(cause),
        BranchCoordinationDenial::Storage(cause) => Denial::CoordinationStorage(cause),
    }
}

fn cleanup_denial(
    cause: WorthQueryCleanupReservationDenial,
) -> WorthQueryApplicationHeadReplacementDenial {
    use WorthQueryApplicationHeadReplacementDenial as Denial;
    match cause {
        WorthQueryCleanupReservationDenial::Capacity => Denial::CleanupCapacity,
        WorthQueryCleanupReservationDenial::SizeOverflow => Denial::SizeOverflow,
        WorthQueryCleanupReservationDenial::BackingAllocation(cause) => {
            Denial::CleanupBackingAllocation(cause)
        }
        WorthQueryCleanupReservationDenial::Request(cause) => Denial::Request(cause),
        WorthQueryCleanupReservationDenial::Storage(cause) => Denial::CleanupStorage(cause),
    }
}
