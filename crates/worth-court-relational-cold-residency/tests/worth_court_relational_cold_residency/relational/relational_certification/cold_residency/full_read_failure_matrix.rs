//! Complete Full-read subject/operation/refusal matrix on actual published input.
use super::{
    full_read_inventory::FullReadDoor as ReadDoor, immutable_recovery_input::copy_input,
    lineage_event_matrix::FAILURES, selected_directory, symbol_view_source::SymbolViewRecipe,
};
use std::{
    collections::BTreeSet,
    sync::{atomic::Ordering, Arc},
};
use worth_relational::facade::{
    durability::*,
    identity::EntityId,
    runtime::RelationalRuntime,
    storage::RelationalStorageUnavailableCause as Cause,
    storage::{RelationalStorageSubject, RelationalStorageUnavailable},
    symbols::Symbol,
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    cold_recovery::{source_with_history, ColdRecoverySourceFixture},
    durable_medium::{
        immutable_members::{ColdRead, ImmutableCheckpointMemberSource},
        TestDurableMedium,
    },
    durable_publication::durable_runtime,
};
#[derive(Debug, Clone, Copy)]
enum ReadSubject {
    RegionHead,
    RegionHistory,
    HistoricalSymbols,
}
impl ReadSubject {
    const ALL: [Self; 3] = [
        Self::RegionHead,
        Self::RegionHistory,
        Self::HistoricalSymbols,
    ];
}
struct MatrixFixture {
    runtime: RelationalRuntime,
    source: Arc<ImmutableCheckpointMemberSource>,
    medium: Arc<TestDurableMedium>,
    entity: EntityId,
    targets: BTreeSet<RelationalDurableMemberLocator>,
    expected_symbol: Option<Symbol>,
}
impl MatrixFixture {
    fn new(
        subject: ReadSubject,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        region: &ColdRecoverySourceFixture,
        symbols: &SymbolViewRecipe,
    ) -> Self {
        if matches!(subject, ReadSubject::HistoricalSymbols) {
            let f = symbols.reopen(request);
            return Self {
                runtime: f.runtime,
                source: f.source,
                medium: f.medium,
                entity: f.entity,
                targets: BTreeSet::from([f.historical_page]),
                expected_symbol: Some(f.historical),
            };
        }
        let (input, source) = copy_input(region);
        let directory = selected_directory::read(&input);
        let targets = directory
            .mirror
            .iter()
            .chain(directory.roots.iter().flat_map(|r| &r.regions))
            .filter(|r| r.partition_id == region.entity.partition_id)
            .map(|r| match subject {
                ReadSubject::RegionHead => r.head,
                ReadSubject::RegionHistory => r.history,
                ReadSubject::HistoricalSymbols => unreachable!("symbol case returned"),
            })
            .collect();
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                request,
                input,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(request, plan)
            .unwrap();
        assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        Self {
            runtime,
            source,
            medium,
            entity: region.entity,
            targets,
            expected_symbol: None,
        }
    }
    fn inject(&self, cause: Cause) {
        assert!(
            !self.targets.is_empty(),
            "matrix cells need actual producer-selected placements"
        );
        self.source.failures.lock().unwrap().extend(
            self.targets
                .iter()
                .map(|locator| (ColdRead::Member(*locator), cause)),
        );
    }
    fn assert_subject(&self, failure: RelationalStorageUnavailable) {
        match (failure.subject, self.expected_symbol) {
            (RelationalStorageSubject::Region(partition), None) => {
                assert_eq!(partition, self.entity.partition_id)
            }
            (RelationalStorageSubject::Symbols { first, last }, Some(symbol)) => {
                assert!(first <= symbol && symbol <= last)
            }
            _ => panic!("matrix returned a different failed subject: {failure:?}"),
        }
    }
    fn run(
        &self,
        door: ReadDoor,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        door.run(execution, &self.runtime, self.entity)
    }
}

#[test]
fn residency_failure_matrix_full_read_subjects_cover_every_door_and_class() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let region = source_with_history(execution, 3, false);
    let symbols = SymbolViewRecipe::new(execution);
    for subject in ReadSubject::ALL {
        for door in ReadDoor::ALL {
            for (cause, sticky) in FAILURES {
                let fixture = MatrixFixture::new(subject, execution, &region, &symbols);
                door.prepare(&fixture.runtime);
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = fixture.medium.offered().len();
                let target_reads_before = fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|member| fixture.targets.contains(member))
                    .count();
                fixture.inject(cause);
                if door.is_current_projection_scan() && !matches!(subject, ReadSubject::RegionHead)
                {
                    fixture.run(door, execution).unwrap_or_else(|failure| {
                        panic!("current scan touched {subject:?}: {failure:?}")
                    });
                    assert_eq!(
                        fixture
                            .source
                            .member_reads
                            .lock()
                            .unwrap()
                            .iter()
                            .filter(|member| fixture.targets.contains(member))
                            .count(),
                        target_reads_before,
                        "current scans open no additional historical member"
                    );
                    assert_eq!(
                        fixture.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(fixture.medium.offered().len(), offered);
                    continue;
                }
                let denial = fixture.run(door, execution).unwrap_err();
                let RelationalExecutionDenialCause::StorageUnavailable(failure) = denial else {
                    panic!("{subject:?}  x  {door:?}  x  {cause:?}: {denial:?}");
                };
                assert_eq!(failure.cause, cause, "{subject:?}  x  {door:?}");
                fixture.assert_subject(failure);
                fixture.source.failures.lock().unwrap().clear();
                let reads = fixture.source.reads.load(Ordering::Relaxed);
                let failed_reads = || {
                    fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|member| fixture.targets.contains(member))
                        .count()
                };
                let failed_before = failed_reads();
                if sticky {
                    assert_eq!(
                        fixture.run(door, execution),
                        Err(denial),
                        "{subject:?}  x  {door:?}  x  {cause:?}"
                    );
                    if matches!(
                        door,
                        ReadDoor::MemoryCheckpoint | ReadDoor::EmbeddedCheckpoint
                    ) {
                        assert_eq!(failed_reads(), failed_before);
                    } else {
                        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                    }
                } else {
                    fixture.run(door, execution).unwrap_or_else(|failure| {
                        panic!("retry {subject:?}  x  {door:?}  x  {cause:?}: {failure:?}")
                    });
                    assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                }
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(fixture.medium.offered().len(), offered);
            }
        }
    }
}
