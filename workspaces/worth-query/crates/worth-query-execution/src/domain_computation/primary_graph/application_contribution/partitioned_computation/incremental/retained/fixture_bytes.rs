//! A fixture declaration measures custody without reading produced charges.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactReaders, WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact,
};
use std::mem::{align_of, size_of};
use worth_execution::KeyedPartitioner;
use worth_relational::facade::identity::EntityId;

impl RetainedComputation {
    pub(in crate::domain_computation::primary_graph) fn fixture_byte_formula(&self) -> u64 {
        use super::super::super::attribution_tests::{Number, Parity};
        let size = |bytes: usize| u64::try_from(bytes).unwrap();
        // The fixture declares [1,2,3,4], odd/even keys, no membership or
        // key reads, and one reached account in each of its two gathers.
        let items = 4
            * (size(size_of::<(PartitionItemId, Number)>())
                + size(size_of::<(PartitionItemId, [u8; 32])>()));
        let keys = 4 * size(size_of::<(PartitionItemId, RetainedCall)>());
        let parity_bytes = 1 + 1 + size("Parity".len()) + 1 + 1;
        let partitions = 2
            * (size(size_of::<RetainedPartition<Parity>>())
                + parity_bytes
                + size(size_of::<EntityId>()));
        let node = size(size_of::<PartitionIdentity>())
            + 3 * size(size_of::<u64>())
            + 4 * size(size_of::<usize>());
        let facts = fixture_facts_bytes();
        let alignment = align_of::<CustodiedComputation>().max(align_of::<usize>());
        let offset = (2 * size_of::<usize>()).div_ceil(alignment) * alignment;
        let capsule = (offset + size_of::<CustodiedComputation>()).div_ceil(alignment) * alignment;
        items
            + keys
            + partitions
            + KeyedPartitioner::<[u8; 32]>::retained_bytes(4, 2, 0).unwrap()
            + 2 * node
            + facts
            + size(capsule)
    }
}

fn fixture_facts_bytes() -> u64 {
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_layout, AccountLabel, AccountStatus,
    };
    let layout = installed_layout();
    // Both gathers read label; odd alone reads status. Sealed strings are
    // the declared bootstrap inputs "primary" and "open". Each field key
    // owns "Account" and its locator; the observed field owns another locator.
    let label = AccountLabel::reference();
    let status = AccountStatus::reference();
    let locators = [
        layout
            .field_locator(label.entity(), label.aspect(), label.field())
            .unwrap(),
        layout
            .field_locator(status.entity(), status.aspect(), status.field())
            .unwrap(),
    ];
    let entry = size_of::<WorthQueryApplicationFactKey>()
        + size_of::<WorthQueryApplicationObservedFact>()
        + size_of::<ComputationFactReaders>();
    let bytes = 2 * entry
        + 2 * "Account".len()
        + 2 * locators
            .iter()
            .map(|l| l.owned_allocation_capacity_bytes())
            .sum::<usize>()
        + "primary".len()
        + "open".len()
        + 3 * size_of::<PartitionIdentity>();
    u64::try_from(bytes).unwrap()
}
