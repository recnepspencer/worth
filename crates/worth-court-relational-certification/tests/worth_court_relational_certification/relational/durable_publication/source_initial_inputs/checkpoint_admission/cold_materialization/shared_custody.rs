//! Two record prefixes retain one actual provider-issued, funded history hold.
use super::*;
use std::sync::{atomic::Ordering, Mutex};

#[test]
fn shared_history_custody_refuses_unfunded_admission_then_survives_until_the_last_basis_drops() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let (runtime, medium) = initialized(request);
    let mut entities = Vec::new();
    for key in ["first-prefix", "second-prefix", "editable-prefix"] {
        let created = create_entity_outcome(request, &runtime, key);
        entities.push(created.semantic_result().created_entities()[0].assigned);
        release_test_commit_snapshot(&runtime, &created);
    }
    let image = capture_external(&runtime, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    drop(runtime);
    let source = SelectedSource::new(medium.clone());
    let runtime = recover(&image, source.clone(), receiver(&medium), request);
    let control = Arc::new(CustodyControl::default());
    let held = Arc::new(Mutex::new(None));
    let budget = SerialMemoryBudget::new(Some(64 << 20));
    control.exhaust_once(budget.clone(), held.clone());
    source.arm_custody(control.clone());
    let caller =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let result = read_head(&ExecutionRequest::serial(&caller), &runtime, entities[0]);
    assert!(
        matches!(
            result,
            Err(RelationalTransactionReadDenial::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            ))
        ),
        "{result:?}"
    );
    assert_eq!(control.captures.load(Ordering::SeqCst), 0);
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
    drop(
        held.lock()
            .unwrap()
            .take()
            .expect("owned provider retention was reached"),
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    let retry_budget = SerialMemoryBudget::new(None);
    let retry = SerialRequest::from_memory(
        retry_budget.clone(),
        CancellationSource::new().token(),
        None,
    );
    for entity in &entities[..2] {
        read_head(&ExecutionRequest::serial(&retry), &runtime, *entity).unwrap();
    }
    assert_eq!(
        control.members.load(Ordering::SeqCst),
        1,
        "both prefixes share the same actual history member"
    );
    assert_eq!(
        control.captures.load(Ordering::SeqCst),
        1,
        "one counted region admission issues one shared hold"
    );
    assert_eq!(control.live.load(Ordering::SeqCst), 1);
    assert!(retry_budget.charged_memory_bytes() > 0);
    update(request, &runtime, entities[2]);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    drop((runtime, retry, caller));
    assert_eq!(
        control.live.load(Ordering::SeqCst),
        1,
        "the surviving basis retains the actual provider hold"
    );
    assert!(retry_budget.charged_memory_bytes() > 0);
    drop(basis);
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
    assert_eq!(retry_budget.charged_memory_bytes(), 0);
}
