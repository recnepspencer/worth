use super::*;
use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

fn empty_cell() -> RelationalBranchReferenceCell {
    RelationalBranchReferenceCell::empty(7, BranchId("main".to_owned()))
        .expect("test branch identity is valid")
}

#[test]
fn a_checkpoint_with_a_one_digit_runtime_ordinal_still_restores() {
    let checkpoint = RelationalBranchReferenceCell::empty(9, BranchId("main".to_owned()))
        .expect("legacy branch identity is valid")
        .checkpoint();
    assert_eq!(checkpoint.runtime_instance_id, 9);
    assert_eq!(
        checkpoint.observation.branch_id().as_str(),
        "relational/9/main"
    );
    let bytes = rmp_serde::to_vec_named(&checkpoint).expect("legacy image encodes");
    let decoded = rmp_serde::from_slice(&bytes).expect("legacy image decodes");
    let restored =
        RelationalBranchReferenceCell::from_checkpoint_with_root(1_000_000_000, decoded, None)
            .expect("the saved ordinal validates its own unchanged branch text");
    assert_eq!(
        restored.observation().branch_id().as_str(),
        "relational/1000000000/main"
    );
    assert_eq!(restored.truth_version(), RelationalBranchVersion::initial());
}

#[test]
fn metadata_movement_advances_generation_without_truth_version() {
    let cell = empty_cell();
    cell.advance_metadata().expect("generation can advance");
    assert_eq!(cell.observation().generation().get(), 1);
    assert_eq!(cell.truth_version(), RelationalBranchVersion::initial());
    assert!(cell.observation().target().is_empty());
}

#[test]
fn truth_movement_advances_generation_and_branch_local_version() {
    let cell = empty_cell();
    cell.advance_truth(FoundationalBranchTarget::empty())
        .expect("truth movement can advance");
    assert_eq!(cell.observation().generation().get(), 1);
    assert_eq!(cell.truth_version().as_u64(), 1);
}

#[test]
fn truth_version_overflow_denies_before_reference_effect() {
    let cell = empty_cell();
    cell.state().truth_version = RelationalBranchVersion::new(u64::MAX);
    let observation_before = cell.observation();
    assert_eq!(
        cell.advance_truth(FoundationalBranchTarget::empty()),
        Err(RelationalBranchCellDenial::TruthVersionOverflow)
    );
    assert_eq!(cell.observation(), observation_before);
}

#[test]
fn generation_overflow_denies_before_truth_effect() {
    let cell = empty_cell();
    let branch_id = cell.observation().branch_id().clone();
    cell.state().observation = RelationalBranchReferenceObservation::new(
        branch_id,
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::new(u64::MAX),
    );
    let version_before = cell.truth_version();
    assert_eq!(
        cell.advance_truth(FoundationalBranchTarget::empty()),
        Err(RelationalBranchCellDenial::GenerationOverflow)
    );
    assert_eq!(cell.truth_version(), version_before);
}

#[test]
fn metadata_generation_overflow_denies_before_reference_effect() {
    let cell = empty_cell();
    let branch_id = cell.observation().branch_id().clone();
    cell.state().observation = RelationalBranchReferenceObservation::new(
        branch_id,
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::new(u64::MAX),
    );
    let observation_before = cell.observation();

    assert_eq!(
        cell.advance_metadata(),
        Err(RelationalBranchCellDenial::GenerationOverflow)
    );
    assert_eq!(cell.observation(), observation_before);
}

#[test]
fn rebinding_preserves_fork_provenance_owner_branch_name() {
    let source = empty_cell();
    let fork = RelationalBranchReferenceCell::from_source(
        7,
        BranchId("storm".to_owned()),
        BranchId("main".to_owned()),
        &source.observation(),
    )
    .expect("fork identity is valid");
    let rebound = fork.rebind_runtime(9).expect("clone identity is valid");

    assert_eq!(
        rebound
            .fork_provenance()
            .expect("fork keeps provenance")
            .branch_id()
            .as_str(),
        "relational/9/main"
    );
}
