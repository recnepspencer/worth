pub(crate) use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) use worth_execution::{
    CancellationToken, CanonicalBits, ChargedBytes, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, MapStop,
    OracleMismatch,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
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

#[test]
fn canonical_encoding_preserves_floating_point_bits_and_field_boundaries() {
    assert_ne!(bits(&0.0_f64), bits(&-0.0_f64));

    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    let other_nan = f64::from_bits(0x7ff8_0000_0000_0043);
    assert_eq!(bits(&nan), bits(&nan));
    assert_ne!(bits(&nan), bits(&other_nan));

    assert_ne!(
        bits(&("a".to_owned(), "bc".to_owned())),
        bits(&("ab".to_owned(), "c".to_owned())),
    );
}
