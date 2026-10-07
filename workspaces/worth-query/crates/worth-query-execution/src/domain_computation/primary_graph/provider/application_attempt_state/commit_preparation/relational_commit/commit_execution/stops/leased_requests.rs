//! Public leased preparation probes, isolated by process because execution
//! admits exactly one physical authority for a process lifetime. Query's other
//! unit-test owners have their own authority; this scope cannot replace it.
use crate::domain_computation::primary_graph::application_attempt::{
    provider_compare_denial::preparation_outcome::application_outcome,
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationCommitOutcome as Outcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use std::{num::NonZeroUsize, sync::OnceLock, time::Instant};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::{KindId, PartitionId},
    indexes::{
        DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexExecutionDenialKind,
        DerivedIndexId, DerivedIndexKind, RelatedEntityEndpoint,
    },
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, TransactionCommitError,
        WorkerIntentBatch,
    },
};

pub(super) fn in_isolated_process(test_name: &str, run: impl FnOnce()) {
    let test_name = test_name.split_once("::").unwrap().1;
    const PROBE: &str = "WORTH_RELATIONAL_PROVIDER_STOP_PROBE";
    if std::env::var(PROBE).as_deref() == Ok(test_name) {
        run();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .env(PROBE, test_name)
        .output()
        .expect("isolated authority probe starts");
    assert!(
        output.status.success(),
        "isolated probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "probe ran its exact test"
    );
}

pub(super) fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: 64 * 1024 * 1024,
        })
        .expect("one authority in this probe process")
    })
}

pub(super) fn request(memory: u64, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, memory, work),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}

pub(super) fn schema() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "probe.entity".to_owned(),
            schema_id: SchemaId("provider-stop-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "probe.relation".to_owned(),
                schema_id: SchemaId("provider-stop-probe".to_owned()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("declared probe entity and relation kinds")
}

pub(super) fn transaction(
    runtime: &RelationalRuntime,
    key: &str,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::default(),
                },
            ))),
        )
        .unwrap();
    transaction
}

fn check_requests(request: LeaseRequest, check: impl Fn(Outcome)) {
    let lease = authority().request_lease(request).unwrap();
    check_lease(&lease, check);
}

fn related_index(runtime: &RelationalRuntime) -> DerivedIndexDefinition {
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "probe.related".to_owned(),
        branch_scoped: true,
        kind: DerivedIndexKind::RelatedEntityOrdering {
            relation_kind: KindId(2),
            parent_endpoint: RelatedEntityEndpoint::SourceParent,
            child_kind: KindId(1),
            ordering: Vec::new(),
        },
    })
}

fn check_lease(lease: &worth_execution::ExecutionResourceLease<'_>, check: impl Fn(Outcome)) {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build();
    let created = transaction(&runtime, "source").commit(&runtime).unwrap();
    let index = related_index(&runtime);
    let index = runtime.index_authority().build_for_commit_with_lease(
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        },
        lease,
    );
    assert!(index.generations.is_empty());
    let denial = index.execution_denial.expect("index preparation refused");
    let DerivedIndexExecutionDenialKind::Cause(cause) = denial.kind;
    // Candidate maintenance currently returns maintenance denials, not this
    // build refusal. This direct build probe stands in for bootstrap's index
    // build door: it exercises real leased owner work and the shared cause
    // conversion. Bootstrap has its own installation seam test; 7.6 will
    // provide production request carriage. This is not a maintenance route.
    check(application_outcome(super::relational_execution_stop(
        cause,
        denial.partition_identity,
    )));

    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let error = runtime
        .prepare_branch_transaction_with_lease(transaction(&runtime, "target"), lease)
        .expect_err("commit preparation refused");
    assert!(matches!(error, TransactionCommitError::Execution { .. }));
    let outcome = application_outcome(super::transaction_commit_stop(error));
    let Outcome::Denied(denial) = &outcome else {
        panic!("preparation cause folded");
    };
    crate::domain_computation::primary_graph::conditional_operation::assert_preparation_retry(
        denial,
    );
    check(outcome);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}

