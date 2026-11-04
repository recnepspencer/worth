//! Readonly association with the actual first completed root of one admission.
use {crate::domain_computation::primary_graph::output_reuse::read_denial, crate::domain_computation::primary_graph::structural_walk::memory, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind, crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement};
use std::sync::{Arc, OnceLock, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(in crate::domain_computation::primary_graph) struct OriginalProgramRootCompletion {
    root: OnceLock<Weak<WorthQueryOutputDemandSettlement>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl OriginalProgramRootCompletion {
    pub(in crate::domain_computation::primary_graph) fn reserve(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed original root association layout")
            .0
            .pad_to_align();
        let memory = memory::reserve(execution, execution.has_accounting(), layout.size() as u64)
            .map_err(|cause| {
            read_denial(cause).with_subject("original history root association")
        })?;
        Ok(Arc::new(Self {
            root: OnceLock::new(),
            _memory: memory,
        }))
    }

    pub(super) fn completed(&self, root: &Arc<WorthQueryOutputDemandSettlement>) {
        self.root.get_or_init(|| Arc::downgrade(root));
    }

    pub(in crate::domain_computation::primary_graph) fn require_original(
        &self,
        root: &WorthQueryOutputDemandSettlement,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let actual = self.root.get().and_then(Weak::upgrade).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "original history root completion is not retained",
            )
        })?;
        // This compares custody of the Native result actually returned by this
        // admitted root, never a numeric address supplied as semantic identity.
        if !std::ptr::eq(root, actual.as_ref()) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "history output root belongs to another original claim",
            ));
        }
        Ok(())
    }
}
