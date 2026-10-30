pub(crate) use {
    crate::phase_two_certification_repair::isolated_context::authority,
    crate::phase_two_certification_repair::isolated_context::lease,
    crate::phase_two_certification_repair::isolated_context::map,
    crate::phase_two_certification_repair::isolated_context::TEST_LOCK, std::num::NonZeroUsize,
    std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    worth_execution::CancellationSource, worth_execution::CanonicalBits,
    worth_execution::ChargedBytes, worth_execution::DecomposeCertificationFailure,
    worth_execution::DecomposeKernelEditions, worth_execution::ExecutionDecompose,
    worth_execution::InterfaceSolution, worth_execution::InteriorResult,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure,
    worth_execution::OracleMismatch, worth_execution::ReduceCertificationFailure,
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) static CLONES: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) struct CountClone(pub(crate) u64);

impl Clone for CountClone {
    fn clone(&self) -> Self {
        CLONES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self(self.0)
    }
}

impl ChargedBytes for CountClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for CountClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }

    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.0.to_le_bytes())
    }
}

pub(crate) struct PanicClone(pub(crate) u64);
impl Clone for PanicClone {
    fn clone(&self) -> Self {
        panic!("clone must be contained")
    }
}
impl ChargedBytes for PanicClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl CanonicalBits for PanicClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.0.to_le_bytes())
    }
}
