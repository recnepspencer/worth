//! Real cold checkpoints preserve both original component bases of a publication.
//! The C2 portions of
//! 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs
//! (`original_custody`) are restored in place: the selected and unrelated
//! envelope cells stay cold, the selected envelope demand carries the readmitted
//! semantic result, and both original roots remain fully demandable. Fixture
//! adaptations: residency comes from the owner's `observe_commit_envelope_custody`;
//! the envelope demand is the public replay read of the same cell; full root
//! demand and its snapshot are `RelationalBasisCustodyWitness::read_retained_full`.
//! The unrelated-artifact deletion belongs to C4 and is not restored here.
mod retirement;
use super::*;
use worth_relational::facade::{
    durability::RelationalRecoveredPublicationCustody, history::CommitId,
    runtime::RelationalRuntime,
};

fn resident(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    commit: CommitId,
) -> bool {
    runtime
        .observe_commit_envelope_custody()
        .into_iter()
        .find(|cell| cell.commit_id == commit)
        .expect("retained commit cell")
        .resident
}

fn readmit_original(
    request: &ExecutionRequest<'_, '_>,
    empty: bool,
) -> (
    RelationalRuntime,
    worth_proof::RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
) {
    let original = original(request, empty);
    let offers = original.medium.offered().len();
    let (runtime, authority) = recover(request, &original.medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(request, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    assert!(!resident(&runtime, subject.receipt().commit_id));
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(request, &subject)
        .unwrap();
    let frame = Artifact::readmit(
        &original.medium.artifact_bytes(subject.position()).unwrap(),
        request,
    )
    .unwrap();
    let semantic = frame.envelope().semantic_result();
    assert_eq!(effect.semantic_result(), semantic);
    let selected = runtime
        .replay()
        .canonical_commit_envelope(request, subject.receipt().commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(effect.semantic_result(), selected.semantic_result());
    let proof = authority
        .admit_recorded_publication(request, &runtime, effect, &original.before, &original.after)
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), semantic);
    let shared: &RelationalRuntime = &runtime;
    shared.run_branch_root_reclamation_pass();
    let before_count = if empty { 0 } else { 1 };
    for (basis, recorded, count) in [
        (
            proof.custody().predecessor(),
            &original.before,
            before_count,
        ),
        (
            proof.custody().successor(),
            &original.after,
            before_count + 1,
        ),
    ] {
        let exported = runtime
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(request, basis)
            .unwrap();
        assert_eq!(&exported, recorded);
        let witness = basis.custody_witness();
        assert_eq!(
            witness.read_retained_full(request, &runtime).unwrap(),
            Some(witness.schema_authority_digest())
        );
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let read = runtime
            .read_truth()
            .read_snapshot(&snapshot, request)
            .expect("original retained read remains funded")
            .expect("original retained snapshot remains readable");
        assert_eq!(
            read.entities().len(),
            count,
            "the actual original basis remains readable"
        );
        assert!(read.relations().is_empty());
        drop(read);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    }
    assert!(runtime
        .owner_component_services()
        .basis_port()
        .compare_current_exact(proof.custody().successor())
        .is_err());
    assert_eq!(
        original.medium.offered().len(),
        offers,
        "recovery never republishes"
    );
    assert!(!resident(&runtime, original.unrelated_id));
    (runtime, proof)
}

fn verify_readmission_and_close(empty: bool) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let (runtime, proof) = readmit_original(&ExecutionRequest::serial(&host), empty);
    drop(proof);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor() {
    verify_readmission_and_close(false);
}

#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    verify_readmission_and_close(true);
}
