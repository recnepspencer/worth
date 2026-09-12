use std::sync::mpsc;
use std::time::Duration;

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint;
use worth_store::physical_runtime::{
    PhysicalDataDispatchOutcome, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalWalGroupAppendOutcome,
    PhysicalWalGroupBarrierOutcome, PreparedPhysicalMutation, RecordAppendBatch, RecordByteLimit,
    RecordReadLimits, RecordWriteSource, RecordWriteSourceError,
};

use super::super::super::durable_publication::{prepare_single, publish_single};
use super::super::super::{configuration, serving_from_initialization};
use crate::read_record;

static SEED: [u8; 20_000] = [0x51; 20_000];
const LEFT: &[u8] = b"left concurrent payload";
const RIGHT: &[u8] = b"right concurrent payload";

#[test]
fn disjoint_data_effects_overlap_before_one_exact_group_root_publication() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let seed = publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([150; 32]),
        RecordAppendBatch::try_from_iter([SEED.as_slice()]).unwrap(),
    )
    .settled_members()[0]
        .record_id(0)
        .unwrap();
    let submission = serving.certification_record_submission();
    let left = prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([151; 32]),
        RecordAppendBatch::try_from_iter([LEFT]).unwrap(),
    );
    let right = prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([152; 32]),
        RecordAppendBatch::try_from_iter([RIGHT]).unwrap(),
    );
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(left, vec![right])) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the exact two-member WAL group must append"),
    };
    assert_disjoint_member_targets(&appended);
    let basis = appended.basis();
    let mut durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => durable.into_members().into_vec(),
        _ => panic!("the exact two-member WAL group must become durable"),
    };
    let right = durable.pop().expect("the group has a right member");
    let left = durable.pop().expect("the group has a left member");
    assert!(durable.is_empty());

    let dispatch_gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let left_submission = serving.certification_record_submission();
    let left_thread = std::thread::spawn(move || left_submission.dispatch_wal_durable_data(left));
    let right_submission = serving.certification_record_submission();
    let right_thread =
        std::thread::spawn(move || right_submission.dispatch_wal_durable_data(right));
    if !dispatch_gate.await_arrivals(2) {
        dispatch_gate.release();
        let left = left_thread.join().unwrap();
        let right = right_thread.join().unwrap();
        panic!(
            "both disjoint data effects must reach backend dispatch before either is released; \
             arrivals={}, left={}, right={}",
            dispatch_gate.arrival_count(),
            dispatch_label(&left),
            dispatch_label(&right),
        );
    }
    let seed_session = serving
        .records()
        .expect("read protection admission")
        .open(
            seed,
            RecordReadLimits::new(RecordByteLimit::new(SEED.len() as u32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(seed_session, SEED.len()).0, SEED.as_slice());

    dispatch_gate.release();
    let left = dispatched(left_thread.join().unwrap());
    let right = dispatched(right_thread.join().unwrap());
    let completed =
        serving.certification_complete_dispatched_group(basis, NonEmpty::new(left, vec![right]));
    assert_eq!(completed.current_root().generation(), 3);
    assert_eq!(completed.settled_members().len(), 2);
    for (member, expected) in completed.settled_members().iter().zip([LEFT, RIGHT]) {
        let session = serving
            .records()
            .expect("read protection admission")
            .open(
                member.record_id(0).unwrap(),
                RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
            )
            .unwrap();
        assert_eq!(read_record(session, expected.len()).0, expected);
    }
    serving.close();
}

#[test]
fn paused_source_materialization_does_not_own_global_preparation_authority() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::super::serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let paused_submission = serving.certification_record_submission();
    let paused_thread = std::thread::spawn(move || {
        prepare_single(
            &paused_submission,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([153; 32]),
            RecordAppendBatch::builder()
                .push_source(PausedInlineSource {
                    bytes: LEFT,
                    entered: entered_tx,
                    release: release_rx,
                    completed: false,
                })
                .build()
                .unwrap(),
        )
    });
    entered_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the paused source must enter canonical materialization");
    let independent = prepared(
        &serving.certification_record_submission(),
        placement,
        PhysicalMutationIdempotencyMaterial::new([154; 32]),
        RecordAppendBatch::try_from_iter([RIGHT]).unwrap(),
    );
    release_tx.send(()).unwrap();
    let paused = into_prepared(paused_thread.join().unwrap());
    assert_ne!(
        paused.mutation_identity().operation_identity(),
        independent.mutation_identity().operation_identity()
    );
    drop((paused, independent));
    serving.close();
}

fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> PreparedPhysicalMutation {
    into_prepared(prepare_single(
        submission,
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        material,
        batch,
    ))
}

fn into_prepared(outcome: PhysicalMutationPreparationOutcome) -> PreparedPhysicalMutation {
    match outcome.into_raw() {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("canonical preparation must succeed"),
    }
}

fn dispatched(
    outcome: PhysicalDataDispatchOutcome,
) -> worth_store::physical_runtime::DataDispatchedPhysicalMutation {
    match outcome {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("the exact WAL-durable member must dispatch"),
    }
}

fn assert_disjoint_member_targets(
    group: &worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers,
) {
    let targets = group
        .members()
        .iter()
        .map(|member| {
            member
                .mutation()
                .reserved()
                .redo()
                .records()
                .iter()
                .flat_map(|record| record.targets())
                .map(|claim| claim.target())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2);
    for left in &targets[0] {
        for right in &targets[1] {
            assert_ne!(left, right, "group members reused data target {left:?}");
        }
    }
}

fn dispatch_label(outcome: &PhysicalDataDispatchOutcome) -> String {
    match outcome {
        PhysicalDataDispatchOutcome::Dispatched(_) => "dispatched".into(),
        PhysicalDataDispatchOutcome::RetryableAfterCleanup(retry) => {
            format!("retryable-after-cleanup:{:?}", retry.pressure())
        }
        PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            format!("not-started:{cause:?}")
        }
        PhysicalDataDispatchOutcome::Indeterminate(failure) => {
            format!("indeterminate:{:?}", failure.cause())
        }
    }
}

struct PausedInlineSource {
    bytes: &'static [u8],
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
    completed: bool,
}

impl RecordWriteSource for PausedInlineSource {
    fn declared_length(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        if self.completed {
            return Ok(0);
        }
        self.entered.send(()).unwrap();
        self.release.recv().unwrap();
        target[..self.bytes.len()].copy_from_slice(self.bytes);
        self.completed = true;
        Ok(self.bytes.len())
    }
}
