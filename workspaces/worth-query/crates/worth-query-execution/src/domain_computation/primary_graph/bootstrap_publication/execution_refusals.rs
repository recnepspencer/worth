//! Bootstrap's seams consume real execution refusals; production has no lease injector.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as QueryKind,
};
use std::{num::NonZeroUsize, sync::OnceLock, time::Instant};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseRequest,
};
use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    indexes::{DerivedIndexDefinition, DerivedIndexExecutionDenialKind, DerivedIndexKind},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::{
        AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
        KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    transactions::{BulkEntityCreateIntent, RelationalExecutionDenialCause as Cause},
};
pub(in crate::domain_computation::primary_graph) fn isolated(test_name: &str, run: impl FnOnce()) {
    let test_name = test_name.split_once("::").unwrap().1;
    const PROBE: &str = "WORTH_BOOTSTRAP_INDEX_EXECUTION_PROBE";
    if std::env::var(PROBE).as_deref() == Ok(test_name) {
        run();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .env(PROBE, test_name)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

pub(in crate::domain_computation::primary_graph) fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: Some(64 * 1024 * 1024),
        })
        .unwrap()
    })
}

pub(in crate::domain_computation::primary_graph) fn request() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 1_000_000),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}

fn refused_index_build(request: LeaseRequest) -> DerivedIndexBuildOutcome {
    let key = AspectKey::new("name").unwrap();
    let field = FieldKey::new("name").unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        key.clone(),
        CanonicalFieldPath::single(field.clone()),
    );
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "result.entity".to_owned(),
            schema_id: SchemaId("result-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField { field },
                    contract: aspects()
                        .contract()
                        .for_key(key)
                        .identified_by(AspectIdentity(1))
                        .at_revision(aspects().vocabulary().revision(1))
                        .scalar(ScalarAspectType::String),
                },
            ]),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("index-source").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..128)
                        .map(|i| ClientKey::raw(format!("index-heavy-{i}")))
                        .collect(),
                    field_patches: (0..128)
                        .map(|i| {
                            AspectFieldPatch::from_locator(
                                locator.clone(),
                                AspectValue::String(format!("index-heavy-{i}").into()),
                            )
                        })
                        .collect(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "large.index.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator,
        },
        branch_scoped: true,
    });
    let build_request = DerivedIndexBuildRequest {
        source_commit_id: committed.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    let baseline = runtime
        .index_authority()
        .build_for_commit(build_request.clone());
    assert!(baseline.execution_denial.is_none());

    assert_eq!(baseline.generations.len(), 1);
    assert!(identity_indexes_built(baseline, 1).is_ok());
    let lease = authority().request_lease(request).unwrap();
    runtime
        .index_authority()
        .build_for_commit_with_lease(build_request, &lease)
}
#[test]
fn result_refusal_is_typed_with_partition_identity() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        isolated(
            concat!(
                module_path!(),
                "::result_refusal_is_typed_with_partition_identity"
            ),
            || {
                let build = refused_index_build(request());
                let refusal = build.execution_denial.expect("real execution refusal");
                assert!(refusal.partition_identity.is_some());
                assert_eq!(
                    refusal.kind,
                    DerivedIndexExecutionDenialKind::Cause(Cause::ResultCapacityExceeded)
                );
                let denied = identity_indexes_built(build, 1).unwrap_err();
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                        kind: QueryKind::ExecutionResource {
                            denial: Resource::ResultCapacityExceeded,
                            partition_identity: refusal.partition_identity,
                            policy_ancestor: None,
                        },
                    }
                );
            },
        );
    });
}

#[test]
fn cancellation_and_deadline_are_control_stops() {
    isolated(
        concat!(
            module_path!(),
            "::cancellation_and_deadline_are_control_stops"
        ),
        || {
            let cancellation = CancellationSource::new();
            cancellation.cancel();
            let mut cancelled = request();
            cancelled.cancellation = cancellation.token();
            let mut timed_out = request();
            timed_out.deadline = Some(Instant::now());
            for (request, expected) in [
                (cancelled, Control::Cancelled),
                (timed_out, Control::TimedOut),
            ] {
                let build = refused_index_build(request);
                assert!(build.execution_denial.is_some());
                let denied = identity_indexes_built(build, 1).unwrap_err();
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                        kind: expected
                    }
                );
            }
        },
    );
}
