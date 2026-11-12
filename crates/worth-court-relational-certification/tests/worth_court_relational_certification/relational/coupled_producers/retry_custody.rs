//! Real caller refusal/cancellation releases an already assembled coupled draft.
use super::*;
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

struct TerminalRule {
    cancellation: Option<CancellationSource>,
    visits: Arc<AtomicUsize>,
}
fn rule_identity() -> CustomInvariantSemanticIdentity {
    CustomInvariantSemanticIdentity {
        rule_id: CustomInvariantRuleId::new("coupled.terminal-cleanup"),
        semantic_version: CustomInvariantSemanticVersion::new(1, 0),
    }
}
impl CustomInvariantRule for TerminalRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: rule_identity(),
            display_name: Arc::from("One coupled output refuses or cancels"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        // Bootstrap creates two entities; only the assembled group creates one.
        if context
            .touched()
            .planned_entity_creates()
            .iter()
            .filter(|row| row.kind_id() == KindId(1))
            .count()
            != 1
        {
            return Ok(CustomInvariantVerdict::Pass);
        }
        self.visits.fetch_add(1, Ordering::SeqCst);
        Ok(if let Some(cancellation) = &self.cancellation {
            cancellation.cancel();
            CustomInvariantVerdict::Pass
        } else {
            CustomInvariantVerdict::Violation(CustomInvariantRefusal::Empty)
        })
    }
}
struct Owned<'a> {
    world: &'a World,
    calls: Cell<usize>,
    duplicate: bool,
}
impl RelationalAuthoredCoupledProducerAssembly for Owned<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        self.calls.set(self.calls.get() + 1);
        for _ in 0..if self.duplicate { 2 } else { 1 } {
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                    admission: self.world.owner.admit_required_output("owned", [3; 32]),
                    absence: Some(self.world.absence(execution, "owned")),
                    meaning: self.world.meaning("owned"),
                    roles: vec![created("owned-output")],
                }),
                &RelationalContentAttachments::default(),
                vec![create("owned-output")],
            )?;
        }
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn terminal_invariant_refusal_releases_the_adopted_coupled_draft() {
    terminal(false);
}
#[test]
fn caller_cancellation_after_applied_candidate_releases_coupled_custody() {
    terminal(true);
}
fn terminal(cancel: bool) {
    let bootstrap = request();
    let cancellation = CancellationSource::new();
    let visits = Arc::new(AtomicUsize::new(0));
    let world = World::with_invariant(
        &ExecutionRequest::serial(&bootstrap),
        CustomInvariantRegistration::new(TerminalRule {
            cancellation: cancel.then(|| cancellation.clone()),
            visits: visits.clone(),
        })
        .unwrap(),
    );
    let before = world.basis();
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let producer = Owned {
        world: &world,
        calls: Cell::new(0),
        duplicate: false,
    };
    let result = world.prepare(execution, &producer).and_then(|proposal| {
        world
            .runtime
            .prepare_validated_proposal(execution, proposal)
    });
    let err = result.unwrap_err();
    if cancel {
        let TransactionCommitError::Execution { denial, .. } = err else {
            panic!("caller cancellation: {err:?}")
        };
        assert_eq!(
            denial.kind,
            CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)
        );
    } else {
        let TransactionCommitError::Conflict { error, .. } = err else {
            panic!("actual invariant refusal: {err:?}")
        };
        let ConflictClass::InvariantViolation { fields, .. } = error.class else {
            panic!("installed rule refusal: {error:?}")
        };
        assert!(
            matches!(fields, InvariantViolationFields::CustomInvariantViolation {
            identity, refusal: CustomInvariantRefusal::Empty
        } if identity == rule_identity())
        );
    }
    assert_eq!(producer.calls.get(), 1, "the handler never reruns");
    assert!(
        visits.load(Ordering::SeqCst) > 0,
        "the applied candidate reached the real invariant"
    );
    assert_eq!(
        world.basis(),
        before,
        "neither source nor producer published"
    );
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "terminal exit releases the actual original-request draft, not a returned self-cycle"
    );
    let _absence = world.absence(&ExecutionRequest::serial(&bootstrap), "owned");
}

#[test]
fn failed_accepted_assembly_releases_its_provisional_coupling() {
    let bootstrap = request();
    let world = World::new(&ExecutionRequest::serial(&bootstrap), true, true);
    let before = world.basis();
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let producer = Owned {
        world: &world,
        calls: Cell::new(0),
        duplicate: true,
    };
    let err = world.prepare(execution, &producer).unwrap_err();
    let TransactionCommitError::Publication { error, .. } = err else {
        panic!("real duplicate owner refusal: {err:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(
            worth_relational::facade::publication::AuthoredRetirementRefusal::RepeatedCoupledOwner
        )
    );
    drop(error);
    assert_eq!(producer.calls.get(), 1);
    assert_eq!(world.basis(), before);
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "failed accepted assembly releases its provisional output and commands"
    );
    let _absence = world.absence(&ExecutionRequest::serial(&bootstrap), "owned");
}

#[test]
fn plan_preview_discards_provisional_coupling_without_publishing() {
    let bootstrap = request();
    let world = World::new(&ExecutionRequest::serial(&bootstrap), true, true);
    let before = world.basis();
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let owners = world
        .runtime
        .readmit_edit_coupled_owners(execution, &world.installed, &before, world.source)
        .unwrap();
    let mut transaction = begin_main_transaction(&world.runtime);
    transaction
        .push_batch(
            update(world.source, "preview-source"),
            execution.allocation_policy(),
        )
        .unwrap();
    let bound = transaction
        .bind_authored_coupled_producers(&world.installed, owners, None)
        .unwrap();
    let producer = Owned {
        world: &world,
        calls: Cell::new(0),
        duplicate: false,
    };
    let mut transaction = world
        .runtime
        .preparation_port()
        .prepare_authored_coupled_transaction(execution, bound, &producer)
        .unwrap()
        .unwrap();
    assert_eq!(
        producer.calls.get(),
        1,
        "real accepted assembly precedes the preview"
    );
    // A new authoring batch invalidates the cached plan, entering actual preview
    // preparation. It does not publish or validate a changed coupled contract.
    transaction
        .push_batch(create("preview-only"), execution.allocation_policy())
        .unwrap();
    assert!(!transaction
        .prepare_merged_plan(execution, &world.runtime)
        .unwrap()
        .merged_intents
        .is_empty());
    assert_eq!(world.basis(), before);
    drop(transaction);
    assert_eq!(budget.charged_memory_bytes(), 0, "preview releases provisional coupling rather than retaining the completed group's self-reference");
    let _absence = world.absence(&ExecutionRequest::serial(&bootstrap), "owned");
}
