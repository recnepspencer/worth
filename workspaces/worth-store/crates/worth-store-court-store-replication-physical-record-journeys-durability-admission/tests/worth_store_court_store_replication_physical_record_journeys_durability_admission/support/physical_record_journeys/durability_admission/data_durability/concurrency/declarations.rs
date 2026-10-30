pub(crate) use std::sync::mpsc;
pub(crate) use std::time::Duration;

pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::PhysicalDataDispatchOutcome,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordWriteSource,
    worth_store::physical_runtime::RecordWriteSourceError,
};

pub(crate) use worth_store_testkit_store::physical_record_journeys::read_record;
pub(crate) use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::durable_publication::wal_data::prepare_single,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

pub(crate) static SEED: [u8; 20_000] = [0x51; 20_000];
pub(crate) const LEFT: &[u8] = b"left concurrent payload";
pub(crate) const RIGHT: &[u8] = b"right concurrent payload";

pub(crate) fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> PreparedPhysicalMutation {
    into_prepared(prepare_single(
        submission,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        material,
        batch,
    ))
}

pub(crate) fn into_prepared(
    outcome: PhysicalMutationPreparationOutcome,
) -> PreparedPhysicalMutation {
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("canonical preparation must succeed"),
    }
}

pub(crate) fn dispatched(
    outcome: PhysicalDataDispatchOutcome,
) -> worth_store::physical_runtime::DataDispatchedPhysicalMutation {
    match outcome {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => {
            dispatched
        }
        _ => panic!("the exact WAL-durable member must dispatch"),
    }
}

pub(crate) fn assert_disjoint_member_targets(
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

pub(crate) fn dispatch_label(outcome: &PhysicalDataDispatchOutcome) -> String {
    match outcome {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(_) => {
            "dispatched".into()
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(retry) => {
            format!("suspended:{:?}", retry.cause())
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted {
            cause, ..
        } => {
            format!("not-started:{cause:?}")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(failure) => {
            format!("indeterminate:{:?}", failure.cause())
        }
    }
}

pub(crate) struct PausedInlineSource {
    pub(crate) bytes: &'static [u8],
    pub(crate) entered: std::sync::mpsc::SyncSender<()>,
    pub(crate) release: std::sync::mpsc::Receiver<()>,
    pub(crate) completed: bool,
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
