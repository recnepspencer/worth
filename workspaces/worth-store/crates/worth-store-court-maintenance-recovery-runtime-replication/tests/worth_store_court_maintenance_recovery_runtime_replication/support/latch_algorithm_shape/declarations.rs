pub(crate) use {worth_store_test_support::harness::physical_isolation::epoch_scope as support};

pub(crate) use {std::collections::HashSet};
pub(crate) use {std::hash::BuildHasher, std::hash::Hasher};

pub(crate) use {worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference, worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority, worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout};
pub(crate) use {worth_store_physical_isolation::lower_latch_acquisition_plan, worth_store_physical_isolation::CanonicalLatchAcquisitionOrder, worth_store_physical_isolation::LatchAcquisitionDenial, worth_store_physical_isolation::LatchAcquisitionRequest, worth_store_physical_isolation::LatchAcquisitionStep, worth_store_physical_isolation::LatchUpgradeAuthority, worth_store_physical_isolation::LatchWaitForGraph, worth_store_physical_isolation::LatchWaitForGraphAdmissionDenial, worth_store_physical_isolation::PhysicalLatchClass, worth_store_physical_isolation::PhysicalLatchDeadlockPolicy, worth_store_physical_isolation::PhysicalLatchFamilyDeadlockPolicy, worth_store_physical_isolation::PhysicalLatchKey, worth_store_physical_isolation::PhysicalLatchWaitEdge};

pub(crate) fn mixed_mode_expected_steps() -> Vec<LatchAcquisitionStep> {
    let keys = algorithm_latch_keys();
    let upgrade = upgrade_authority();
    vec![
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.root),
        worth_store_physical_isolation::LatchAcquisitionStep::exclusive(keys.manifest),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.segment_a),
        worth_store_physical_isolation::LatchAcquisitionStep::exclusive(keys.segment_b),
        worth_store_physical_isolation::LatchAcquisitionStep::upgrade(keys.extent_a, upgrade),
        worth_store_physical_isolation::LatchAcquisitionStep::exclusive(keys.extent_b),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.page_a),
        worth_store_physical_isolation::LatchAcquisitionStep::upgrade(keys.page_b, upgrade),
        worth_store_physical_isolation::LatchAcquisitionStep::shared(keys.chunk),
    ]
}

pub(crate) fn lower_steps(steps: Vec<LatchAcquisitionStep>) -> Vec<LatchAcquisitionStep> {
    lower_latch_acquisition_plan(worth_store_physical_isolation::LatchAcquisitionRequest::for_declared_footprint(steps))
        .unwrap()
        .steps()
        .to_vec()
}

pub(crate) fn reversed(mut steps: Vec<LatchAcquisitionStep>) -> Vec<LatchAcquisitionStep> {
    steps.reverse();
    steps
}

pub(crate) fn rotated_left(mut steps: Vec<LatchAcquisitionStep>, by: usize) -> Vec<LatchAcquisitionStep> {
    steps.rotate_left(by);
    steps
}

pub(crate) fn salted_hash_steps(steps: Vec<LatchAcquisitionStep>, salt: u64) -> Vec<LatchAcquisitionStep> {
    let mut rebuilt = std::collections::HashSet::with_hasher(SaltedBuildHasher { salt });
    rebuilt.extend(steps);
    rebuilt.into_iter().collect()
}

pub(crate) fn assert_counter_rows(
    receipt: &worth_store_physical_isolation::LatchCounterPerformanceReceipt,
    expected: &[(&str, u64)],
) {
    let observed: Vec<(&str, u64)> = receipt
        .counter_rows()
        .iter()
        .map(|row| (row.name().as_str(), row.observed_count()))
        .collect();

    assert_eq!(observed, expected);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AlgorithmLatchKeys {
    pub(crate) root: PhysicalLatchKey,
    pub(crate) manifest: PhysicalLatchKey,
    pub(crate) segment_a: PhysicalLatchKey,
    pub(crate) segment_b: PhysicalLatchKey,
    pub(crate) extent_a: PhysicalLatchKey,
    pub(crate) extent_b: PhysicalLatchKey,
    pub(crate) page_a: PhysicalLatchKey,
    pub(crate) page_b: PhysicalLatchKey,
    pub(crate) chunk: PhysicalLatchKey,
}

pub(crate) fn algorithm_latch_keys() -> AlgorithmLatchKeys {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let root_epoch = root.epoch();
    let segment_a = root
        .admit_segment_publication_epoch(current_generation_segment_reference(3))
        .unwrap()
        .epoch();
    let segment_b = root
        .admit_segment_publication_epoch(current_generation_segment_reference(4))
        .unwrap()
        .epoch();
    let extent_a = root
        .admit_extent_publication_epoch(current_generation_extent_reference(5))
        .unwrap()
        .epoch();
    let extent_b = root
        .admit_extent_publication_epoch(current_generation_extent_reference(6))
        .unwrap()
        .epoch();
    let page_a = root
        .admit_page_publication_epoch(current_generation_page_reference(7))
        .unwrap()
        .epoch();
    let page_b = root
        .admit_page_publication_epoch(current_generation_page_reference(8))
        .unwrap()
        .epoch();
    let chunk = root.future_chunk_publication_epoch_placeholder().epoch();

    AlgorithmLatchKeys {
        root: worth_store_physical_isolation::PhysicalLatchKey::root(root_epoch),
        manifest: worth_store_physical_isolation::PhysicalLatchKey::manifest(root_epoch, root.manifest_epoch()),
        segment_a: worth_store_physical_isolation::PhysicalLatchKey::segment(root_epoch, segment_a),
        segment_b: worth_store_physical_isolation::PhysicalLatchKey::segment(root_epoch, segment_b),
        extent_a: worth_store_physical_isolation::PhysicalLatchKey::extent(root_epoch, extent_a),
        extent_b: worth_store_physical_isolation::PhysicalLatchKey::extent(root_epoch, extent_b),
        page_a: worth_store_physical_isolation::PhysicalLatchKey::page(root_epoch, page_a),
        page_b: worth_store_physical_isolation::PhysicalLatchKey::page(root_epoch, page_b),
        chunk: worth_store_physical_isolation::PhysicalLatchKey::future_chunk(root_epoch, chunk),
    }
}

pub(crate) fn upgrade_authority() -> LatchUpgradeAuthority {
    worth_store_physical_isolation::LatchUpgradeAuthority::from_physical_read_stability_authority(
        &physical_authority_from_complete_closeout(),
    )
}

pub(crate) fn all_latch_classes() -> [PhysicalLatchClass; 6] {
    [
        worth_store_physical_isolation::PhysicalLatchClass::Root,
        worth_store_physical_isolation::PhysicalLatchClass::Manifest,
        worth_store_physical_isolation::PhysicalLatchClass::Segment,
        worth_store_physical_isolation::PhysicalLatchClass::Extent,
        worth_store_physical_isolation::PhysicalLatchClass::Page,
        worth_store_physical_isolation::PhysicalLatchClass::FutureChunk,
    ]
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SaltedBuildHasher {
    pub(crate) salt: u64,
}

impl BuildHasher for SaltedBuildHasher {
    type Hasher = SaltedHasher;

    fn build_hasher(&self) -> Self::Hasher {
        SaltedHasher { state: self.salt }
    }
}

pub(crate) struct SaltedHasher {
    pub(crate) state: u64,
}

impl Hasher for SaltedHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = self.state.rotate_left(5) ^ u64::from(*byte);
        }
    }
}