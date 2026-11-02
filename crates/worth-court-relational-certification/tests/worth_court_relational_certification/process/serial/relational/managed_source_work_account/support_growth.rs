//! Real fanout observes the original payer after canonical scope discovery.
use super::*;
use std::sync::{atomic::AtomicU64, Mutex};
use worth_execution::SerialMemoryBudget;
use worth_relational::facade::mvcc::*;
mod fixture;

#[derive(Debug)]
struct Controls {
    original: SerialMemoryBudget,
    cancel: Option<CancellationSource>,
}
#[derive(Debug)]
struct Participant {
    owner: owner::SourceOwner,
    controls: Mutex<Controls>,
    observed: Arc<AtomicU64>,
    rows: Mutex<Vec<EntityId>>,
    count: usize,
}
impl RelationalPublicationCompanion for Participant {
    fn prepare_managed_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, ManagedIndexSourcePreparationDenial> {
        let controls = self.controls.lock().unwrap();
        assert_eq!(
            self.observed
                .swap(controls.original.charged_memory_bytes(), Ordering::SeqCst),
            0
        );
        let mut scopes = source.scopes();
        let scope = scopes.next().expect("genuine created directory");
        assert!(scopes.next().is_none());
        let rows = scope.proposed();
        let read = rows.acquire_all()?;
        let actual: Vec<_> = rows
            .occurrences(&read)?
            .iter()
            .map(|row| row.entity())
            .collect();
        assert_eq!(actual.len(), self.count);
        *self.rows.lock().unwrap() = actual;
        if let Some(parent) = &controls.cancel {
            parent.cancel();
            return Err(rows
                .acquire_all()
                .err()
                .expect("original cancellation stops real Source read")
                .into());
        }
        self.owner.prepare_managed_source(execution, source)
    }
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.owner.prepare(context)
    }
}

struct Sample {
    committed: bool,
    provider_bytes: u64,
}
fn sample(count: usize, memory: u64, cancel: bool) -> Sample {
    let parent = CancellationSource::new();
    let storage =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(memory)), parent.token(), None);
    let original = storage.memory().unwrap().clone();
    let (registry, program) = fixture::registry();
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(fixture::Reader(owner::Reader {
        program: program.clone(),
        court: court.clone(),
        maximum_work: None,
    }))
    .unwrap()
    .with_managed_index_sources(vec![program])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let observed = Arc::new(AtomicU64::new(0));
    let participant = Arc::new(Participant {
        owner: owner::SourceOwner(court.clone()),
        controls: Mutex::new(Controls {
            original: original.clone(),
            cancel: cancel.then_some(parent),
        }),
        observed: observed.clone(),
        count,
        rows: Mutex::new(Vec::new()),
    });
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(participant.clone())
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let outcome = attempt(&runtime, &ExecutionRequest::serial(&storage), count);
    let committed = outcome.is_ok();
    let after = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    let provider_bytes = observed.load(Ordering::SeqCst);
    if committed {
        assert_ne!(after, before);
        check_rows(outcome.as_ref().unwrap(), count, &participant, &court);
    } else {
        assert_eq!(
            after, before,
            "failed Source preparation changes no Native head"
        );
        if cancel {
            assert_eq!(
                super::classify(&outcome),
                Terminal::Request(RelationalExecutionDenialCause::Cancelled)
            );
            assert!(provider_bytes > 0);
        } else {
            let error = outcome.as_ref().unwrap_err();
            assert!(
                matches!(error, TransactionCommitError::Execution { denial, .. } if matches!(denial.kind,
                CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted, .. }
                | RelationalExecutionDenialCause::DeclaredMemoryExhausted { requested, admitted }) if requested > admitted)),
                "actual original-memory refusal: {error:?}"
            );
            assert_eq!(
                provider_bytes, 0,
                "original memory refuses before the participant"
            );
        }
    }
    drop(outcome);
    let mut retry_retention = None;
    if !committed {
        assert_eq!(
            original.charged_memory_bytes(),
            0,
            "failed actual request ownership released"
        );
        // Same runtime, registration, selected unchanged head and authored batch.
        // Only the original request is freshly funded and uncancelled.
        let retry = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
            CancellationSource::new().token(),
            None,
        );
        let retry_memory = retry.memory().unwrap().clone();
        *participant.controls.lock().unwrap() = Controls {
            original: retry_memory.clone(),
            cancel: None,
        };
        observed.store(0, Ordering::SeqCst);
        let result = attempt(&runtime, &ExecutionRequest::serial(&retry), count)
            .expect("fresh funded lawful retry on the same runtime");
        check_rows(&result, count, &participant, &court);
        assert_ne!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        drop(result);
        retry_retention = Some(retry_memory);
    }
    drop(registration);
    drop(runtime);
    drop(court);
    drop(participant);
    assert_eq!(original.charged_memory_bytes(), 0);
    if let Some(memory) = retry_retention {
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
    Sample {
        committed,
        provider_bytes,
    }
}
fn attempt(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    count: usize,
) -> Result<CommitResult, TransactionCommitError> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(fixture::batch(count), execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, runtime)
}
fn check_rows(
    result: &CommitResult,
    count: usize,
    participant: &Participant,
    court: &owner::Court,
) {
    let scope = result.created_entity(&created(1, "scope")).unwrap();
    let mut expected: Vec<_> = (0..count)
        .map(|index| {
            result
                .created_entity(&created(4, &format!("item-{index}")))
                .unwrap()
        })
        .collect();
    expected.sort();
    assert_eq!(
        *participant.rows.lock().unwrap(),
        expected,
        "exact real authored Source roster, including ordering and empty-inverse exclusion"
    );
    assert_eq!(
        *court.staged.lock().unwrap(),
        std::collections::BTreeSet::from([scope])
    );
    assert_eq!(
        *court.observed.lock().unwrap(),
        std::collections::BTreeSet::from([scope])
    );
}

#[test]
fn source_support_fanout_has_geometric_original_memory_growth() {
    let _serial = crate::leased_preparation::ordinary_context::serial();
    // Predeclared generous envelope includes all original Native preparation
    // backing at the provider boundary; this is not a support-only invoice.
    let small = sample(32, 16 * 1024 * 1024, false);
    let doubled = sample(64, 16 * 1024 * 1024, false);
    eprintln!(
        "Source fanout original provider bytes: 32={}, 64={}",
        small.provider_bytes, doubled.provider_bytes
    );
    assert!(small.committed && doubled.committed);
    assert!(small.provider_bytes > 0);
    assert!(doubled.provider_bytes < 3 * small.provider_bytes);
    assert!(!sample(32, 16 * 1024 * 1024, true).committed);
    assert!(!sample(32, 128 * 1024, false).committed);
}
