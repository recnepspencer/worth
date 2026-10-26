//! Explicit subject  x  public Full-read door  x  failure-class matrix.
use super::cold_read_door::ReadDoor;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use std::collections::BTreeSet;

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
pub(super) const FAILURES: [Cause; 6] = [
    Cause::MissingMember,
    Cause::SourceUnavailable,
    Cause::Damaged,
    Cause::IntegrityMismatch,
    Cause::LimitExceeded {
        length: 2,
        limit: 1,
    },
    Cause::MemoryExhausted,
];

struct MatrixFixture {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    entity: crate::identity::data::EntityId,
    targets: BTreeSet<RelationalDurableMemberLocator>,
    expected_symbol: Option<crate::symbols::data::Symbol>,
}
impl MatrixFixture {
    fn new(subject: ReadSubject, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        if matches!(subject, ReadSubject::HistoricalSymbols) {
            let fixture = super::cold_symbols::view_levels::reopened(execution);
            return Self {
                runtime: fixture.runtime,
                source: fixture.source,
                entity: fixture.entity,
                targets: BTreeSet::from([fixture.historical_page]),
                expected_symbol: Some(fixture.historical),
            };
        }
        let (recovery, source, entity) = fixture_with_history(execution, false, 3);
        source.damaged.store(false, Ordering::Relaxed);
        #[derive(serde::Deserialize)]
        struct Input {
            manifest: crate::durability::data::RelationalCheckpointManifest,
        }
        let manifest = rmp_serde::from_slice::<Input>(
            recovery
                .checkpoint()
                .unwrap()
                .checkpoint()
                .bytes()
                .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
                .unwrap(),
        )
        .unwrap()
        .manifest;
        let targets = manifest
            .mirror
            .iter()
            .chain(manifest.roots.iter().flat_map(|root| root.regions.iter()))
            .filter(|region| region.partition_id == entity.partition_id)
            .map(|region| match subject {
                ReadSubject::RegionHead => region.head,
                ReadSubject::RegionHistory => region.history,
                ReadSubject::HistoricalSymbols => unreachable!("symbol fixture returned above"),
            })
            .collect();
        let mut runtime = durable_runtime_with_test_schema();
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        Self {
            runtime,
            source,
            entity,
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
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in ReadSubject::ALL {
        for door in ReadDoor::ALL {
            for cause in FAILURES {
                let fixture = MatrixFixture::new(subject, execution);
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = attached_test_medium(&fixture.runtime).offered().len();
                fixture.inject(cause);
                let denial = fixture.run(door, execution).unwrap_err();
                let RelationalExecutionDenialCause::StorageUnavailable(failure) = denial else {
                    panic!("{subject:?}  x  {door:?}  x  {cause:?}: {denial:?}");
                };
                assert_eq!(failure.cause, cause, "{subject:?}  x  {door:?}");
                fixture.assert_subject(failure);
                fixture.source.failures.lock().unwrap().clear();
                let reads = fixture.source.reads.load(Ordering::Relaxed);
                if cause.is_proven_damage() {
                    assert_eq!(
                        fixture.run(door, execution),
                        Err(denial),
                        "{subject:?}  x  {door:?}  x  {cause:?}"
                    );
                    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
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
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
            }
        }
    }
}
