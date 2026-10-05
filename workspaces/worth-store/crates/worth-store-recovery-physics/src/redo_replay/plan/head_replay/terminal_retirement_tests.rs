use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedPhysicalRecoveryProjection, PhysicalPageSizeClass,
    PhysicalRecoveryProjectionDecodeLimits, ReleaseCustodyHeadBlockV1,
};
use worth_store_wal::LogSequenceNumber;

use super::*;
use crate::redo_replay::terminal_head_retirement_fixture::{
    data_projection, format, member, nonterminal_head, projection, selected_terminal_head,
    source_leaf, store, survivor, terminal_head, SelectedTerminalHead, SOURCE_GENERATION, TREE,
};
use crate::{
    admit_physical_redo_members, AdmittedPhysicalRedoMembers, ExceededHeadReplayBound,
    HeadReplayBound, PhysicalRedoAdmissionLimits,
};

const EFFECT_BYTES: u64 = 64 * 1024;
const HEAP_BYTES: u64 = 1 << 20;
const DENIED_AS_UPSERT: SelectedReleaseHeadReplayDenial =
    SelectedReleaseHeadReplayDenial::NotAdmittedUpsert;
const DENIED_AS_RETIREMENT: SelectedReleaseHeadReplayDenial =
    SelectedReleaseHeadReplayDenial::NotAdmittedTerminalHeadRetirement;

fn two_heads() -> SelectedTerminalHead {
    selected_terminal_head(vec![terminal_head(), survivor()])
}

/// The real C.9 admission of one record-less retirement member at LSN 12.
fn admitted(claimed: &PersistedPhysicalRecoveryProjection) -> AdmittedPhysicalRedoMembers {
    admit_physical_redo_members(
        vec![member(claimed, 12, 1, RecoveryOperationFate::Indeterminate)],
        store(),
        format(),
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: u64::MAX,
            targets: 0,
            distinct_targets: 0,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: 0,
                record_identities: 0,
                placements: 0,
                segment_updates: 0,
                manifests: 0,
                total_entries: 2,
                inline_allocations: 0,
            },
        },
    )
    .expect("C.9 admits a record-less terminal head retirement member")
}

fn replay(
    selected: &SelectedTerminalHead,
    source: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    effect_bytes: u64,
    heap_bytes: u64,
    selected_frame: Result<&[u8], ()>,
) -> (
    Result<VerifiedSelectedTerminalHeadRetirementReplay, SelectedReleaseHeadReplayDenial>,
    u32,
) {
    let claimed = projection(selected.retirement.clone());
    let members = admitted(&claimed);
    let view = members.admitted_root_step_members().next().unwrap();
    let mut reads = 0;
    let admitted = VerifiedSelectedTerminalHeadRetirementReplay::admit_member(
        view,
        source,
        format,
        effect_bytes,
        heap_bytes,
        |reference, bytes| {
            assert_eq!(reference, selected.retirement.source_root());
            assert_eq!(bytes, u64::from(format.page_size().bytes()));
            reads += 1;
            selected_frame.map(<[u8]>::to_vec)
        },
    );
    (admitted, reads)
}

#[test]
fn the_exact_terminal_head_of_the_selected_tree_is_retired_and_nothing_else_changes() {
    let selected = two_heads();
    let frame = selected.source_frame.as_slice();
    let (admitted, reads) = replay(
        &selected,
        &selected.source,
        format(),
        EFFECT_BYTES,
        HEAP_BYTES,
        Ok(frame),
    );
    let admitted = admitted.unwrap();
    assert_eq!(reads, 1);
    assert_eq!(admitted.retirement(), &selected.retirement);
    assert_eq!(admitted.retirement().expected_prior(), terminal_head());
    assert_eq!(admitted.source_frame_bytes(), frame.len() as u64);
    assert_eq!(admitted.lsn_range().start(), LogSequenceNumber::new(12));
    assert_eq!(
        admitted.lsn_range().end_exclusive(),
        LogSequenceNumber::new(13)
    );
    assert_eq!(admitted.operation(), [1; 32]);
    assert_eq!(admitted.fate(), RecoveryOperationFate::Indeterminate);
    let claimed = projection(selected.retirement.clone());
    let redo = member(&claimed, 12, 1, RecoveryOperationFate::Indeterminate);
    assert_eq!(
        admitted.canonical_redo_sha256(),
        <[u8; 32]>::from(Sha256::digest(redo.canonical_redo()))
    );

    // The WAL-chosen result is the selected leaf minus exactly the retired
    // entry: one rewritten leaf that still holds only the survivor.
    let [write] = admitted.retirement().node_writes() else {
        panic!("one leaf is rewritten");
    };
    assert_eq!(admitted.retirement().result_root(), Some(write.reference()));
    let (leaf, _) =
        ReleaseCustodyHeadBlockV1::decode(write.frame(), write.reference(), TREE).unwrap();
    assert_eq!(leaf.entries(), Some([survivor()].as_slice()));
}

#[test]
fn retiring_the_only_head_of_the_selected_tree_leaves_no_head_tree() {
    let selected = selected_terminal_head(vec![terminal_head()]);
    let frame = selected.source_frame.as_slice();
    let (admitted, reads) = replay(
        &selected,
        &selected.source,
        format(),
        EFFECT_BYTES,
        HEAP_BYTES,
        Ok(frame),
    );
    let admitted = admitted.unwrap();
    assert_eq!(reads, 1);
    assert_eq!(admitted.retirement().expected_prior(), terminal_head());
    assert_eq!(admitted.retirement().result_root(), None);
    assert!(admitted.retirement().node_writes().is_empty());
}

