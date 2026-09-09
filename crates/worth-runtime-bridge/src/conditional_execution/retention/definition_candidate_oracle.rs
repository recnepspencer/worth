//! Independent test oracle for one source-free successor definition.
//! It enumerates retained allocations directly from Rust layouts and does not
//! call the production charge implementation or inspect the ledger total.

use std::alloc::Layout;
use std::mem::{align_of, size_of};
use std::sync::OnceLock;

fn arc_layout(value: Layout) -> u64 {
    Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(value)
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}

fn arc<T>() -> u64 {
    arc_layout(Layout::new::<T>())
}

fn text(bytes: usize) -> u64 {
    arc_layout(Layout::array::<u8>(bytes).unwrap())
}

fn tree<K, V>(entries: usize) -> u64 {
    let alignment = align_of::<K>()
        .max(align_of::<V>())
        .max(align_of::<usize>());
    ((11 * size_of::<K>()
        + 11 * size_of::<V>()
        + 13 * size_of::<usize>()
        + 2 * size_of::<u16>()
        + 5 * alignment)
        * (entries + 1)) as u64
}

pub(crate) fn source_free_successor<Provider, SemanticContract>(
    signal_branch_identity: &str,
    contract_identity: &str,
    location_identity: &str,
    projection_identity_bytes: usize,
    provider_state_bytes: usize,
    semantic_contract_state_bytes: usize,
) -> u64 {
    arc::<super::super::BridgeInstalledConditionalLowering>()
        + arc::<super::super::liveness::BridgeConditionalLoweringLease>()
        + arc::<super::BridgeRetentionReservation>()
        + arc::<worth_signal::facade::branch::SignalBranchIdentity>()
        + signal_branch_identity.len() as u64
        + arc::<OnceLock<worth_signal::facade::branch::SignalConditionalExecutionPort<(), (), ()>>>(
        )
        + arc_layout(
            Layout::array::<crate::correspondence::BridgeInstalledSemanticCorrespondence>(0)
                .unwrap(),
        )
        + tree::<
            super::super::lowering_registry::BridgeConditionalLoweringKey,
            super::super::lowering_registry::BridgeConditionalLoweringSlot,
        >(1)
        + tree::<
            super::super::lowering_registry::BridgeExactConditionalBasisKey,
            std::sync::Arc<super::super::BridgeInstalledConditionalLowering>,
        >(1)
        + 1
        + arc::<Provider>()
        + arc::<SemanticContract>()
        + provider_state_bytes as u64
        + semantic_contract_state_bytes as u64
        + text(projection_identity_bytes)
        + text(contract_identity.len())
        + text(location_identity.len())
}
