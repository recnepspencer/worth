pub(crate) use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

pub(crate) use {
    worth_store_physical_isolation::latch_counter_backed_performance_receipt,
    worth_store_physical_isolation::lower_latch_acquisition_plan,
    worth_store_physical_isolation::pre_wait_denial_for_execution_time_latch_discovery,
    worth_store_physical_isolation::pre_wait_denial_for_hierarchy_inversion,
    worth_store_physical_isolation::pre_wait_denial_for_unauthorized_latch_upgrade,
    worth_store_physical_isolation::pre_wait_denial_for_unordered_latch_set,
    worth_store_physical_isolation::DeadlockPreventionDenial,
    worth_store_physical_isolation::LatchAcquisitionDenial,
    worth_store_physical_isolation::LatchAcquisitionRequest,
    worth_store_physical_isolation::LatchAcquisitionStep,
    worth_store_physical_isolation::LatchUpgradeAuthority,
    worth_store_physical_isolation::LatchWaitCounterSnapshot,
    worth_store_physical_isolation::LatchWaitForGraph,
    worth_store_physical_isolation::PhysicalLatchClass,
    worth_store_physical_isolation::PhysicalLatchKey,
    worth_store_physical_isolation::PhysicalLatchWaitEdge,
};
pub(crate) use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
};

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

pub(crate) fn assert_denial_evidence(
    evidence: &worth_store_physical_isolation::LatchDeniedBeforeWaitEvidence,
    expected: DeadlockPreventionDenial,
    expected_rows: &[(&str, u64)],
) {
    assert_latch_denial_evidence(evidence, expected, expected_rows);
}

pub(crate) fn assert_latch_denial_evidence(
    evidence: &worth_store_physical_isolation::LatchDeniedBeforeWaitEvidence,
    expected: LatchAcquisitionDenial,
    expected_rows: &[(&str, u64)],
) {
    assert_eq!(evidence.denial(), expected);
    assert_counter_rows(evidence.counter_receipt(), expected_rows);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RepresentativeLatchKeys {
    pub(crate) root: PhysicalLatchKey,
    pub(crate) manifest: PhysicalLatchKey,
    pub(crate) segment: PhysicalLatchKey,
    pub(crate) extent: PhysicalLatchKey,
    pub(crate) page: PhysicalLatchKey,
    pub(crate) chunk: PhysicalLatchKey,
}

pub(crate) fn representative_latch_keys() -> RepresentativeLatchKeys {
    let authority = physical_authority_from_complete_closeout();
    let root = current_root_from_authority(&authority);
    let root_epoch = root.epoch();
    let segment = root
        .admit_segment_publication_epoch(current_generation_segment_reference(3))
        .unwrap()
        .epoch();
    let extent = root
        .admit_extent_publication_epoch(current_generation_extent_reference(5))
        .unwrap()
        .epoch();
    let page = root
        .admit_page_publication_epoch(current_generation_page_reference(7))
        .unwrap()
        .epoch();
    let chunk = root.future_chunk_publication_epoch_placeholder().epoch();

    RepresentativeLatchKeys {
        root: worth_store_physical_isolation::PhysicalLatchKey::root(root_epoch),
        manifest: worth_store_physical_isolation::PhysicalLatchKey::manifest(
            root_epoch,
            root.manifest_epoch(),
        ),
        segment: worth_store_physical_isolation::PhysicalLatchKey::segment(root_epoch, segment),
        extent: worth_store_physical_isolation::PhysicalLatchKey::extent(root_epoch, extent),
        page: worth_store_physical_isolation::PhysicalLatchKey::page(root_epoch, page),
        chunk: worth_store_physical_isolation::PhysicalLatchKey::future_chunk(root_epoch, chunk),
    }
}

pub(crate) fn latch_classes(steps: &[LatchAcquisitionStep]) -> Vec<PhysicalLatchClass> {
    steps.iter().map(|step| step.key().class()).collect()
}
