pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
};

pub(crate) use {
    worth_execution::CancellationToken, worth_execution::CanonicalBits,
    worth_execution::ChargedBytes, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
    worth_execution::MapPartition, worth_execution::MapStop, worth_execution::OracleMismatch,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) fn bits<T: CanonicalBits>(value: &T) -> Vec<u8> {
    let mut output = Vec::new();
    assert!(value.visit_canonical_bits(&mut |chunk| {
        output.extend_from_slice(chunk);
        true
    }));
    assert_eq!(value.canonical_len(), Some(output.len()));
    output
}

#[derive(Clone, Copy)]
pub(crate) struct DeclaredBits {
    pub(crate) length: usize,
    pub(crate) bytes: &'static [u8],
}

impl ChargedBytes for DeclaredBits {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for DeclaredBits {
    fn canonical_len(&self) -> Option<usize> {
        Some(self.length)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(self.bytes)
    }
}
