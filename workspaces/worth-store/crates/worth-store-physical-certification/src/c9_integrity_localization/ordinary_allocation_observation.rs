//! Start the real mutation director: preparation alone does not perform layout reads.
use super::{
    artifact_edit::ArtifactOperator as Op, artifact_inventory::ArtifactInventory,
    artifact_process::ArtifactRequest,
};
use worth_store::physical_runtime::*;

pub(super) fn require(
    serving: &ServingPhysicalRuntime,
    request: &ArtifactRequest,
    inventory: &ArtifactInventory,
) {
    if let Some(index) = request.poison {
        if inventory.granules[index].family != "free_space_membership_block"
            || !matches!(
                request.operator,
                Op::CoveredByte | Op::Checksum | Op::ScopeSubstitution
            )
        {
            return;
        }
    } else if !matches!(
        request.role,
        super::artifact_process::ArtifactProcessRole::AllocationControl
    ) {
        return;
    }
    let format = AdmittedPhysicalRecordFormat::admit(inventory.format);
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(16).unwrap())
        .segment_pages(SegmentPageCount::new(4).unwrap())
        .admit(format)
        .unwrap();
    let before = serving.resident_admission_counters();
    let snapshot =
        super::process_manifest::ProcessTreeSnapshot::observe_live_diagnostic(&request.root)
            .unwrap();
    let outcome = super::production_store::start_batch(
        serving,
        placement,
        request.profile,
        request.profile.batches() + 1,
    )
    .unwrap()
    .wait();
    if request.poison.is_none() {
        let PhysicalMutationOutcome::Completed(completed) = outcome else {
            panic!("clean reusable allocation must complete through the same public director");
        };
        assert_eq!(
            completed.persisted_records().len(),
            request
                .profile
                .inline_records_per_batch(request.profile.batches() + 1)
                + 1
        );
        let after = serving.resident_admission_counters();
        assert_eq!(
            after.refusals_before_owner_entry(),
            before.refusals_before_owner_entry()
        );
        assert!(after.fresh_validations() > before.fresh_validations());
        assert!(after.owner_decoder_entries() > before.owner_decoder_entries());
        println!(
            "C9 clean allocation completed with {} acknowledged records",
            completed.persisted_records().len()
        );
        return;
    }
    let fate = match outcome {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => fate,
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("allocation became effectful before rejection: {fate:?}")
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("allocation completed despite selected poisoned free-space truth")
        }
    };
    assert_eq!(
        fate.cause(),
        PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
    );
    let after = serving.resident_admission_counters();
    assert_eq!(
        after.refusals_before_owner_entry() - before.refusals_before_owner_entry(),
        1
    );
    assert_eq!(
        after.failed_rechecks_after_owner_entry(),
        before.failed_rechecks_after_owner_entry()
    );
    assert_eq!(
        after.fresh_validations() - before.fresh_validations() + after.exact_record_reuses()
            - before.exact_record_reuses(),
        after.owner_decoder_entries() - before.owner_decoder_entries()
            + after.owner_projection_entries()
            - before.owner_projection_entries()
            + 1
    );
    snapshot.require_unchanged(&request.root).unwrap();
    println!("C9 allocation denied before group seal; counters={after:?}");
}
