//! Moved whole from `domain_computation::primary_graph::application_contribution::computation::wave_dispatch`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::ComputedSlot;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::PreparedCompute;
use crate::domain_computation::request_execution::WorthQueryAdvancementDenial;
use crate::domain_computation::request_execution::execution_stop::checkpoint_denial;
use crate::domain_computation::request_execution::lease_denial;
use worth_execution::MapKernelFailure;
use worth_execution::MapOutcome;
use worth_execution::MapStop;

// A member outcome cannot stop the map; only its request can.
pub(in crate::domain_computation::primary_graph) enum NoMemberFailure {}

pub struct PreparedMember {
    pub requests: Vec<PreparedCompute>,
}

pub struct ComputedMember {
    slots: Vec<ComputedSlot>,
}

pub(in crate::domain_computation::primary_graph) fn settle_outcome(
    outcome: MapOutcome<ComputedMember, NoMemberFailure>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    match outcome {
        MapOutcome::Complete { values, report: _ } => values.into_iter().map(Ok).collect(),
        MapOutcome::Stopped {
            completed_prefix,
            boundary: _,
            reason,
            report: _,
        } => {
            let mut settled: Vec<_> = completed_prefix.into_iter().map(Ok).collect();
            let failed = match reason {
                MapStop::Failure { identity: _, cause: MapKernelFailure::Domain(impossible) } => match impossible {},
                MapStop::Failure { identity: _, cause: MapKernelFailure::Stop(stop) } => Err(from_checkpoint(stop)),
                MapStop::Failure { identity: _, cause: MapKernelFailure::Panic } => Err(WorthQueryAdvancementDenial::Panicked),
                MapStop::Failure { identity: _, cause: MapKernelFailure::ResultCapacityExceeded } => Err(
                    WorthQueryAdvancementDenial::Resource(crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::ResultCapacityExceeded)),
                MapStop::WorkExhausted { identity: _ } => Err(WorthQueryAdvancementDenial::Resource(
                    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::WorkExhausted)),
                MapStop::Admission(denial) => Err(WorthQueryAdvancementDenial::Resource(lease_denial(denial))),
                MapStop::Allocation(kind) => Err(WorthQueryAdvancementDenial::Resource(
                    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::NativeAllocationRefused {
                        kind, requested_payload_bytes: None,
                    })),
            };
            settled.push(failed);
            settled
        }
    }
}

impl ComputedMember {
    pub(in crate::domain_computation::primary_graph) fn finish<T>(
        self,
        complete: impl FnOnce(Vec<ComputedSlot>) -> T,
    ) -> T {
        complete(self.slots)
    }
    pub(in crate::domain_computation::primary_graph) fn into_slots(self) -> Vec<ComputedSlot> {
        self.finish(|slots| slots)
    }
}

pub(in crate::domain_computation::primary_graph) fn from_checkpoint(stop: worth_execution::MapKernelStop) -> WorthQueryAdvancementDenial {
    match checkpoint_denial(stop) {
        crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
            WorthQueryAdvancementDenial::Resource(denial)
        }
        crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial::Interrupted(denial) => {
            WorthQueryAdvancementDenial::Interrupted(denial)
        }
        crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
            WorthQueryAdvancementDenial::NestedStopped
        }
    }
}

impl ComputedMember {
    #[doc(hidden)]
    pub fn into_slots_for_test(self) -> Vec<ComputedSlot> {
        Self::into_slots(self)
    }
}
