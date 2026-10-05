use std::mem::size_of;

use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// A B-tree insertion may split every occupied level and create a root.
pub(in crate::domain_computation::primary_graph::output_lineage) fn tree_insert_bytes<K, V>(
    entries: usize,
) -> Option<u64> {
    let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
    let node = size_of::<(K, V)>()
        .checked_mul(11)?
        .checked_add(size_of::<usize>().checked_mul(16)?)?
        .checked_add(64)?;
    u64::try_from(node.checked_mul(levels.checked_add(2)?)?).ok()
}

pub(in crate::domain_computation::primary_graph::output_lineage) fn tree_work<K>(
    entries: usize,
) -> Option<u64> {
    // An empty tree visits its vacant root. There is no stored key to compare;
    // copied insertion keys are charged separately by the preparing owner.
    if entries == 0 {
        return Some(1);
    }
    let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
    // All current address keys have fixed-width comparison operations. Work
    // counts visited keys; their inline bytes are retained by the tree bound.
    let _key = std::marker::PhantomData::<K>;
    let comparisons = entries.min(11).checked_mul(levels)?.checked_add(1)?;
    u64::try_from(comparisons).ok()
}

pub(in crate::domain_computation::primary_graph::output_lineage) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "prepared output lineage slot exceeded its request admission",
    )
}

pub(in crate::domain_computation::primary_graph::output_lineage) fn arc_bytes<T>() -> Option<u64> {
    let alignment = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
    let header = size_of::<usize>().checked_mul(2)?;
    let offset = header.checked_add(alignment - 1)? / alignment * alignment;
    u64::try_from(
        offset
            .checked_add(size_of::<T>())?
            .checked_add(alignment - 1)?
            / alignment
            * alignment,
    )
    .ok()
}
