//! Orphan discovery inspects actual required metadata, not known-open guesses.
use super::associated_effects::perform;
use crate::facade::durability::RelationalAssociatedEffectRefusal;
use crate::facade::mvcc::RelationalCallerAssociation as Association;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let source = durable_runtime_for_medium(&medium);
    let known = Association::from_bytes(b"world:known-original-open").unwrap();
    let orphan = Association::from_bytes(b"world:orphan-effect-without-open").unwrap();
    let other = Association::from_bytes(b"another-owner:unrelated").unwrap();
    for (key, association) in [
        ("known", &known),
        ("orphan", &orphan),
        ("unrelated-associated", &other),
    ] {
        let pending = perform(execution, &source, key, association);
        assert!(
            source
                .durability_authority()
                .discover_all_associated_effects(execution)
                .unwrap()
                .iter()
                .any(|subject| subject.association() == Some(association)),
            "actual pending route is discoverable before catalog settlement"
        );
        let result = source.settle_performed_publication(pending).unwrap();
        release_test_commit_snapshot(&source, &result);
    }
    create_entity(execution, &source, "unassociated-current-head");
    capture_and_file_checkpoint(execution, &source, &medium);
    let mut runtime = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(execution, &mut runtime, &medium);
    for position in medium.positions().into_iter().take(3) {
        medium.drop_artifact_and_member(position);
    }
    medium.start_recording_member_reads();
    let subjects = runtime
        .durability_authority()
        .discover_all_associated_effects(execution)
        .unwrap();
    assert!(
        medium.non_index_member_read_digests().is_empty(),
        "enumerating associated effects must perform zero canonical body reads"
    );
    assert_eq!(subjects.len(), 3);
    for (subject, expected) in subjects.iter().zip([&known, &orphan, &other]) {
        assert_eq!(subject.association(), Some(expected));
        assert!(runtime
            .history
            .commit_artifact(subject.receipt().commit_id)
            .unwrap()
            .envelope_residency()
            .positioned_if_resident()
            .is_none());
    }
    assert_eq!(
        runtime
            .durability_authority()
            .discover_associated_effects(execution, &known)
            .unwrap()
            .len(),
        1
    );
    assert!(
        medium.non_index_member_read_digests().is_empty(),
        "filtered associated-effect discovery must perform zero canonical body reads"
    );
    let memory = runtime_with_test_schema();
    let result = memory
        .settle_performed_publication(perform(execution, &memory, "memory-associated", &known))
        .unwrap();
    assert!(matches!(
        memory
            .durability_authority()
            .discover_all_associated_effects(execution),
        Err(RelationalAssociatedEffectRefusal::AssociatedMemory { .. })
    ));
    release_test_commit_snapshot(&memory, &result);
}

fn same_subject(
    keyed: &crate::facade::durability::RelationalAssociatedEffectSubject,
    scanned: &crate::facade::durability::RelationalAssociatedEffectSubject,
) {
    assert_eq!(keyed.receipt(), scanned.receipt());
    assert_eq!(keyed.position(), scanned.position());
    assert_eq!(keyed.artifact_digest(), scanned.artifact_digest());
    assert_eq!(keyed.association(), scanned.association());
}

/// #388: the keyed form finds exactly the enumerated subject in either custody
/// cell, nothing for an unassociated or unknown commit, and reads no cold body.
#[test]
fn keyed_discovery_returns_the_enumerated_subject_of_either_custody_cell_without_cold_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let source = durable_runtime_for_medium(&medium);
    let association = Association::from_bytes(b"world:keyed-original-open").unwrap();
    let pending = perform(execution, &source, "keyed-pending", &association);
    let scanned = source
        .durability_authority()
        .discover_all_associated_effects(execution)
        .unwrap();
    let [performed_only] = scanned.as_slice() else {
        panic!("one performed route awaits catalog settlement")
    };
    assert!(source
        .history
        .commit_artifact(performed_only.receipt().commit_id)
        .is_none());
    let keyed = source
        .durability_authority()
        .discover_associated_effect(execution, performed_only.receipt())
        .unwrap()
        .expect("the performed-only cell is discoverable by key");
    same_subject(&keyed, performed_only);
    let committed = source.settle_performed_publication(pending).unwrap();
    release_test_commit_snapshot(&source, &committed);
    create_entity(execution, &source, "keyed-unassociated");
    capture_and_file_checkpoint(execution, &source, &medium);
    let mut runtime = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(execution, &mut runtime, &medium);
    for position in medium.positions() {
        medium.drop_artifact_and_member(position);
    }
    medium.start_recording_member_reads();
    let authority = runtime.durability_authority();
    let scanned = authority
        .discover_all_associated_effects(execution)
        .unwrap();
    let [settled] = scanned.as_slice() else {
        panic!("one associated catalog-settled commit")
    };
    let keyed = authority
        .discover_associated_effect(execution, settled.receipt())
        .unwrap()
        .expect("the catalog-settled cell is discoverable by key");
    same_subject(&keyed, settled);
    let unassociated = runtime.history.latest_commit_artifact().unwrap().receipt();
    assert_ne!(&unassociated, settled.receipt());
    assert!(authority
        .discover_associated_effect(execution, &unassociated)
        .unwrap()
        .is_none());
    let mut unknown = settled.receipt().clone();
    unknown.commit_id = crate::history::data::CommitId(u64::MAX);
    assert!(authority
        .discover_associated_effect(execution, &unknown)
        .unwrap()
        .is_none());
    let mut renamed = settled.receipt().clone();
    renamed.branch_id = crate::facade::history::BranchId("another-branch".to_string());
    assert!(
        authority
            .discover_associated_effect(execution, &renamed)
            .unwrap()
            .is_none(),
        "the subject must carry exactly the asked receipt, not only its commit id"
    );
    assert!(
        medium.non_index_member_read_digests().is_empty(),
        "keyed associated-effect discovery must perform zero canonical body reads"
    );
}
