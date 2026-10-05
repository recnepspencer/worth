use super::*;

#[path = "tests/source.rs"]
mod source;
use source::{
    admitted_source, admitted_source_over_two_leaves, admitted_source_under,
    admitted_source_with_cutoff, record_format,
};

#[test]
fn v2_roster_and_rooted_walk_share_one_admission_budget_before_head_read() {
    let source = admitted_source();
    let format = record_format();
    let generous = 256 * 1024;
    let mut positive_reads = 0;
    let admitted = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        8,
        generous,
        |reference, _| {
            positive_reads += 1;
            assert_eq!(reference, source.head.reference(format));
            Ok::<_, ()>(source.head.encode(format))
        },
    )
    .expect("the exact V2 checkpoint and rooted source head admit");
    assert_eq!(positive_reads, 1);
    assert_eq!(admitted.selected_heads(), &[source.entry]);
    assert_eq!(
        admitted.accumulator_v2().head_roster_digest(),
        source.digest
    );
    assert_eq!(admitted.accumulator_v2().head_count(), 1);
    let owned = admitted.owned_heap_bytes().unwrap();
    let walker = admitted.head_walk_peak_resident_bytes();
    assert!(owned > 0 && walker > 0);
    let insufficient = owned + walker - 1;
    assert!(insufficient > owned && insufficient > walker);
    let mut denied_reads = 0;
    let denied = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        8,
        insufficient,
        |_, _| {
            denied_reads += 1;
            Ok::<_, ()>(source.head.encode(format))
        },
    );
    assert!(matches!(
        denied,
        Err(SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::ResidentBoundExceeded { required, admitted }))
            if required > admitted && admitted == insufficient
    ));
    assert_eq!(denied_reads, 0);
    assert!(admitted.admission_peak_resident_bytes() <= generous);
}

#[test]
fn different_valid_checkpoint_stream_denies_before_rooted_head_read() {
    let source = admitted_source();
    let other = admitted_source_with_cutoff(19);
    assert_eq!(source.checkpoint.source(), other.checkpoint.source());
    assert_ne!(source.checkpoint.facts(), other.checkpoint.facts());
    let format = record_format();
    let mut reads = 0;
    let denied = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &other.checkpoint,
        &source.root,
        format,
        8,
        256 * 1024,
        |_, _| {
            reads += 1;
            Ok::<_, ()>(source.head.encode(format))
        },
    );
    assert!(matches!(
        denied,
        Err(SelectedHeadRosterAdmissionDenial::Custody(
            SelectedCustodyDenial::CertificateRoster
        ))
    ));
    assert_eq!(reads, 0);
}

#[test]
fn swapped_checkpoint_certificate_fails_the_roster_certificate_law() {
    let source = admitted_source();
    let format = record_format();
    let admitted = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        8,
        256 * 1024,
        |_, _| Ok::<_, ()>(source.head.encode(format)),
    )
    .expect("the exact V2 checkpoint and rooted source head admit");
    let records = source.checkpoint.certificate_records();
    assert!(admitted.certificates_match(records.iter().map(AsRef::as_ref)));

    // Same records, same count; only the Batch and accumulator trade places.
    let released = records
        .iter()
        .enumerate()
        .filter(|(_, frame)| {
            worth_store_physical_format::decode_checkpoint_certificate(frame).is_ok_and(
                |(kind, _)| {
                    kind == worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop
                },
            )
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [batch, accumulator] = released.as_slice() else {
        panic!("fixture checkpoint carries one Batch and its V2 accumulator")
    };
    let mut swapped = records.to_vec();
    swapped.swap(*batch, *accumulator);
    assert!(!admitted.certificates_match(swapped.iter().map(AsRef::as_ref)));

    let mut missing = records.to_vec();
    missing.remove(*accumulator);
    assert!(!admitted.certificates_match(missing.iter().map(AsRef::as_ref)));
}

#[test]
fn a_roster_of_more_heads_than_the_caller_admits_names_that_bound_and_its_count() {
    let source = admitted_source();
    let format = record_format();
    let mut reads = 0;
    let mut admit = |maximum_head_entries| {
        VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
            &source.selected,
            &source.checkpoint,
            &source.root,
            format,
            maximum_head_entries,
            256 * 1024,
            |_, _| {
                reads += 1;
                Ok::<_, ()>(source.head.encode(format))
            },
        )
    };
    // The roster verified and counts one head: only the caller's bound fails.
    assert!(matches!(
        admit(0),
        Err(SelectedHeadRosterAdmissionDenial::HeadEntries {
            observed: 1,
            admitted: 0,
        })
    ));
    assert_eq!(admit(1).unwrap().accumulator_v2().head_count(), 1);
    assert_eq!(reads, 1, "the refused roster read no head block");
}

#[test]
fn a_root_over_more_blocks_than_its_roster_and_level_allow_is_the_walkers_node_bound() {
    // One head at level one allows the root and one block below it; a root
    // over two leaves announces a third block before any leaf is read.
    let source = admitted_source_over_two_leaves();
    let format = record_format();
    let mut reads = 0;
    let denied = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        8,
        256 * 1024,
        |reference, _| {
            reads += 1;
            Ok::<_, ()>(source.blocks[reference.block() as usize - 1].encode(format))
        },
    );
    assert!(
        matches!(
            denied,
            Err(SelectedHeadRosterAdmissionDenial::Walk(
                ReleaseCustodyHeadWalkDenial::NodeBound {
                    observed: 3,
                    admitted: 2,
                }
            ))
        ),
        "{denied:?}"
    );
    assert_eq!(reads, 1, "only the root was read");
}

#[test]
fn a_sound_tree_of_more_blocks_and_levels_than_it_has_heads_admits_whole() {
    // Seventeen single-child branches over the one head's leaf: eighteen
    // blocks on eighteen levels, admitted for the one head they hold.
    let source = admitted_source_under(20, 17);
    let format = record_format();
    let mut reads = 0;
    let mut admit = |damaged: Option<u64>| {
        VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
            &source.selected,
            &source.checkpoint,
            &source.root,
            format,
            1,
            256 * 1024,
            |reference, _| {
                reads += 1;
                let mut frame = source.blocks[reference.block() as usize - 1].encode(format);
                if damaged == Some(reference.block()) {
                    let last = frame.len() - 1;
                    frame[last] ^= 1;
                }
                Ok::<_, ()>(frame)
            },
        )
    };
    let admitted = admit(None).expect("no bound on blocks comes from the head count");
    assert_eq!(admitted.selected_heads(), &[source.entry]);
    // A damaged block of the same tree is damage, under the same admission.
    assert!(matches!(
        admit(Some(9)),
        Err(SelectedHeadRosterAdmissionDenial::Walk(
            ReleaseCustodyHeadWalkDenial::Format(_)
        ))
    ));
    assert_eq!(
        reads,
        18 + 10,
        "the whole tree, then down to the damaged block"
    );
}
