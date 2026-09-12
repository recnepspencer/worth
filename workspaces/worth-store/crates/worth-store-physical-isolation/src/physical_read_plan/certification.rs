#[cfg(any(test, feature = "certification-authority"))]
use crate::epoch::{manifest_epoch_from_entry_seed, root_epoch_from_entry_seed};
#[cfg(any(test, feature = "certification-authority"))]
use crate::{
    admit_seed_stable_read_plan, lower_latch_acquisition_plan, CurrentPhysicalRoot,
    CurrentPhysicalRootBasis, GenerationCountedPhysicalReference, LatchAcquisitionRequest,
    LatchAcquisitionStep, PhysicalLatchKey, PhysicalOrderingContract, PhysicalReadPlanFootprint,
    PhysicalReadPlanReleaseSemantics, PhysicalReadPlanRetryPosture,
    PhysicalReadReachabilityBarrier, ProtectedPhysicalReferenceSet, ReadPlanAdmissionScratchArena,
    ReadPlanCounterSnapshot, SeedStableReadPlan, StablePhysicalReadPlan,
};
#[cfg(any(test, feature = "certification-authority"))]
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalReferenceAuthority, PhysicalSegmentId,
};

#[cfg(any(test, feature = "certification-authority"))]
pub fn stable_physical_read_plan_for_certification_test(
    guarded_bytes: u64,
) -> StablePhysicalReadPlan {
    stable_physical_read_plan_for_certification_seed(17, guarded_bytes)
}

#[cfg(any(test, feature = "certification-authority"))]
pub fn stable_physical_read_plan_for_certification_seed(
    root_seed: u64,
    guarded_bytes: u64,
) -> StablePhysicalReadPlan {
    let root = current_root_for_certification_seed(root_seed);
    stable_physical_read_plan_for_certification_root(root, guarded_bytes)
}

pub(crate) fn stable_physical_read_plan_for_certification_root(
    root: CurrentPhysicalRoot,
    guarded_bytes: u64,
) -> StablePhysicalReadPlan {
    let reference = current_page_slot_reference_for_certification_test();
    let protected = ProtectedPhysicalReferenceSet::from_current_generation_refs_with_scratch(
        [reference],
        ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1),
    )
    .expect("certification read plan protected set should admit");
    let compact = crate::CompactProtectedReferenceSet::from_reference_set_with_scratch(
        protected,
        ReadPlanAdmissionScratchArena::for_protected_reference_capacity(1),
    )
    .expect("certification read plan compact footprint should admit");
    let footprint = PhysicalReadPlanFootprint::new(compact, guarded_bytes);
    let footprint_basis = footprint.declared_footprint_basis();
    let release = PhysicalReadPlanReleaseSemantics::reader_releases_all();
    let page_epoch = root
        .admit_page_publication_epoch(reference)
        .expect("certification page slot reference should admit")
        .epoch();
    let latch_plan =
        lower_latch_acquisition_plan(LatchAcquisitionRequest::for_declared_footprint(vec![
            LatchAcquisitionStep::shared(PhysicalLatchKey::root(root.epoch())),
            LatchAcquisitionStep::shared(PhysicalLatchKey::manifest(
                root.epoch(),
                root.manifest_epoch(),
            )),
            LatchAcquisitionStep::shared(PhysicalLatchKey::page(root.epoch(), page_epoch)),
        ]))
        .expect("certification latch plan should lower");
    let counters = ReadPlanCounterSnapshot::from_plan(
        &footprint,
        &latch_plan,
        PhysicalReadPlanRetryPosture::Current,
    );
    admit_seed_stable_read_plan(SeedStableReadPlan::new(
        root,
        crate::physical_epoch_vector_for_current_root(root)
            .expect("certification root epoch vector should admit"),
        footprint,
        latch_plan,
        crate::physical_read_plan::PhysicalReadPlanCompletion::new(
            PhysicalReadReachabilityBarrier::from_footprint_basis(footprint_basis, release),
            release,
            PhysicalReadPlanRetryPosture::Current,
            counters,
        ),
    ))
    .expect("certification stable read plan should admit")
}

#[cfg(any(test, feature = "certification-authority"))]
fn current_root_for_certification_seed(seed: u64) -> CurrentPhysicalRoot {
    let basis = CurrentPhysicalRootBasis::new(
        root_epoch_from_entry_seed(seed),
        manifest_epoch_from_entry_seed(seed),
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    );
    CurrentPhysicalRoot::from_physical_isolation_entry(
        basis,
        PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .expect("certification root ordering should admit")
}

#[cfg(any(test, feature = "certification-authority"))]
fn current_page_slot_reference_for_certification_test() -> crate::CurrentGenerationPhysicalReference
{
    let generation = PhysicalGeneration::from_raw(9).expect("generation");
    let slot_cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            PhysicalSegmentId::from_raw(7).expect("segment"),
            PhysicalPageId::from_raw(11).expect("page"),
            PhysicalRecordSlot::from_raw(1).expect("slot"),
        )
        .with_slot_generation(generation);
    GenerationCountedPhysicalReference::from_admitted_reference(
        PhysicalReferenceAuthority::for_canonical_physical_format().admit_page_slot(slot_cell),
    )
    .require_current_generation(generation)
    .expect("certification physical reference should be current")
}

pub(crate) fn read_plan_completion_for_certification_root(
    root: CurrentPhysicalRoot,
    resident_bytes: u64,
) -> super::PhysicalReadPlanCompletionReceipt {
    stable_physical_read_plan_for_certification_root(root, resident_bytes)
        .into_execution_ready_handle()
        .complete_plan()
}
