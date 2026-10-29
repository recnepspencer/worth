pub use {std::num::NonZeroU32, std::num::NonZeroU64};

pub use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalWalFrameWriteDisposition, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::PhysicalWalReservationDenial, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

pub use {crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_with_wal_policy, crate::physical_record_journeys::media, crate::physical_record_journeys::success};
pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};

// Exact encoded size of the two-member `second-a`/`second-b` group with
// the current recovery projection; the independent file oracle checks it.
// Each member includes the fixed domain and seven-byte route metadata.
pub const SEGMENT_BYTES: u64 = 35_304;

pub fn wal_policy(inventory: u32) -> PhysicalWalPolicy {
    worth_store::physical_runtime::PhysicalWalPolicy::segmented(
        worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(inventory).unwrap()),
    )
}

pub fn prepared(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    identity: u8,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([identity; 32]))
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("rotation fixture mutation preparation must succeed"),
    }
}

pub fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<PreparedPhysicalMutation>,
) -> SealedPhysicalDurabilityGroupMembers {
    let members = worth_proof::NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("rotation fixture groups are nonempty"));
    match submission.append_prepared_wal_group(members) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation)
        | worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => panic!(
            "the admitted whole group must append: cause={:?}, appended={}, remaining={}, wal={:?}",
            continuation.cause(),
            continuation.appended_member_count(),
            continuation.remaining_member_count(),
            submission.wal_observation().map(|wal| (
                wal.appended_frames(),
                wal.appended_bytes(),
                wal.segment(),
                wal.valid_prefix_bytes(),
            )),
        ),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted { cause, .. } => {
            panic!("group was not admitted: {cause:?}")
        }
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
            panic!("group admission was rejected: {:?}", rejected.cause())
        }
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Indeterminate(_) => {
            panic!("group append became indeterminate")
        }
    }
}

pub fn assert_group_segment(
    group: &SealedPhysicalDurabilityGroupMembers,
    segment: u64,
    dispositions: &[PhysicalWalFrameWriteDisposition],
) {
    assert_eq!(group.members().len(), dispositions.len());
    for (member, disposition) in group.members().iter().zip(dispositions) {
        let declaration = member.mutation().reserved().declaration();
        assert_eq!(declaration.segment().get(), segment);
        assert_eq!(declaration.generation().get(), 1);
        assert_eq!(declaration.disposition(), *disposition);
        assert_eq!(member.mutation().settlement().disposition(), *disposition);
        assert_eq!(
            member.mutation().settlement().artifact(),
            member.mutation().reserved().artifact()
        );
    }
}