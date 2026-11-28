use super::{diff_packet_stream, direct_diff_record_order};
use crate::authority::commit::preparation::packets::diff::{
    DiffPreparationHeader, DiffPreparationPacket,
};
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatchOwnerPort as _;
use contracts::publication::patch::data::patch_detail::PatchDetail;
use contracts::publication::patch::data::record_structural_change::RecordStructuralChange;
use contracts::transactions::data::primitives::RecordRef;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::authority::commit::preparation::reduction::merge::canonical_merge_streams;

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

    let lease = publication_lease(10_000);
    let inputs = packets
        .into_iter()
        .enumerate()
        .map(
            |(ordinal, packet)| contracts::execution::read_only_packets::ReadOnlyPacket {
                identity: worth_foundational::PartitionIdentity::new(ordinal as u64),
                input_bytes: (packet.authoritative_record_patches.capacity()
                    * std::mem::size_of::<PublishedAuthoritativeRecordPatch>())
                    as u64
                    + packet
                        .authoritative_record_patches
                        .iter()
                        .map(|record| record.owned_allocation_capacity_bytes())
                        .sum::<u64>(),
                memory: contracts::execution::packet_memory::PacketMemory::Declared {
                    scratch: 0,
                    result: 8 * 1024,
                },
                value: packet,
            },
        )
        .collect();
    let streams = contracts::execution::read_only_packets::execute_read_only_packets(
        inputs,
        &lease,
        diff_packet_stream,
        |_| 0,
    )
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
        relation_birth: None,
        authoritative_patch: contracts::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatch::empty(),
        semantic_changes: Vec::new(),
        contains_opaque_aspect: false,
        detail: PatchDetail::DenseBitset(vec![raw_entity_id, structural_change_label.len() as u64]),
    }
}

fn fragment(
    raw_entity_id: u64,
    structural_change: RecordStructuralChange,
) -> worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::patch_fragments::FoundationalPatchFragment{
    let record = patch_record(raw_entity_id, structural_change);
    worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::patch_fragments::FoundationalPatchFragment {
        published: record,
        patch: worth_foundational::facade::AuthoritativeRecordAspectPatch::empty(),
    }
}

fn publication_lease(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    worth_harness::facade::execution_fixture::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(work)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn leased_publication_matches_serial_without_post_merge_clone() {
    let instrumentation = worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let fragments = vec![
        fragment(7, RecordStructuralChange::Updated),
        fragment(3, RecordStructuralChange::Deleted),
        fragment(3, RecordStructuralChange::Created),
        fragment(8, RecordStructuralChange::RetainedForAudit),
        fragment(7, RecordStructuralChange::Created),
    ];
    let serial = super::assemble_patch(&instrumentation, None, fragments.clone(), None).unwrap();
    let lease = publication_lease(10_000);
    let leased = super::assemble_patch(&instrumentation, None, fragments, Some(&lease)).unwrap();
    assert_eq!(leased, serial);
}

#[test]
fn leased_publication_stops_inside_large_packet_without_publishing() {
    let instrumentation = worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let fragments = (0..100)
        .map(|index| fragment(index, RecordStructuralChange::Updated))
        .collect();
    let lease = publication_lease(120);
    let result = super::assemble_patch(&instrumentation, None, fragments, Some(&lease));
    assert!(matches!(
        result,
        Err(
            contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                    kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkExhausted
                    ),
                    ..
                },
                ..
            }
        )
    ));
}
