//! Real Source preparation shares one original invoice, including refusal and mixed work.
use crate::relational::managed_source::{created, owner, registry};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_execution::{ExecutionWorkCeiling, WorkCeilingDenial};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_relational::facade::{identity::*, runtime::*, symbols::ClientKey, transactions::*};
#[path = "managed_source_work_account/participant.rs"]
mod participant;
mod public_only;
mod support_growth;

const EXTRA: u64 = 65_536;
#[derive(Clone, Copy, Debug)]
enum Mode {
    Serial,
    Leased,
    Inherited,
    NativeOnly,
}
#[derive(Clone, Copy, Debug, Default)]
struct Scenario {
    custom: u64,
    native: u64,
    refuse: bool,
    cancel_parent: bool,
    rule_limit: bool,
    public_only: bool,
    public_also: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Terminal {
    Committed,
    Provider,
    RuleLimit,
    Request(RelationalExecutionDenialCause),
}
#[derive(Debug)]
struct Sample {
    work: u64,
    terminal: Terminal,
    reached: usize,
}

fn policy(work: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(8 * 1024 * 1024),
            Some(work),
        ),
    )
}
fn sample(mode: Mode, scenario: Scenario, ceiling: u64) -> Sample {
    let parent = CancellationSource::new();
    let child = CancellationSource::new();
    let storage = SerialRequest::from_policy(&policy(ceiling), parent.token(), None);
    let native = SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ),
        child.token(),
        None,
    );
    let lease = matches!(mode, Mode::Leased).then(|| {
        crate::leased_preparation::ordinary_context::authority()
            .request_lease(worth_execution::LeaseRequest {
                policy: policy(ceiling),
                cancellation: parent.token(),
                deadline: None,
            })
            .unwrap()
    });
    let execution = match mode {
        Mode::Serial => ExecutionRequest::serial(&storage),
        Mode::Leased => ExecutionRequest::leased(lease.as_ref().unwrap()),
        Mode::Inherited | Mode::NativeOnly => ExecutionRequest::serial(&native),
    };
    let (registry, program) = registry();
    let court = Arc::new(owner::Court::default());
    let reached = Arc::new(AtomicUsize::new(0));
    let builder = RelationalRuntimeApi::builder().schema_registry(registry);
    let builder = if scenario.public_also {
        builder.managed_index_source(program.clone())
    } else {
        builder
    };
    let builder = if scenario.public_only {
        builder.managed_index_source(program)
    } else {
        let rule = CustomInvariantRegistration::new(owner::Reader {
            program: program.clone(),
            court: court.clone(),
            maximum_work: scenario.rule_limit.then_some(std::num::NonZeroU64::MIN),
        })
        .unwrap()
        .with_managed_index_sources(vec![program])
        .unwrap();
        builder.custom_invariant(rule)
    };
    let runtime = builder.build().unwrap();
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(participant::Participant {
            owner: owner::SourceOwner(court.clone()),
            scenario,
            parent,
            reached: reached.clone(),
        }))
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let batch = WorkerIntentBatch::new("source-work-account")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("scope"),
            fields: Default::default(),
        })))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("item"),
            fields: Default::default(),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("scope-item"),
                source: EntityReference::Created(created(1, "scope")),
                target: EntityReference::Created(created(4, "item")),
                fields: Default::default(),
            },
        )));
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let (outcome, work) = if matches!(mode, Mode::NativeOnly) {
        (transaction.commit(&execution, &runtime), 0)
    } else {
        let (outcome, report) = ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(
                lease.as_ref(),
                if matches!(mode, Mode::Leased) {
                    None
                } else {
                    Some(&storage)
                },
                |_| transaction.commit(&execution, &runtime),
            )
            .unwrap();
        (outcome, report.charged_work())
    };
    let terminal = classify(&outcome);
    let after = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    if terminal == Terminal::Committed {
        assert_ne!(after, before);
    } else {
        assert_eq!(
            after, before,
            "{mode:?} {scenario:?}: refused Source moves no Native head"
        );
    }
    if scenario.cancel_parent {
        assert!(!child.is_cancelled(), "the serial child token stays live");
    }
    drop(outcome);
    drop(registration);
    drop(runtime);
    drop(court);
    drop(basis);
    if let Some(lease) = &lease {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    Sample {
        work,
        terminal,
        reached: reached.load(Ordering::SeqCst),
    }
}
fn classify(outcome: &Result<CommitResult, TransactionCommitError>) -> Terminal {
    match outcome {
        Ok(_) => Terminal::Committed,
        Err(TransactionCommitError::Execution { denial, .. }) => match denial.kind {
            CommitExecutionDenialKind::Cause(cause) => Terminal::Request(cause),
        },
        Err(TransactionCommitError::Preparation { error, .. }) => {
            let source = error
                .managed_source_denial()
                .expect("actual Source preparation refusal");
            if participant::is_refused(source) {
                Terminal::Provider
            } else if matches!(
                source,
                ManagedIndexSourcePreparationDenial::Source(DerivedRelationReadDenial::Structural(
                    StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded)
                ))
            ) {
                Terminal::RuleLimit
            } else {
                panic!("unexpected Source refusal: {source:?}")
            }
        }
        other => panic!("unexpected native terminal: {other:?}"),
    }
}

