pub(crate) use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

pub(crate) use worth_store_physical_format::PhysicalGeneration;
pub(crate) use {
    worth_store_physical_isolation::admit_seed_stable_read_plan,
    worth_store_physical_isolation::CurrentGenerationPhysicalReference,
    worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial,
    worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics,
    worth_store_physical_isolation::PhysicalReadPlanRetryPosture,
    worth_store_physical_isolation::PostProtectionPhysicalReadObservation,
    worth_store_physical_isolation::ProtectedPhysicalReferenceSet,
    worth_store_physical_isolation::PublishedReaderHazard,
    worth_store_physical_isolation::ReadPlanAdmissionScratchArena,
    worth_store_physical_isolation::StablePhysicalReadPlan,
    worth_store_physical_isolation::TraversalAdmissionGuard,
    worth_store_physical_isolation::UnprotectedReadIntent,
};
pub(crate) use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::generation_counted_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_operation_digest_closeout,
};

pub(crate) fn admit_plan(
    authority: &worth_store_physical_isolation::PhysicalReadStabilityAuthority,
    root: worth_store_physical_isolation::CurrentPhysicalRoot,
    references: ProtectedPhysicalReferenceSet,
    resident_bytes: u64,
    scratch_capacity: usize,
) -> StablePhysicalReadPlan {
    let observed_references = references.clone();
    let intent = worth_store_physical_isolation::UnprotectedReadIntent::for_known_footprint(
        root,
        references,
        resident_bytes,
    )
    .with_release_semantics(
        worth_store_physical_isolation::PhysicalReadPlanReleaseSemantics::reader_releases_all(),
    );
    let hazard =
        worth_store_physical_isolation::PublishedReaderHazard::publish(authority, intent).unwrap();
    let observed = post_protection_observation(authority, &hazard, root, observed_references);
    let validated = hazard
        .observe_authority_after_publication(authority, observed)
        .unwrap()
        .validate()
        .unwrap();
    let receipt = worth_store_physical_isolation::TraversalAdmissionGuard::from_validated_root(validated)
        .admit(worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(scratch_capacity))
        .unwrap();
    admit_seed_stable_read_plan(receipt.into_cursor().finish()).unwrap()
}

pub(crate) fn protected_set<const N: usize>(
    references: [CurrentGenerationPhysicalReference; N],
    scratch_capacity: usize,
) -> ProtectedPhysicalReferenceSet {
    worth_store_physical_isolation::ProtectedPhysicalReferenceSet::from_current_generation_refs_with_scratch(
        references,
        worth_store_physical_isolation::ReadPlanAdmissionScratchArena::for_protected_reference_capacity(scratch_capacity),
    )
    .unwrap()
}

pub(crate) fn post_protection_observation(
    authority: &worth_store_physical_isolation::PhysicalReadStabilityAuthority,
    hazard: &PublishedReaderHazard,
    root: worth_store_physical_isolation::CurrentPhysicalRoot,
    references: ProtectedPhysicalReferenceSet,
) -> PostProtectionPhysicalReadObservation {
    worth_store_physical_isolation::PostProtectionPhysicalReadObservation::from_authority_after_hazard_publication(
        authority, hazard, root, references,
    )
    .unwrap()
}
