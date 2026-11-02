//! Original-request custody of independent replay/normalized plan copies.
use crate::mvcc::{OwnedMergePlan, RelationalTransactionIntent};
use crate::tests::support::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn replay_input_copy_refuses_before_growth() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let original = batch_create("input-refusal");
    let budget = SerialMemoryBudget::new(Some(0));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&serial);
    let denied = transaction
        .stage_recovered_intents(
            &request,
            crate::history::data::CommitId(1),
            &original.intents,
        )
        .unwrap_err();
    assert!(matches!(denied, TransactionCommitError::Execution { .. }));
    assert!(transaction.batches().is_empty());
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}

#[test]
fn replay_validation_keeps_normalized_copy_after_inputs_drop() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let original = batch_create("accepted-copy");
    let budget = SerialMemoryBudget::new(Some(1_000_000));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&serial);
    transaction
        .stage_recovered_intents(
            &request,
            crate::history::data::CommitId(1),
            &original.intents,
        )
        .unwrap();
    let normalized = {
        let proposal = transaction.validate(&request, &runtime).unwrap();
        assert_eq!(
            proposal
                .state
                .prepared
                .merged_plan
                .plan()
                .merged_intents
                .len(),
            original.intents.len()
        );
        // Move the SAME whole owner. All other proposal/prepared/index owners
        // die at this lexical boundary and cannot mask a missing plan bill.
        proposal.state.prepared.merged_plan
    };
    assert!(
        budget.charged_memory_bytes() > 0,
        "the independent normalized plan alone retains its bill"
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    drop(normalized);
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "only the isolated original plan owner releases its data"
    );
}

#[test]
fn normalized_small_output_does_not_retain_large_input_bill() {
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut original = batch_create("small-value");
    let MutationIntent::Create(CreateIntent::Entity(entity)) = &mut original.intents[0] else {
        panic!("fixture create")
    };
    entity.client_key = crate::symbols::data::ClientKey::raw("raw-key-".repeat(2048));
    let budget = SerialMemoryBudget::new(Some(1_000_000));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&serial);
    transaction
        .stage_recovered_intents(
            &request,
            crate::history::data::CommitId(1),
            &original.intents,
        )
        .unwrap();
    let input_charge = budget.charged_memory_bytes();
    let normalized = {
        let proposal = transaction.validate(&request, &runtime).unwrap();
        let MutationIntent::Create(CreateIntent::Entity(entity)) =
            &proposal.state.prepared.merged_plan.plan().merged_intents[0]
        else {
            panic!("accepted create")
        };
        assert!(
            entity.client_key.as_symbol().is_some(),
            "the genuine normalizer selected the smaller representation"
        );
        // The claim concerns the normalized plan, not the independently
        // charged fixed proposal backing or its other accepted state.
        proposal.into_state().prepared.merged_plan
    };
    assert!(
        budget.charged_memory_bytes() < input_charge,
        "accepted output must not pin the discarded input peak"
    );
    drop(normalized);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn interrupted_copy_round_restores_original_transaction() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let original = batch_create("restored-raw-input");
    transaction
        .push_batch(
            original.clone(),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(1_000_000));
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let result = transaction.with_funded_preparation_overlay(&request, |candidate| {
        let normalized = candidate
            .normalized_intents_for_merge(&runtime.preparation_runtime_snapshot(), &request)?;
        let MutationIntent::Create(CreateIntent::Entity(entity)) =
            &normalized.plan().merged_intents[0]
        else {
            panic!("normalized create")
        };
        assert!(entity.client_key.as_symbol().is_some());
        cancellation.cancel();
        request
            .consult()
            .map_err(crate::mvcc::plan_copy_admission::request_denial)
    });
    assert!(matches!(
        result,
        Err(TransactionCommitError::Execution { .. })
    ));
    assert_eq!(transaction.batches(), &[original]);
    assert!(transaction.last_merged_plan.is_none());
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "abandoned candidate data dies before refund"
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}

#[test]
fn merge_copy_funds_headers_without_rebilling_shared_leaves() {
    let runtime = runtime_with_test_schema();
    let source_serial = crate::test_execution_policy::serial_request();
    let source_request = ExecutionRequest::serial(&source_serial);
    create_entity(&source_request, &runtime, "main-anchor");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        &source_request,
        &runtime,
        "feature-only",
        BranchId("feature".into()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            &source_request,
            crate::merge::data::MergeExecutionRequest {
                target_branch: BranchId("main".into()),
                source_branch: BranchId("feature".into()),
                merge_intent: crate::merge::data::MergeIntent::ReconcileIntoTarget,
            },
        )
        .unwrap();
    let outcome = runtime
        .execute_prepared_merge(&source_request, prepared)
        .unwrap();
    let original = runtime
        .replay()
        .canonical_commit_envelope(&source_request, outcome.commit.commit.commit_id)
        .unwrap()
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(1_000_000));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&serial);
    let copied = OwnedMergePlan::copy_recovered(&original, &request)
        .unwrap()
        .expect("actual governed merge authority");
    let source = &original
        .merge_execution_authority
        .as_ref()
        .unwrap()
        .execution_summary;
    let copy = &copied.plan().merge_execution_summary;
    assert!(!source.correspondence_witness.rows().is_empty());
    assert_eq!(copy, source);
    assert_eq!(
        copy.correspondence_witness.rows().as_ptr(),
        source.correspondence_witness.rows().as_ptr(),
        "same intrinsic row owner, no second data bill"
    );
    assert_eq!(
        copy.branch_basis
            .merge_base()
            .supporting_left_ancestors()
            .as_ptr(),
        source
            .branch_basis
            .merge_base()
            .supporting_left_ancestors()
            .as_ptr()
    );
    assert!(
        budget.charged_memory_bytes() > 0,
        "new independent header/intents are funded"
    );
    drop(copied);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