#[test]
fn a_selected_root_other_than_the_claimed_source_denies_before_any_read() {
    let selected = two_heads();
    let root = selected.retirement.source_root();
    let other_tree = source_leaf(vec![nonterminal_head(), survivor()], format()).reference();
    let manifest = |generation, tree, head_root, next_block| {
        DurablePhysicalRootManifest::builder(generation, tree, 4, 1)
            .release_custody_head_root(head_root)
            .next_release_custody_head_block(next_block)
            .admit()
            .unwrap()
    };
    assert_eq!(
        manifest(SOURCE_GENERATION, TREE, Some(root), 2),
        selected.source
    );
    for source in [
        manifest(SOURCE_GENERATION + 1, TREE, Some(root), 2),
        manifest(SOURCE_GENERATION, TREE + 1, Some(root), 2),
        manifest(SOURCE_GENERATION, TREE, Some(other_tree), 2),
        manifest(SOURCE_GENERATION, TREE, None, 2),
        manifest(SOURCE_GENERATION, TREE, Some(root), 3),
    ] {
        let frame = selected.source_frame.as_slice();
        assert_eq!(
            replay(
                &selected,
                &source,
                format(),
                EFFECT_BYTES,
                HEAP_BYTES,
                Ok(frame)
            ),
            (Err(SelectedReleaseHeadReplayDenial::SourceRoot), 0)
        );
    }
}

#[test]
fn a_selected_frame_that_differs_from_the_carried_path_is_not_the_claimed_tree() {
    let selected = two_heads();
    // Selected media holds the same key while its release is still in flight.
    let in_flight = source_leaf(vec![nonterminal_head(), survivor()], format());
    for (selected_frame, denial) in [
        (
            Ok(in_flight.frame()),
            SelectedReleaseHeadReplayDenial::SourcePath,
        ),
        (Err(()), SelectedReleaseHeadReplayDenial::Read),
    ] {
        assert_eq!(
            replay(
                &selected,
                &selected.source,
                format(),
                EFFECT_BYTES,
                HEAP_BYTES,
                selected_frame
            ),
            (Err(denial), 1)
        );
    }
}

#[test]
fn bounds_and_the_exact_recomputation_deny_before_any_read() {
    let selected = two_heads();
    let frame = selected.source_frame.as_slice();
    let framed = selected.retirement.framed_bytes().unwrap();
    let heap = selected
        .retirement
        .verification_additional_peak_bytes(format())
        .unwrap()
        .max(selected.retirement.owned_heap_bytes().unwrap())
        .max(u64::from(format().page_size().bytes()));
    let wide = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let exact = replay(
        &selected,
        &selected.source,
        format(),
        framed,
        heap,
        Ok(frame),
    );
    assert!(exact.0.is_ok());
    for (format, effect_bytes, heap_bytes, denial) in [
        (
            format(),
            framed - 1,
            heap,
            SelectedReleaseHeadReplayDenial::BoundExceeded(ExceededHeadReplayBound {
                bound: HeadReplayBound::EffectBytes,
                observed: framed,
                admitted: framed - 1,
            }),
        ),
        (
            format(),
            framed,
            heap - 1,
            SelectedReleaseHeadReplayDenial::BoundExceeded(ExceededHeadReplayBound {
                bound: HeadReplayBound::HeapBytes,
                observed: heap,
                admitted: heap - 1,
            }),
        ),
        (wide, EFFECT_BYTES, HEAP_BYTES, DENIED_AS_RETIREMENT),
    ] {
        assert_eq!(
            replay(
                &selected,
                &selected.source,
                format,
                effect_bytes,
                heap_bytes,
                Ok(frame)
            ),
            (Err(denial), 0)
        );
    }
}

#[test]
fn each_head_replay_admits_only_its_own_member_kind() {
    let selected = two_heads();
    let claimed = projection(selected.retirement.clone());
    let members = admitted(&claimed);
    let retirement_member = members.admitted_root_step_members().next().unwrap();
    let data = data_projection();
    let data_member = AdmittedRootStepMemberView {
        materialization: &data,
        ..retirement_member
    };
    let mut reads = 0;
    let mut read = |_, _| {
        reads += 1;
        Ok::<Vec<u8>, ()>(selected.source_frame.clone())
    };
    assert_eq!(
        VerifiedSelectedTerminalHeadRetirementReplay::admit_member(
            data_member,
            &selected.source,
            format(),
            EFFECT_BYTES,
            HEAP_BYTES,
            &mut read,
        ),
        Err(DENIED_AS_RETIREMENT)
    );
    assert_eq!(
        super::super::VerifiedSelectedReleaseHeadReplayV14::admit_effect(
            super::super::upsert_effect(claimed.operation()),
            &selected.source,
            format(),
            EFFECT_BYTES,
            HEAP_BYTES,
            &mut read,
            None,
            retirement_member.operation(),
            retirement_member.group(),
            retirement_member.fate(),
            retirement_member.canonical_redo_sha256(),
        ),
        Err(DENIED_AS_UPSERT)
    );
    assert_eq!(reads, 0);
}