fn resource(stop: Outcome) -> Resource {
    let Outcome::Denied(failure) = stop else {
        panic!("wrong provider stop: {stop:?}")
    };
    let Kind::ExecutionResource { denial, .. } = failure.kind() else {
        panic!("lost resource cause")
    };
    denial
}

#[test]
fn cancellation_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::cancellation_through_both_public_doors"),
        || {
            let source = CancellationSource::new();
            source.cancel();
            let mut request = request(32 * 1024 * 1024, 1_000_000);
            request.cancellation = source.token();
            check_requests(request, |stop| {
                let Outcome::Denied(denial) = stop else {
                    panic!("lost cancellation cause");
                };
                assert_eq!(denial.execution_denial_cause(), Some(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)));
            });
        },
    );
}

#[test]
fn deadline_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::deadline_through_both_public_doors"),
        || {
            let mut request = request(32 * 1024 * 1024, 1_000_000);
            request.deadline = Some(Instant::now());
            check_requests(request, |stop| {
                let Outcome::Denied(denial) = stop else {
                    panic!("lost deadline cause");
                };
                assert_eq!(denial.execution_denial_cause(), Some(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut)));
            });
        },
    );
}

#[test]
fn work_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::work_through_both_public_doors"),
        || {
            check_requests(request(32 * 1024 * 1024, 0), |stop| {
                assert_eq!(resource(stop), Resource::WorkExhausted)
            });
        },
    );
}

#[test]
fn policy_memory_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::policy_memory_through_both_public_doors"),
        || {
            check_requests(request(0, 1_000_000), |stop| {
                let Outcome::Denied(failure) = stop else {
                    panic!("wrong policy stop")
                };
                assert!(matches!(
                    failure.kind(),
                    Kind::ExecutionResource {
                        denial: Resource::MemoryLimit {
                            requested: 1..,
                            admitted: 0,
                            level: Level::Policy
                        },
                        policy_ancestor: Some(0),
                        ..
                    }
                ));
            });
        },
    );
}

#[test]
fn process_memory_through_both_public_doors() {
    in_isolated_process(
        concat!(module_path!(), "::process_memory_through_both_public_doors"),
        || {
            let competing = authority()
                .request_lease(request(64 * 1024 * 1024, 1_000_000))
                .unwrap();
            let _held = competing.reserve_memory(64 * 1024 * 1024).unwrap();
            check_requests(request(32 * 1024 * 1024, 1_000_000), |stop| {
                assert!(matches!(
                    resource(stop),
                    Resource::MemoryLimit {
                        requested: 1..,
                        admitted: 0,
                        level: Level::Process,
                    }
                ));
            });
        },
    );
}

#[test]
fn healthy_fixture_completes_both_leased_preparations() {
    in_isolated_process(
        concat!(
            module_path!(),
            "::healthy_fixture_completes_both_leased_preparations"
        ),
        || {
            let runtime = RelationalRuntimeApi::builder()
                .schema_registry(schema())
                .build();
            let created = transaction(&runtime, "source").commit(&runtime).unwrap();
            let index = related_index(&runtime);
            let lease = authority()
                .request_lease(request(32 * 1024 * 1024, 1_000_000))
                .unwrap();
            let outcome = runtime.index_authority().build_for_commit_with_lease(
                DerivedIndexBuildRequest {
                    source_commit_id: created.commit.commit_id,
                    branch_id: BranchId("main".to_owned()),
                    index_ids: vec![index.index_id],
                },
                &lease,
            );
            assert!(outcome.execution_denial.is_none());
            assert!(outcome.basis_denial.is_none());
            assert!(outcome.failed_indexes.is_empty());
            assert_eq!(outcome.generations.len(), 1);
            let before = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0;
            let _prepared = runtime
                .prepare_branch_transaction_with_lease(transaction(&runtime, "target"), &lease)
                .expect("the same fixture admits ordinary leased preparation");
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        },
    );
}

#[cfg(test)]
mod custom_kernels;
#[cfg(test)]
mod nested_leases;
#[cfg(test)]
mod result_ceiling;
