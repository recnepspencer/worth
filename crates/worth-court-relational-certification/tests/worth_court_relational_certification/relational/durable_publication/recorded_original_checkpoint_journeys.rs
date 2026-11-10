//! Real cold checkpoints preserve both original component bases of a publication.
use super::*;

fn readmit_original(empty: bool) {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let original = original(request, empty);
    let offers = original.medium.offered().len();
    let (runtime, authority) = recover(request, &original.medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(request, &original.association)
        .unwrap()
        .pop()
        .unwrap();
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
    let proof = authority
        .admit_recorded_publication(request, &runtime, effect, &original.before, &original.after)
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), semantic);
    let shared: &worth_relational::facade::runtime::RelationalRuntime = &runtime;
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
    drop(proof);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn recorded_component_checkpoint_readmits_canonical_predecessor_and_successor() {
    readmit_original(false);
}

#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    readmit_original(true);
}
