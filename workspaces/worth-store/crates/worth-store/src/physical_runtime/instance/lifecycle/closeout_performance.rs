use crate::physical_runtime::{
    record_serving::{RecordPublicationResidueObservation, RecordServingCounterSnapshot},
    CheckpointPerformanceExpectation, CloseoutPerformanceExpectation,
    GroupCommitPerformanceExpectation, IdempotencyPerformanceExpectation,
    PageBasisPerformanceExpectation, PhysicalCheckpointShutdown, PhysicalDurabilityCloseoutOutcome,
    PhysicalDurabilityPerformanceContract, PhysicalDurabilityPerformanceSummary,
    PhysicalIoPerformanceExpectation, PhysicalMutationCostSnapshot, PhysicalMutationShutdown,
    PhysicalQueuePerformanceExpectation, PhysicalTrafficPerformanceExpectation,
    PhysicalWalObservation, PhysicalWorkShutdownObservation,
};

pub(super) struct PhysicalCloseoutPerformanceObservation<'a> {
    pub(super) witness: worth_store_aspect_native::StorePhysicalBoundaryWitness,
    pub(super) mutation: PhysicalMutationShutdown,
    pub(super) mutation_cost: PhysicalMutationCostSnapshot,
    pub(super) checkpoint: PhysicalCheckpointShutdown,
    pub(super) wal: PhysicalWalObservation,
    pub(super) records: RecordServingCounterSnapshot,
    pub(super) residency: worth_store_buffer_pool::PhysicalResidencyShutdown,
    pub(super) work: &'a PhysicalWorkShutdownObservation,
    pub(super) residue: RecordPublicationResidueObservation,
    pub(super) closeout: &'a PhysicalDurabilityCloseoutOutcome,
}

pub(super) fn closeout_performance_summary(
    observation: PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceSummary {
    PhysicalDurabilityPerformanceSummary::from_observed_contracts(
        observation.witness,
        [
            group_commit_contract(&observation),
            checkpoint_contract(&observation),
            page_basis_contract(&observation),
            idempotency_contract(&observation),
            closeout_contract(&observation),
        ],
    )
}

fn group_commit_contract(
    observation: &PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceContract {
    let [groups, data_writes, data_bytes, _, acknowledgments, peak_group_members] =
        observation.mutation_cost.values();
    let group_member_limit = observation
        .closeout
        .recovery_handoff()
        .map_or(0, |handoff| {
            u64::from(handoff.durability_policy().group_commit_limit().get().get())
        });
    PhysicalDurabilityPerformanceContract::GroupCommit(GroupCommitPerformanceExpectation::new(
        PhysicalTrafficPerformanceExpectation::new(
            observation.mutation.started(),
            groups,
            acknowledgments,
        ),
        PhysicalIoPerformanceExpectation::new(
            observation.wal.appended_frames(),
            observation.wal.appended_bytes(),
        ),
        PhysicalIoPerformanceExpectation::new(data_writes, data_bytes),
        groups,
        PhysicalQueuePerformanceExpectation::new(peak_group_members, group_member_limit),
    ))
}

fn checkpoint_contract(
    observation: &PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceContract {
    let checkpoint = observation.checkpoint;
    let terminal = checkpoint
        .completed()
        .saturating_add(checkpoint.proven_no_effect())
        .saturating_add(checkpoint.indeterminate());
    let retained_wal_segments = observation
        .closeout
        .recovery_handoff()
        .map_or(0, |handoff| handoff.wal_tail().segments().len() as u64);
    PhysicalDurabilityPerformanceContract::Checkpoint(CheckpointPerformanceExpectation::new(
        PhysicalTrafficPerformanceExpectation::new(
            checkpoint.started(),
            checkpoint.completed(),
            terminal,
        ),
        PhysicalIoPerformanceExpectation::new(checkpoint.completed(), checkpoint.encoded_bytes()),
        checkpoint.dirty_records(),
        retained_wal_segments,
    ))
}

fn page_basis_contract(
    observation: &PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceContract {
    let [_, data_writes, data_bytes, persisted_records, _, _] = observation.mutation_cost.values();
    PhysicalDurabilityPerformanceContract::PageBasis(PageBasisPerformanceExpectation::new(
        data_writes,
        data_bytes,
        persisted_records,
    ))
}

fn idempotency_contract(
    observation: &PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceContract {
    let expectation = observation.closeout.recovery_handoff().map_or(
        IdempotencyPerformanceExpectation::from_values([0; 6]),
        |handoff| {
            let counts = handoff.operation_fates().counts();
            let live = counts
                .unresolved()
                .saturating_add(counts.completed())
                .saturating_add(counts.proven_no_effect())
                .saturating_add(counts.indeterminate());
            IdempotencyPerformanceExpectation::from_counts(live, counts)
        },
    );
    PhysicalDurabilityPerformanceContract::Idempotency(expectation)
}

fn closeout_contract(
    observation: &PhysicalCloseoutPerformanceObservation<'_>,
) -> PhysicalDurabilityPerformanceContract {
    let residency = observation.residency.counters();
    let live_residency_bytes = residency
        .metadata_bytes()
        .saturating_add(residency.resident_bytes())
        .saturating_add(residency.dirty_replacement_bytes())
        .saturating_add(residency.active_operation_bytes());
    let mutation_terminal = observation
        .mutation
        .completed()
        .saturating_add(observation.mutation.proven_no_effect())
        .saturating_add(observation.mutation.indeterminate());
    let checkpoint_terminal = observation
        .checkpoint
        .completed()
        .saturating_add(observation.checkpoint.proven_no_effect())
        .saturating_add(observation.checkpoint.indeterminate());
    PhysicalDurabilityPerformanceContract::Closeout(CloseoutPerformanceExpectation::from_values([
        mutation_terminal,
        checkpoint_terminal,
        observation
            .work
            .residual()
            .saturating_add(observation.work.unaccounted_terminal()),
        observation.records.live_handles(),
        live_residency_bytes,
        publication_residue_class_count(observation.residue),
    ]))
}

const fn publication_residue_class_count(residue: RecordPublicationResidueObservation) -> u64 {
    residue.staging_catalog_candidate() as u64
        + residue.successor_root() as u64
        + residue.successor_routing_block() as u64
        + residue.successor_membership_blocks() as u64
        + residue.successor_free_space() as u64
        + residue.next_segment_artifacts() as u64
        + residue.reusable_segment_artifacts() as u64
        + residue.next_arena_artifact() as u64
}
