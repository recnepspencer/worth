//! Real original intent authority refuses substituted author inputs before rules.
use super::transition_open_original as original;
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use worth_relational::facade::{
    mvcc::{RelationalCallerAssociation, RelationalPublicationDenial},
    runtime::RelationalHomeProgramStepContracts,
    transactions::TransactionCommitError,
};
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_store_application::ApplicationHomeTransitionIntentDenial;

#[test]
fn an_original_home_step_refuses_a_substituted_body_before_rules_or_publication() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let evaluated = Arc::new(AtomicUsize::new(0));
    let contracts = original::contracts(&fixture, &execution, evaluated.clone());
    let (grant, opened) = original::original_open(&mut fixture, &contracts, &execution);
    let source = opened.expected_head().clone();
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let native = fixture.home.relational().runtime();
    let transaction = |batch| {
        let mut transaction = native
            .begin_home_program_step(
                &execution,
                source.basis().relational_basis(),
                RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None,
                },
                grant.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
                grant.original(),
                &association,
                grant.admission(),
            )
            .unwrap();
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        transaction
    };
    let refused = transaction(crate::batches::create(1)).validate(&execution, native);
    assert!(matches!(refused, Err(TransactionCommitError::PublicationDenied { denial: RelationalPublicationDenial::HomeProgramTransition(
        worth_relational::facade::durability::RelationalHomeProgramWriteDenial::BodyMismatch), .. })));
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    assert!(native
        .durability_authority()
        .discover_associated_effects(&execution, &association)
        .unwrap()
        .is_empty());
    let proposal = transaction(crate::batches::create(0))
        .validate(&execution, native)
        .unwrap();
    let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    assert_eq!(
        performed.attempt_identity().ordinal(),
        grant.original().attempt().ordinal()
    );
    assert!(evaluated.load(Ordering::SeqCst) > 0);
    drop(performed.consume());
    drop((grant, contracts, source));
    // This bounded court has not published the final descriptor binding. Its
    // original plan remains pending and retains its real operation custody.
    drop(fixture);
}

#[test]
fn changed_body_rosters_refuse_before_claiming_the_original_resumed_step() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let evaluated = Arc::new(AtomicUsize::new(0));
    let recorded_body = {
        let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        original::commit_intent(&mut fixture, &contracts, &execution);
        let recorded_body = fixture
            .home
            .home_program_transition_intent()
            .unwrap()
            .bodies()[0];
        // Interrupt the genuine intent before its first original World open.
        drop((contracts, fixture));
        recorded_body
    };
    let fixture = original::reopen(directory.path(), &execution);
    let contracts = original::contracts(&fixture, &execution, evaluated.clone());
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let target = HomeProgramTransitionTargetMeaning::from_recorded(0, contracts.catalog_digest());
    let digest = HomeProgramTransitionPlanDigest::from_recorded([7; 32]);
    let mut edited = crate::batches::transition_body_roster(&execution, 1);
    edited[0] = crate::batches::transition_body_roster(&execution, 2)[1];
    let refused = fixture.home.admit_unopened_home_program_transition_step(
        0, &source, digest, 1, target, &edited, &execution,
    );
    assert!(matches!(
        refused,
        Err(ApplicationHomeTransitionIntentDenial::TransitionPlanDrift)
    ));
    let grant = fixture
        .home
        .admit_unopened_home_program_transition_step(
            0,
            &source,
            digest,
            1,
            target,
            &crate::batches::transition_body_roster(&execution, 1),
            &execution,
        )
        .unwrap();
    assert_eq!(grant.admission().body_commitment(), recorded_body.bytes());
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    // Still an unfinished transition; abandon it with its real claim custody.
    drop((grant, source, contracts, fixture));
}

#[test]
fn cancellation_inside_a_large_authored_value_stops_the_original_request_and_refunds_its_scope() {
    use worth_execution::{
        with_request_consult_observer_for_test, CancellationSource, SerialMemoryBudget,
        SerialRequest,
    };
    use worth_relational::facade::{
        runtime::{home_program_transition_body_digest, RelationalHomeProgramBodyDenial},
        transactions::{AspectFieldPatch, CreateIntent, MutationIntent},
    };
    let body = |size| {
        let mut batch = crate::batches::create(0);
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
            panic!("the ordinary create fixture must retain its entity input")
        };
        let locator = spec.fields.locators().next().unwrap().clone();
        spec.fields = AspectFieldPatch::from_locator(
            locator,
            worth_foundational::facade::AspectValue::InlineBytes(
                worth_foundational::facade::InlineBytes::try_new(vec![7; size]).unwrap(),
            ),
        );
        [batch]
    };
    let intent = worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary();
    let zero_memory = SerialMemoryBudget::new(Some(0));
    let zero =
        SerialRequest::from_memory(zero_memory.clone(), CancellationSource::new().token(), None);
    // Native work accounting owns a real, fixed framework context. Zero original
    // memory must refuse that context, rather than silently switch accounting off.
    assert!(matches!(
        home_program_transition_body_digest(
            &ExecutionRequest::serial(&zero),
            &intent,
            &body(65536)
        ),
        Err(RelationalHomeProgramBodyDenial::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(_)
            )
        ))
    ));
    assert_eq!(zero_memory.charged_memory_bytes(), 0);
    let memory = SerialMemoryBudget::new(None);
    let baseline =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let (small, small_checks) = with_request_consult_observer_for_test(
        |_| {},
        || {
            home_program_transition_body_digest(
                &ExecutionRequest::serial(&baseline),
                &intent,
                &body(4096),
            )
        },
    );
    let small = small.unwrap();
    let (large, large_checks) = with_request_consult_observer_for_test(
        |_| {},
        || {
            home_program_transition_body_digest(
                &ExecutionRequest::serial(&baseline),
                &intent,
                &body(65536),
            )
        },
    );
    assert_ne!(small, large.unwrap());
    assert!(
        large_checks >= small_checks + 15,
        "each additional payload chunk must consult the original request"
    );
    assert_eq!(memory.charged_memory_bytes(), 0);
    let stop = small_checks + 1;
    let cancellation = CancellationSource::new();
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    // Same body shape and scalar headers; only the final raw payload grows.
    // This checkpoint lies beyond all work needed for the one-chunk control.
    let (outcome, observed) = with_request_consult_observer_for_test(
        move |ordinal| {
            if ordinal == stop {
                cancellation.cancel();
            }
        },
        || {
            home_program_transition_body_digest(
                &ExecutionRequest::serial(&request),
                &intent,
                &body(65536),
            )
        },
    );
    assert!(observed >= stop);
    assert!(matches!(
        outcome,
        Err(RelationalHomeProgramBodyDenial::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(memory.charged_memory_bytes(), 0);
}
