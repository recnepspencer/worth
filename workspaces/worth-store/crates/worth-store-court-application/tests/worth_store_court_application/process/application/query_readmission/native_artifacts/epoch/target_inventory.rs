//! A genuine World grant cannot freshen a target compiled in an earlier epoch.
use super::*;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_runtime_world::facade::{
    RuntimeWorldCancellationSource, WorldDurablePublicationOutcome, WorldOpenAttemptOutcome,
};
mod target_rule;
mod target_schema;

#[test]
fn real_home_target_refuses_old_receiver_epoch_and_executes_fresh_inventory() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(directory.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, mut home) = transition_unshared(directory.path(), &phase);
        let initial = query.home_initial_schema_receipt().clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
        let (_, before) = native.observe_branch(&native.main_branch_identity()).unwrap();
        let compile = |native: &RelationalRuntime, basis| native.prepare_home_program_contracts(
            &execution, basis, target_schema::registry(native),
            vec![CustomInvariantRegistration::new(target_rule::TargetRule(calls.clone())).unwrap()], Vec::new()).unwrap();
        let old = compile(native, &before);
        native.replace_equivalent_configuration_epoch_for_test().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let (query, mut home, source_owner) = existing_from_preparation(query, home, &phase);
        let source = home.world().observation_port().observe_root_product_branch().unwrap();
        let native = home.relational().runtime();
        assert_eq!(before.descriptor(), source.basis().relational_basis().descriptor());
        let fresh = compile(native, source.basis().relational_basis());
        assert_eq!(old.catalog_digest(), fresh.catalog_digest());
        let rows = administration::rows(native, &execution, "Account");
        let [row] = rows.as_slice() else { panic!("actual nonempty account"); };
        let original = administration::field(row, FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance").aspect(), "Balance").clone();
        let pinned = administration::view(native);
        let proposed = target_schema::transition(native);
        let change = RelationalTransactionIntent::ordinary().with_schema_transition(proposed.clone(), None);
        let body = batch(row.entity_id);
        let digest = home_program_transition_body_digest(&execution, &change, std::slice::from_ref(&body)).unwrap();
        let reserved = home.reserve_home_program_transition_plan(source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([71; 32]), 1,
            fresh.target_meaning(&execution, 0).unwrap(), &[digest], &execution).unwrap();
        home.stage_home_program_transition_intent(reserved,
            worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution).unwrap();
        loop {
            execution.consult().unwrap();
            match home.advance_home_program_transition_intent(&execution).unwrap() {
                ApplicationHomeTransitionIntentOutcome::Committed => break,
                ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
                other => panic!("actual intent refused: {other:?}"),
            }
        }
        let mut grant = home.admit_home_program_transition_step(0, &source, &execution).unwrap();
        let cancellation = RuntimeWorldCancellationSource::new();
        let reservation = grant.take_reservation().unwrap();
        let pending = home.world().publication_port().prepare_home_transition_step(&execution,
            source.clone(), change, grant.plan(), grant.original(), reservation,
            grant.admission(), &cancellation.token(), None).unwrap();
        let mut outcome = pending.advance(&execution);
        let opened = loop {
            execution.consult().unwrap();
            match outcome {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
                other => panic!("original World open refused: {other:?}"),
            }
        };
        let association = RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let native = home.relational().runtime();
        let begin = |contracts| native.begin_home_program_step(&execution,
            source.basis().relational_basis(), RelationalHomeProgramStepContracts::Target { contracts, transition: Some((proposed.clone(), None)) },
            grant.plan(), HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
            grant.original(), &association, grant.admission(),
worth_relational::facade::indexes::DerivedIndexMaintenanceBudget { maximum_work_units: None, maximum_cold_record_slots: None, maximum_derived_rows: None },
);
        assert!(matches!(begin(&old), Err(RelationalHomeProgramContractDenial::SourceContractChanged)));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(home.world().observation_port().observe_root_product_branch().unwrap(), source);
        let mut transaction = begin(&fresh).unwrap();
        transaction.push_batch(body, execution.allocation_policy()).unwrap();
        let proposal = transaction.validate(&execution, native).unwrap();
        assert!(calls.load(Ordering::SeqCst) > 0, "fresh B inventory actually executes");
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let mut outcome = home.world().publication_port().execute_durable_without_signal(execution, prepared, &cancellation.token());
        let performed = loop {
            execution.consult().unwrap();
            match outcome {
                WorldDurablePublicationOutcome::Performed(performed) => break performed,
                WorldDurablePublicationOutcome::Pending(pending) => outcome = pending.advance(&execution),
                other => panic!("current epoch publication refused: {other:?}"),
            }
        };
        assert_eq!(performed.new_product_head().reference_generation().get(), 1);
        drop(performed.consume());
        assert!(native.config().schema.registry.entity_registration(worth_relational::facade::identity::KindId(2000)).is_err(), "publication did not replace construction A");
        let initial_again = native.readmit_recovered_initial_schema_installation(initial.clone()).unwrap();
        assert_eq!(initial_again.custom_invariant_generation(), initial.custom_invariant_generation());
        assert_eq!(initial_again.custom_invariant_inventory_digest(), initial.custom_invariant_inventory_digest());
        let current_view = administration::view(native);
        let current_pin = native.read_truth().pin_view_read_capability(&current_view, &[], &execution).unwrap();
        assert!(current_pin.schema().entity_registration(worth_relational::facade::identity::KindId(2000)).is_ok());
        let old_pin = native.read_truth().pin_view_read_capability(&pinned, &[], &execution).unwrap();
        assert!(old_pin.schema().entity_registration(worth_relational::facade::identity::KindId(2000)).is_err());
        drop((current_pin, old_pin, current_view));
        let historical = pinned.bounded_entities_of_kind(&execution, row.kind.kind_id, 1024).unwrap().unwrap().into_records();
        assert_eq!(administration::field(&historical[0], FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance").aspect(), "Balance"), &original);
        assert_eq!(administration::field(&administration::rows(native, &execution, "Account")[0], FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance").aspect(), "Balance"), &AspectValue::UInt64(29));
        drop((query, source_owner, source, before, pinned, old, fresh, grant, home));
    }).unwrap();
}
