//! Real checkpoint buckets guard text admission and applied candidate rounds.
use super::*;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use crate::transactions::data::{CommitPreparationReason, EntityReference, RelationSpec};
use sha2::{Digest, Sha256};

const POPULATION: usize = 600;

struct SymbolFixture {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    label_bucket: RelationalDurableMemberLocator,
    cold_key: String,
    endpoints: [crate::identity::data::EntityId; 2],
    relation_floor: usize,
}

fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> SymbolFixture {
    let producer = durable_runtime_with_test_schema();
    let bucket = Sha256::digest(b"label")[0];
    let cold_key = (0..65_536)
        .map(|id| format!("cold-label-bucket-{id}"))
        .find(|text| Sha256::digest(text.as_bytes())[0] == bucket)
        .unwrap();
    let mut keys = (0..POPULATION)
        .map(|id| crate::symbols::data::ClientKey::raw(format!("symbol-seed-{id:04}")))
        .collect::<Vec<_>>();
    keys.push(crate::symbols::data::ClientKey::raw(cold_key.clone()));
    let mut transaction = test_owner_begin_transaction_for_main(&producer);
    transaction
        .push_batch(
            WorkerIntentBatch::new("cold-symbol-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    field_patches: keys.iter().map(|_| name_field_patch("seed")).collect(),
                    client_keys: keys,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &producer).unwrap();
    let entities = changed_entities(&committed);
    let endpoints = [entities[0], entities[1]];
    release_test_commit_snapshot(&producer, &committed);
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
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
    let label_bucket = manifest
        .symbol_buckets
        .iter()
        .find(|entry| entry.bucket == bucket)
        .unwrap()
        .pages[0];
    let relation_floor = manifest
        .mirror
        .iter()
        .find(|leaf| leaf.partition_id == PartitionId::main())
        .unwrap()
        .relation_slots;
    assert!(
        manifest.symbols.len() > 1,
        "fixture spans dictionary id pages"
    );
    let attachment = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(attachment);
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
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no symbol or region members"
    );
    SymbolFixture {
        runtime,
        source,
        label_bucket,
        cold_key,
        endpoints,
        relation_floor,
    }
}

fn relation_transaction(
    fixture: &SymbolFixture,
    key: &str,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("cold-revision-symbol").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw(key),
                    source: EntityReference::Existing(fixture.endpoints[0]),
                    target: EntityReference::Existing(fixture.endpoints[1]),
                    fields: relation_label_field_patch("edge"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
}

fn storage_failure(error: TransactionCommitError) -> RelationalStorageUnavailable {
    match error {
        TransactionCommitError::Preparation { error, .. } => match error.reason() {
            CommitPreparationReason::StorageUnavailable(failure) => failure,
            other => panic!("expected storage preparation refusal, got {other:?}"),
        },
        other => panic!("expected pre-effect preparation failure, got {other:?}"),
    }
}

#[derive(Clone, Copy, Debug)]
enum Door {
    FieldText,
    Preview,
    Validation,
    Candidate,
    CommitRevision,
}

impl Door {
    fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fixture: &SymbolFixture,
    ) -> Result<(), RelationalStorageUnavailable> {
        if matches!(self, Self::FieldText) {
            let transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
            let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
                b"label".to_vec().into_boxed_slice(),
            ));
            return fixture.runtime.admit_field_text(execution, &transaction.basis, &input).map(|_| ()).map_err(|error| match error {
                crate::symbols::data::RelationalFieldTextAdmissionDenial::StorageUnavailable(failure) => failure,
                other => panic!("field text must retain its storage failure: {other:?}"),
            });
        }
        let key = if matches!(self, Self::CommitRevision) {
            "member-before"
        } else {
            &fixture.cold_key
        };
        let mut transaction = relation_transaction(fixture, key);
        match self {
            Self::Preview => transaction
                .prepare_merged_plan(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::Validation => transaction
                .validate(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::Candidate => fixture
                .runtime
                .prepare_branch_transaction(execution, transaction)
                .map(|_| ())
                .map_err(storage_failure),
            Self::CommitRevision => transaction
                .commit(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::FieldText => unreachable!(),
        }
    }
}

#[test]
fn cold_symbol_buckets_refuse_each_preparation_door_without_journal_effects_and_stay_damaged() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in [
        Door::FieldText,
        Door::Preview,
        Door::Validation,
        Door::Candidate,
        Door::CommitRevision,
    ] {
        let fixture = reopened(execution);
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&fixture.runtime).offered().len();
        let symbols = fixture.runtime.services.symbols.snapshot();
        *fixture.source.failed_member.lock().unwrap() = Some(fixture.label_bucket);
        let failure = door
            .run(execution, &fixture)
            .expect_err("cold dictionary damage is not absence or empty success");
        assert!(
            matches!(failure.subject, RelationalStorageSubject::Symbols { .. }),
            "{door:?}: {failure:?}"
        );
        assert_eq!(failure.cause, Cause::Damaged);
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        *fixture.source.failed_member.lock().unwrap() = None;
        assert_eq!(door.run(execution, &fixture), Err(failure));
        assert_eq!(
            fixture.source.reads.load(Ordering::Relaxed),
            reads,
            "{door:?} damage is sticky"
        );
        assert_eq!(
            fixture.runtime.services.symbols.snapshot(),
            symbols,
            "failed bucket or draft publishes no meaning"
        );
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

#[test]
fn a_cold_symbol_probe_spends_the_abandoned_record_identity_and_reads_only_touched_buckets() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let committed = relation_transaction(&fixture, "member-before")
        .commit(execution, &fixture.runtime)
        .unwrap();
    let relation = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    assert!(
        relation.slot_index() > fixture.relation_floor,
        "a discarded symbol-probe draft cannot reuse its identity"
    );
    assert!(
        relation.slot_index() < fixture.relation_floor + 16,
        "retry growth uses headroom"
    );
    assert!(
        fixture.source.reads.load(Ordering::Relaxed) < 16,
        "one root and touched buckets fit bounded headroom"
    );
    assert!(
        fixture.runtime.services.symbols.snapshot().entries.len() < POPULATION / 2,
        "operation must not load the dictionary population"
    );
    release_test_commit_snapshot(&fixture.runtime, &committed);
}
