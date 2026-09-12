//! Local compaction-plan fixtures. No Store I/O, live reader retention, byte
//! validation, or recovery execution is claimed by these synthetic inputs.

use worth_store_physical_isolation::{
    next_root_epoch_for_certification, stable_physical_read_plan_for_certification_seed,
    CompactionCandidateRangeSet, CompactionCutoverDelta, CompactionProtectedReferenceSet,
    CompactionReadInterlockPlan, CompactionRewritePublication, CompactionSourceIntegrityAdmission,
    CompactionSourceIntegrityEvidence, NewRootPublicationProof, OldReachabilityPreservation,
    PhysicalPublicationIntent, PhysicalPublicationReadiness, PublicationLatchReadiness,
    PublicationRootCandidate, RootSwapOrderingContract,
};

pub fn admitted_compaction_plan() -> CompactionReadInterlockPlan {
    admitted_compaction_plan_for_seed(17)
}

pub fn admitted_compaction_plan_for_seed(seed: u64) -> CompactionReadInterlockPlan {
    let read = stable_physical_read_plan_for_certification_seed(seed, 8);
    let source = read.root().epoch();
    let reference = read.footprint().protected().references()[0].current_generation();
    let protected = CompactionProtectedReferenceSet::from_read_plan(&read);
    let receipt = read.into_execution_ready_handle().complete_plan();
    let integrity =
        CompactionSourceIntegrityEvidence::from_stable_read_receipt_and_integrity_admission(
            receipt,
            CompactionSourceIntegrityAdmission::for_certification_test(reference.owner(), 8)
                .unwrap(),
        )
        .unwrap();
    CompactionReadInterlockPlan::admit(
        protected,
        CompactionCandidateRangeSet::from_current_generation_refs([reference]).unwrap(),
        source,
        next_root_epoch_for_certification(source),
        integrity,
    )
    .unwrap()
}

pub fn published_compaction(plan: CompactionReadInterlockPlan) -> CompactionRewritePublication {
    let manifest = plan.protected().root().manifest_epoch().get() + 1;
    published_compaction_at_manifest(plan, manifest)
}

pub fn published_compaction_at_manifest(
    plan: CompactionReadInterlockPlan,
    manifest: u64,
) -> CompactionRewritePublication {
    let delta = CompactionCutoverDelta::lower_to_manifest(plan, manifest).unwrap();
    let old_root = delta.plan().protected().root();
    let new_root = delta.rewritten_root();
    let validation = super::publication::root_publication_validation(new_root.scope(), 2);
    let old = PublicationRootCandidate::admit(
        old_root,
        super::publication::root_publication_validation(old_root.scope(), 1),
    )
    .unwrap();
    let new = PublicationRootCandidate::admit(new_root, validation).unwrap();
    let preservation = OldReachabilityPreservation::from_protected_footprint(
        delta.plan().protected().footprint_basis(),
    )
    .unwrap();
    let validated = PhysicalPublicationIntent::copy_on_write_root_manifest(old, new, preservation)
        .validate_copy_on_write_inputs()
        .unwrap();
    let readiness = PhysicalPublicationReadiness::from_validated_intent(
        &validated,
        NewRootPublicationProof::from_root_validation(validation),
        PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    );
    let receipt = validated
        .lower_with_ordering(RootSwapOrderingContract::acquire_release_or_stronger())
        .unwrap()
        .join_readiness(readiness)
        .unwrap()
        .complete_plan();
    CompactionRewritePublication::publish_rewrite(delta, receipt).unwrap()
}
