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
    original: InspectionOriginal,
    contract: WorthQueryInstalledApplicationInspectionContract,
}

enum InspectionOriginal {
    Query {
        original: WorthQueryApplicationQueryObservation,
        before: Option<WorthQueryApplicationQueryObservation>,
    },
    Source(super::source_original::SourceOriginal),
}

pub(super) enum InspectionTargetDenial {
    ForeignBinding,
    Storage(CaptureStop<RetainedStorageDenial>),
}

impl WorthQueryApplicationInspectionTarget {
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        original: &WorthQueryApplicationQueryObservation,
        before: Option<&WorthQueryApplicationQueryObservation>,
        contract: WorthQueryInstalledApplicationInspectionContract,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<Self>, InspectionTargetDenial> {
        if original.source.original_binding() != Some(contract.binding_observation())
            || before.is_some_and(|before| {
                before.source.original_binding() != Some(contract.binding_observation())
            })
        {
            return Err(InspectionTargetDenial::ForeignBinding);
        }
        run_retained_stage(execution, memory, |context, memory| {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let bytes = clone_bytes(original, before)
                .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory {
                memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
            }
            Ok(Arc::new(Self {
                original: InspectionOriginal::Query {
                    original: original.clone(),
                    before: before.cloned(),
                },
                contract,
            }))
        })
        .map_err(InspectionTargetDenial::Storage)
    }

    pub(super) fn capture_source(
        execution: &ExecutionRequest<'_, '_>,
        original: &super::source_original::SourceOriginal,
        contract: WorthQueryInstalledApplicationInspectionContract,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<Self>, InspectionTargetDenial> {
        run_retained_stage(execution, memory, |context, memory| {
            let bytes = Layout::new::<[AtomicUsize; 2]>()
                .extend(Layout::new::<Self>())
                .ok()
                .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
                .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory {
                memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
            }
            Ok(Arc::new(Self {
                original: InspectionOriginal::Source(original.clone()),
                contract,
            }))
        })
        .map_err(InspectionTargetDenial::Storage)
    }
    pub(super) fn source(&self) -> Option<&super::source_original::SourceOriginal> {
        match &self.original {
            InspectionOriginal::Source(source) => Some(source),
            _ => None,
        }
    }
    pub(super) fn parameters(&self) -> worth_foundational::facade::CanonicalDigestId {
        match &self.original {
            InspectionOriginal::Query { original, .. } => {
                original.source.parameter_binding_identity
            }
            InspectionOriginal::Source(source) => {
                *source.disclosure.disclosure().parameters().identity()
            }
        }
    }

    pub(in crate::domain_computation) fn contract(
        &self,
    ) -> &WorthQueryInstalledApplicationInspectionContract {
        &self.contract
    }

    pub(in crate::domain_computation) fn runtime_authority(&self) -> u64 {
        match &self.original {
            InspectionOriginal::Query { original, .. } => original.source.runtime_authority,
            InspectionOriginal::Source(source) => source.runtime,
        }
    }

    pub(in crate::domain_computation) fn schema_binding(
        &self,
    ) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        match &self.original {
            InspectionOriginal::Query { original, .. } => &original.source.schema_binding,
            InspectionOriginal::Source(source) => source.disclosure.schema(),
        }
    }

    pub(super) fn matches_original(
        &self,
        original: &WorthQueryApplicationQueryObservation,
    ) -> bool {
        self.original()
            .is_some_and(|actual| same_original(actual, original))
    }

    pub(super) fn matches_before(&self, before: &WorthQueryApplicationQueryObservation) -> bool {
        self.before()
            .is_some_and(|actual| same_original(actual, before))
    }

    pub(super) fn before(&self) -> Option<&WorthQueryApplicationQueryObservation> {
        match &self.original {
            InspectionOriginal::Query { before, .. } => before.as_ref(),
            InspectionOriginal::Source(_) => None,
        }
    }

    pub(super) fn original(&self) -> Option<&WorthQueryApplicationQueryObservation> {
        match &self.original {
            InspectionOriginal::Query { original, .. } => Some(original),
            InspectionOriginal::Source(_) => None,
        }
    }
}

fn same_original(
    actual: &WorthQueryApplicationQueryObservation,
    original: &WorthQueryApplicationQueryObservation,
) -> bool {
    actual.basis == original.basis
        && actual.result_count == original.result_count
        && actual.source.shares_original_custody(&original.source)
}

fn clone_bytes(
    original: &WorthQueryApplicationQueryObservation,
    before: Option<&WorthQueryApplicationQueryObservation>,
) -> Option<u64> {
    let backing = original
        .basis
        .retained_clone_heap_bytes()?
        .checked_add(match before {
            Some(before) => before.basis.retained_clone_heap_bytes()?,
            None => 0,
        })?;
    let arc = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<WorthQueryApplicationInspectionTarget>())
        .ok()?
        .0
        .pad_to_align()
        .size();
    u64::try_from(arc).ok()?.checked_add(backing)
}
