//! One funded clone retains the actual opaque original observation, never a reader.
use super::super::application_observation::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use std::{
    alloc::Layout,
    sync::{atomic::AtomicUsize, Arc},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};
use worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract;

pub(in crate::domain_computation) struct WorthQueryApplicationInspectionTarget {
    original: WorthQueryApplicationQueryObservation,
    contract: WorthQueryInstalledApplicationInspectionContract,
}

pub(super) enum InspectionTargetDenial {
    ForeignBinding,
    Storage(CaptureStop<RetainedStorageDenial>),
}

impl WorthQueryApplicationInspectionTarget {
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        original: &WorthQueryApplicationQueryObservation,
        contract: WorthQueryInstalledApplicationInspectionContract,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<Self>, InspectionTargetDenial> {
        if original.source.original_binding() != Some(contract.binding_observation()) {
            return Err(InspectionTargetDenial::ForeignBinding);
        }
        run_retained_stage(execution, memory, |context, memory| {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let bytes = clone_bytes(original)
                .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory {
                memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
            }
            Ok(Arc::new(Self {
                original: original.clone(),
                contract,
            }))
        })
        .map_err(InspectionTargetDenial::Storage)
    }

    pub(in crate::domain_computation) fn contract(
        &self,
    ) -> &WorthQueryInstalledApplicationInspectionContract {
        &self.contract
    }

    pub(in crate::domain_computation) fn runtime_authority(&self) -> u64 {
        self.original.source.runtime_authority
    }

    pub(in crate::domain_computation) fn schema_binding(
        &self,
    ) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        &self.original.source.schema_binding
    }

    pub(super) fn matches_original(
        &self,
        original: &WorthQueryApplicationQueryObservation,
    ) -> bool {
        self.original.basis == original.basis
            && self.original.result_count == original.result_count
            && self
                .original
                .source
                .shares_original_custody(&original.source)
    }

    pub(super) fn original(&self) -> &WorthQueryApplicationQueryObservation {
        &self.original
    }
}

fn clone_bytes(original: &WorthQueryApplicationQueryObservation) -> Option<u64> {
    let backing = original.basis.retained_clone_heap_bytes()?;
    let arc = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<WorthQueryApplicationInspectionTarget>())
        .ok()?
        .0
        .pad_to_align()
        .size();
    u64::try_from(arc).ok()?.checked_add(backing)
}
