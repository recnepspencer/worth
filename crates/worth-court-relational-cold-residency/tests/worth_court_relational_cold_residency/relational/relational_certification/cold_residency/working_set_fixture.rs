//! A real checkpointed Home reopened under a declarative `ResidencyBudget`
//! (Store M4.2 S1/S3): every cold-backed region joins the runtime's working
//! set, and public reads must stay within it.
use worth_execution::ExecutionRequest;
use worth_relational::facade::config::{RelationalRuntimeProfile, ResidencyBudget};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableRecoverySource, ResidencyPressureSnapshot,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::{
    begin_main_transaction, changed_entities, release_test_commit_snapshot,
};
use worth_testkit_relational::cold_recovery::{capture_source, ColdRecoverySourceFixture};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::{field_patch, schema};

/// Distinct regions in the reopened root; each holds one sizeable entity.
pub(super) const REGIONS: u32 = 30;

pub(super) struct ColdHome {
    pub(super) input: ColdRecoverySourceFixture,
    pub(super) entities: Vec<EntityId>,
}

/// One producer Home with `REGIONS` cold regions behind a native checkpoint.
pub(super) fn cold_home(request: &ExecutionRequest<'_, '_>) -> ColdHome {
    let medium = TestDurableMedium::new();
    let producer = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .build_durable(medium.attachment())
        .unwrap();
    let payload = "x".repeat(16 * 1024);
    let mut entities = Vec::new();
    for region in 1..=REGIONS {
        let key = format!("region-{region}");
        let mut transaction = begin_main_transaction(&producer);
        transaction
            .push_batch(
                WorkerIntentBatch::new(key.clone()).push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::new(region),
                        kind_id: KindId(1),
                        client_key: key.as_str().into(),
                        fields: field_patch("name", "name", &format!("{key}-{payload}")),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(request, &producer).unwrap();
        entities.extend(changed_entities(&committed));
        release_test_commit_snapshot(&producer, &committed);
    }
    let entity = entities[0];
    ColdHome {
        input: capture_source(request, producer, medium, entity, false),
        entities,
    }
}

/// Reopen the checkpoint into a fresh durable runtime bounded by `budget`.
pub(super) fn reopen(
    request: &ExecutionRequest<'_, '_>,
    home: &ColdHome,
    budget: ResidencyBudget,
) -> RelationalRuntime {
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .residency_budget(budget)
        .build_durable(home.input.medium.attachment())
        .unwrap();
    let input = RelationalDurableRecoverySource::new(
        home.input.input.checkpoint().cloned(),
        Vec::new(),
        Vec::new(),
        home.input.source.clone(),
    );
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
    runtime
}

pub(super) fn budget(head: u64, historical: u64) -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: head,
        historical_working_set_bytes: historical,
        max_in_flight_fill_bytes: u64::MAX / 4,
        streaming_window_bytes: 64 * 1024,
    }
}

pub(super) fn pressure(runtime: &RelationalRuntime) -> ResidencyPressureSnapshot {
    runtime
        .residency_pressure()
        .expect("a budgeted runtime observes its working set")
}

/// Resident bytes one region's Head costs, measured by one unbounded scan.
pub(super) fn head_region_bytes(request: &ExecutionRequest<'_, '_>, home: &ColdHome) -> u64 {
    let runtime = reopen(request, home, budget(u64::MAX / 4, u64::MAX / 4));
    assert_eq!(scan(request, &runtime), REGIONS as usize);
    let resident = pressure(&runtime).head_resident_bytes;
    assert!(resident > 0, "the scan admitted cold regions");
    resident.div_ceil(u64::from(REGIONS))
}

/// One public kind scan of the main branch's held observation.
pub(super) fn scan(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> usize {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    scan_observation(request, runtime, &basis.observation()).unwrap()
}

pub(super) fn scan_observation(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    observation: &worth_relational::facade::mvcc::RelationalBranchObservation,
) -> Result<usize, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    let view = runtime
        .read_truth()
        .project_observation(observation)
        .unwrap();
    view.bounded_entities_of_kind(request, KindId(1), 1_000_000)
        .map(|answer| {
            answer
                .expect("the scan fits its declared bound")
                .records()
                .len()
        })
}

/// Cold member reads the producer's immutable source has served so far.
pub(super) fn member_reads(home: &ColdHome) -> usize {
    home.input
        .source
        .reads
        .load(std::sync::atomic::Ordering::Relaxed)
}

pub(super) fn memory_exhausted(
    cause: &worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> bool {
    format!("{cause:?}").contains("MemoryExhausted")
}
