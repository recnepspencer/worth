use worth_store_physical_format::{
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordSlot, PhysicalReferenceAuthority, PhysicalSegmentId,
};
use worth_store_physical_isolation::{
    physical_read_stability_authority_for_certification_test, CurrentGenerationPhysicalReference,
    CurrentPhysicalRoot, GenerationCountedPhysicalReference, PhysicalOrderingContract,
};

pub fn generation_counted_page_reference(generation: u64) -> GenerationCountedPhysicalReference {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(17).unwrap();
    let page = PhysicalPageId::from_raw(23).unwrap();
    let slot = PhysicalRecordSlot::from_raw(1).unwrap();
    let cell = generations
        .slot_cell(segment, page, slot)
        .with_slot_generation(PhysicalGeneration::from_raw(generation).unwrap());
    GenerationCountedPhysicalReference::from_admitted_reference(references.admit_page_slot(cell))
}

pub fn current_generation_page_reference(generation: u64) -> CurrentGenerationPhysicalReference {
    generation_counted_page_reference(generation)
        .require_current_generation(PhysicalGeneration::from_raw(generation).unwrap())
        .unwrap()
}

pub fn current_generation_extent_reference(generation: u64) -> CurrentGenerationPhysicalReference {
    generation_counted_extent_reference(generation)
        .require_current_generation(PhysicalGeneration::from_raw(generation).unwrap())
        .unwrap()
}

pub fn current_generation_segment_reference(generation: u64) -> CurrentGenerationPhysicalReference {
    generation_counted_segment_reference(generation)
        .require_current_generation(PhysicalGeneration::from_raw(generation).unwrap())
        .unwrap()
}

pub fn current_root_from_authority(
    authority: &worth_store_physical_isolation::PhysicalReadStabilityAuthority,
) -> CurrentPhysicalRoot {
    CurrentPhysicalRoot::from_physical_isolation_entry(
        authority.root_epoch_basis().current_root_basis(),
        PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .unwrap()
}

pub fn physical_authority_from_complete_closeout(
) -> worth_store_physical_isolation::PhysicalReadStabilityAuthority {
    physical_read_stability_authority_for_certification_test(
        20,
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    )
}

pub fn physical_authority_from_complete_closeout_for_store(
    store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
) -> worth_store_physical_isolation::PhysicalReadStabilityAuthority {
    physical_read_stability_authority_for_certification_test(
        20,
        store_identity.authority_identity(),
    )
}

pub fn physical_authority_from_operation_digest_closeout(
    operation_digest: &str,
) -> worth_store_physical_isolation::PhysicalReadStabilityAuthority {
    physical_read_stability_authority_for_certification_test(
        certification_root_seed(operation_digest),
        worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
            .authority_identity(),
    )
}

pub fn physical_authority_from_operation_digest_closeout_for_store(
    operation_digest: &str,
    store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
) -> worth_store_physical_isolation::PhysicalReadStabilityAuthority {
    physical_read_stability_authority_for_certification_test(
        certification_root_seed(operation_digest),
        store_identity.authority_identity(),
    )
}

fn generation_counted_extent_reference(generation: u64) -> GenerationCountedPhysicalReference {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(29).unwrap();
    let extent = PhysicalExtentId::from_raw(31).unwrap();
    let cell = generations
        .extent_cell(segment, extent)
        .with_extent_generation(PhysicalGeneration::from_raw(generation).unwrap());
    GenerationCountedPhysicalReference::from_admitted_reference(references.admit_extent(cell))
}

fn generation_counted_segment_reference(generation: u64) -> GenerationCountedPhysicalReference {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(37).unwrap();
    let cell = generations
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(generation).unwrap());
    GenerationCountedPhysicalReference::from_segment_cell(cell)
}

fn certification_root_seed(operation_digest: &str) -> u64 {
    operation_digest
        .bytes()
        .fold(0xcbf29ce484222325, |seed, byte| {
            seed.wrapping_mul(0x100000001b3)
                .wrapping_add(u64::from(byte))
        })
}
