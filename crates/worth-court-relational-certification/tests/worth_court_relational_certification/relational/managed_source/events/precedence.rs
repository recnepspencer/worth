//! Native relation laws explain a cycle before its dependent Source is prepared.
use super::*;
use fixture::*;

#[test]
fn native_acyclicity_precedes_ordered_source_preparation_and_the_rewire_twin_publishes() {
    let parts = registry::parts();
    let registry = registry::base_with_predecessor_laws(vec![AcyclicityContractDeclaration {
        contract_id: "event-predecessor:acyclic".into(),
        traversal_direction: DirectedTraversalKind::SourceToTarget,
        allowed_cycle_class: AllowedCycleClass::NoCycles,
    }])
    .register_derived_relations(vec![parts.clone()])
    .unwrap();
    let path = registry.derived_relation(&parts.identity).unwrap();
    let program = registry
        .bind_managed_index_source(&path, vec![], vec![])
        .unwrap();
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(Reader {
        base: owner::Reader {
            maximum_work: None,
            program: program.clone(),
            court: court.clone(),
        },
        path,
        control: Control::default(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let world = seed(&runtime, &execution);
    let [a, b, _, _, _] = world.events;
    let born = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("lawful-predecessor").push(link(5, b, a, "ba")),
    );
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
    let denied = try_commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("cycle").push(link(5, a, b, "ab")),
    )
    .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = denied else {
        panic!("the Native acyclicity refusal is required: {denied:?}");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::Acyclicity {
                contract_id,
                relation_kind_id,
                source,
                target,
            },
        ..
    } = error.class
    else {
        panic!("the declared Native cycle law is required: {error:?}");
    };
    assert_eq!(contract_id.as_str(), "event-predecessor:acyclic");
    assert_eq!(relation_kind_id, KindId(5));
    assert_eq!(source, EntityReference::Existing(a));
    assert_eq!(target, EntityReference::Existing(b));
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1,
        before
    );
    assert!(court.staged.lock().unwrap().is_empty());
    assert!(court.observed.lock().unwrap().is_empty());
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("rewire-twin")
            .push(delete(relation(&born, 5, b, a, "ba")))
            .push(link(5, a, b, "ab")),
    );
    assert!(court.staged.lock().unwrap().contains(&a));
    assert!(court.observed.lock().unwrap().contains(&a));
}
