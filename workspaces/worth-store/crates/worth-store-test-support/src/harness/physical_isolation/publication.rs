use super::epoch_scope::{
    current_generation_page_reference, current_root_from_authority,
    physical_authority_from_complete_closeout, physical_authority_from_complete_closeout_for_store,
};
use super::read_plan::{admit_plan, protected_set};
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalReferenceAuthority,
    PhysicalRootReference, RootPublicationValidationWitness,
};
use worth_store_physical_isolation::{
    CopyOnWritePublicationPlan, NewRootPublicationProof, OldReachabilityPreservation,
    PhysicalPublicationIntent, PhysicalPublicationReadiness, PhysicalReadPlanReleaseReceipt,
    PublicationLatchReadiness, PublicationRootCandidate, PublicationRootSuccessorOwner,
    RootSwapOrderingContract,
};

pub struct PublicationInputs {
    pub old_authority: worth_store_physical_isolation::PhysicalReadStabilityAuthority,
    pub old_root: worth_store_physical_isolation::CurrentPhysicalRoot,
    pub new_root: worth_store_physical_isolation::CurrentPhysicalRoot,
    pub old_candidate: PublicationRootCandidate,
    pub new_candidate: PublicationRootCandidate,
    pub old_reachability: OldReachabilityPreservation,
    pub old_release: PhysicalReadPlanReleaseReceipt,
    pub new_validation: RootPublicationValidationWitness,
}

pub fn publication_inputs() -> PublicationInputs {
    publication_inputs_with_root_generation(701)
}

pub fn publication_inputs_with_root_generation(reference_generation: u64) -> PublicationInputs {
    let old_authority = physical_authority_from_complete_closeout();
    let old_root = current_root_from_authority(&old_authority);
    let old_validation = root_publication_validation(old_root.scope(), 1);
    let old_candidate = PublicationRootCandidate::admit(old_root, old_validation).unwrap();
    let new_candidate = PublicationRootSuccessorOwner::plan(
        old_candidate,
        physical_generation(reference_generation),
    )
    .unwrap();
    publication_inputs_from_candidates(
        old_authority,
        old_candidate,
        new_candidate,
        reference_generation,
    )
}

pub fn publication_inputs_for_store(
    store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
    reference_generation: u64,
) -> PublicationInputs {
    let old_authority = physical_authority_from_complete_closeout_for_store(store_identity);
    let old_root = current_root_from_authority(&old_authority);
    let old_candidate =
        PublicationRootCandidate::admit(old_root, root_publication_validation(old_root.scope(), 1))
            .unwrap();
    let new_candidate = PublicationRootSuccessorOwner::plan(
        old_candidate,
        physical_generation(reference_generation),
    )
    .unwrap();
    publication_inputs_from_candidates(
        old_authority,
        old_candidate,
        new_candidate,
        reference_generation,
    )
}

fn publication_inputs_from_candidates(
    old_authority: worth_store_physical_isolation::PhysicalReadStabilityAuthority,
    old_candidate: PublicationRootCandidate,
    new_candidate: PublicationRootCandidate,
    reference_generation: u64,
) -> PublicationInputs {
    let old_root = old_candidate.root();
    let new_root = new_candidate.root();
    let new_validation = new_candidate.validation();
    let reference = current_generation_page_reference(reference_generation);
    let old_plan = admit_plan(
        &old_authority,
        old_root,
        protected_set([reference], 4),
        8,
        4,
    );
    let old_reachability = OldReachabilityPreservation::from_protected_footprint(
        old_plan.footprint().declared_footprint_basis(),
    )
    .unwrap();
    let old_release = old_plan.into_execution_ready_handle().release();
    PublicationInputs {
        old_authority,
        old_root,
        new_root,
        old_candidate,
        new_candidate,
        old_reachability,
        old_release,
        new_validation,
    }
}

pub fn admitted_copy_on_write_plan(inputs: &PublicationInputs) -> CopyOnWritePublicationPlan {
    let intent = PhysicalPublicationIntent::copy_on_write_root_manifest(
        inputs.old_candidate,
        inputs.new_candidate,
        inputs.old_reachability,
    );
    let validated = intent.validate_copy_on_write_inputs().unwrap();
    let lowered = validated
        .clone()
        .lower_with_ordering(RootSwapOrderingContract::acquire_release_or_stronger())
        .unwrap();
    let readiness = PhysicalPublicationReadiness::from_validated_intent(
        &validated,
        NewRootPublicationProof::from_root_validation(inputs.new_validation),
        PublicationLatchReadiness::declared_publish_latches_released_before_blocking_io(),
    );
    lowered.join_readiness(readiness).unwrap()
}

pub fn mismatched_release_receipt(reference_generation: u64) -> PhysicalReadPlanReleaseReceipt {
    let old_authority = physical_authority_from_complete_closeout();
    let old_root = current_root_from_authority(&old_authority);
    let reference = current_generation_page_reference(reference_generation);
    admit_plan(
        &old_authority,
        old_root,
        protected_set([reference], 4),
        8,
        4,
    )
    .into_execution_ready_handle()
    .release()
}

pub fn root_publication_validation(root: u64, generation: u64) -> RootPublicationValidationWitness {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let cell = generations
        .root_publication_cell(PhysicalRootReference::from_raw(root).unwrap())
        .with_root_publication_generation(PhysicalGeneration::from_raw(generation).unwrap());
    references
        .validate_root_publication(references.admit_root_publication(cell), cell)
        .unwrap()
}

fn physical_generation(generation: u64) -> PhysicalGeneration {
    PhysicalGeneration::from_raw(generation).expect("fixture root generation must be nonzero")
}