#[test]
fn source_preparation_custom_and_native_work_share_one_original_invoice() {
    let _serial = crate::leased_preparation::ordinary_context::serial();
    for mode in [Mode::Serial, Mode::Leased, Mode::Inherited] {
        for refuse in [false, true] {
            let base = sample(
                mode,
                Scenario {
                    refuse,
                    ..Default::default()
                },
                1_000_000,
            );
            for (custom, native) in [(EXTRA, 0), (0, EXTRA), (EXTRA / 2, EXTRA / 2)] {
                let changed = sample(
                    mode,
                    Scenario {
                        custom,
                        native,
                        refuse,
                        ..Default::default()
                    },
                    1_000_000,
                );
                assert_eq!(changed.terminal, base.terminal);
                assert_eq!(
                    changed.work,
                    base.work + EXTRA,
                    "{mode:?}: actual custom/native visits bill exactly once, even on refusal"
                );
                assert_eq!(changed.reached, 1);
            }
        }
        let base = sample(mode, Scenario::default(), EXTRA - 1);
        assert_eq!(
            base.terminal,
            Terminal::Committed,
            "{mode:?}: unchanged Source setup must fit the insufficient-extra-work twin: {base:?}"
        );
        for (custom, native) in [(EXTRA, 0), (EXTRA / 2, EXTRA / 2)] {
            let stopped = sample(
                mode,
                Scenario {
                    custom,
                    native,
                    ..Default::default()
                },
                EXTRA - 1,
            );
            assert_eq!(
                stopped.terminal,
                Terminal::Request(RelationalExecutionDenialCause::WorkExhausted)
            );
            assert_eq!(
                stopped.reached, 1,
                "the actual Source receiver exhausted the original account"
            );
            assert!(stopped.work < EXTRA);
            assert!(stopped.work > base.work);
        }
    }
    let native = sample(
        Mode::NativeOnly,
        Scenario {
            custom: EXTRA,
            native: EXTRA,
            ..Default::default()
        },
        0,
    );
    assert_eq!(native.terminal, Terminal::Committed);
    assert_eq!(native.work, 0);
}

#[test]
fn source_preparation_distinguishes_installed_allowance_and_inherited_parent_cancellation() {
    let rule = sample(
        Mode::Serial,
        Scenario {
            rule_limit: true,
            ..Default::default()
        },
        1_000_000,
    );
    assert_eq!(rule.terminal, Terminal::RuleLimit);
    let cancelled = sample(
        Mode::Inherited,
        Scenario {
            cancel_parent: true,
            ..Default::default()
        },
        1_000_000,
    );
    assert_eq!(
        cancelled.terminal,
        Terminal::Request(RelationalExecutionDenialCause::Cancelled)
    );
    assert_eq!(cancelled.reached, 1);
    assert!(cancelled.work > 0);
}
