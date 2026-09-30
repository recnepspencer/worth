use super::{diff_packet_stream, direct_diff_record_order};
use crate::authority::commit::preparation::packets::diff::{
    DiffPreparationHeader, DiffPreparationPacket,
};
use crate::authority::commit::preparation::reduction::merge::canonical_merge_streams;
use crate::identity::data::{EntityId, PartitionId};
use crate::publication::patch::data::{
    PatchDetail, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use crate::transactions::data::RecordRef;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn direct_serial_patch_preparation_matches_packet_merge_order() {
    let records = vec![
        patch_record(7, RecordStructuralChange::Updated),
        patch_record(3, RecordStructuralChange::Deleted),
        patch_record(3, RecordStructuralChange::Created),
        patch_record(8, RecordStructuralChange::RetainedForAudit),
        patch_record(7, RecordStructuralChange::Created),
        patch_record(3, RecordStructuralChange::Updated),
    ];

    let mut packets = Vec::new();
    for (packet_index, chunk) in records.chunks(2).enumerate() {
        packets.push(DiffPreparationPacket {
            header: DiffPreparationHeader {
                packet_index_floor: packet_index * 2,
            },
            authoritative_record_patches: chunk.to_vec(),
        });
    }

    let inputs = packets
        .into_iter()
        .enumerate()
        .map(|(ordinal, packet)| crate::execution::ReadOnlyPacket {
            identity: worth_foundational::PartitionIdentity::new(ordinal as u64),
            input_bytes: 0,
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
            value: packet,
        })
        .collect();
    let streams =
        crate::execution::execute_read_only_packets(inputs, None, diff_packet_stream, |_| 0)
            .expect("serial packet preparation must complete");
    let merged = canonical_merge_streams(streams)
        .into_iter()
        .map(|(_key, record)| record)
        .collect::<Vec<_>>();
    let direct = direct_diff_record_order(records);

    assert_eq!(direct, merged);
}

fn patch_record(
    raw_entity_id: u64,
    structural_change: RecordStructuralChange,
) -> PublishedAuthoritativeRecordPatch {
    let structural_change_label = format!("{structural_change:?}");
    PublishedAuthoritativeRecordPatch {
        target: RecordRef::Entity(EntityId::new(PartitionId(0), raw_entity_id, 0)),
        structural_change,
        authoritative_patch: crate::publication::patch::data::PublishedAuthoritativePatch::empty(),
        semantic_changes: Vec::new(),
        contains_opaque_aspect: false,
        detail: PatchDetail::DenseBitset(vec![raw_entity_id, structural_change_label.len() as u64]),
    }
}

fn fragment(
    raw_entity_id: u64,
    structural_change: RecordStructuralChange,
) -> crate::authority::mutation::FoundationalPatchFragment {
    let record = patch_record(raw_entity_id, structural_change);
    crate::authority::mutation::FoundationalPatchFragment {
        target: record.target,
        structural_change: record.structural_change,
        patch: worth_foundational::facade::AuthoritativeRecordAspectPatch::empty(),
        published_patch: record.authoritative_patch,
        semantic_changes: record.semantic_changes,
        contains_opaque_aspect: record.contains_opaque_aspect,
        detail: record.detail,
    }
}

fn publication_lease(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 8 * 1024 * 1024, work),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn leased_publication_matches_serial_without_post_merge_clone() {
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder().build();
    let preparation = runtime.preparation_runtime_snapshot();
    let fragments = vec![
        fragment(7, RecordStructuralChange::Updated),
        fragment(3, RecordStructuralChange::Deleted),
        fragment(3, RecordStructuralChange::Created),
        fragment(8, RecordStructuralChange::RetainedForAudit),
        fragment(7, RecordStructuralChange::Created),
    ];
    let serial = super::assemble_patch(&preparation, fragments.clone(), None).unwrap();
    let lease = publication_lease(10_000);
    let leased = super::assemble_patch(&preparation, fragments, Some(&lease)).unwrap();
    assert_eq!(leased, serial);
}

#[test]
fn leased_publication_stops_inside_large_packet_without_publishing() {
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder().build();
    let preparation = runtime.preparation_runtime_snapshot();
    let fragments = (0..100)
        .map(|index| fragment(index, RecordStructuralChange::Updated))
        .collect();
    let lease = publication_lease(120);
    let result = super::assemble_patch(&preparation, fragments, Some(&lease));
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::WorkExhausted,
                    ..
                },
                ..
            }
        )
    ));
}
