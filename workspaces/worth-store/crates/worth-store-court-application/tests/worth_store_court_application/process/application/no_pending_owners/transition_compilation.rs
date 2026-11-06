//! Genuine compilation refusals and retained roots through the public owners.
mod fresh_installation;
mod source_metadata;

#[test]
fn refused_target_schema_compilation_does_not_consume_an_authority_identity() {
    let ordinary = super::transition_steps::steps(true, false, false);
    let after_refusal = super::transition_steps::steps(true, false, true);
    assert_eq!(after_refusal, ordinary,
        "the actual next RootSchemaAuthority identity must equal the successful control, not merely its allocation count");
}

#[test]
fn the_actual_target_root_keeps_its_compilation_budget_after_preparation_drops() {
    super::transition_steps::steps(true, false, false);
}

#[test]
fn cancellation_during_target_metadata_compilation_releases_backing_before_any_effect() {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use worth_execution::{
        with_request_consult_observer_for_test, CancellationSource, MapKernelStop,
        SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
    };
    use worth_relational::facade::{
        runtime::RelationalHomeProgramContractDenial as Denial,
        transactions::RelationalExecutionDenialCause as Cause,
    };
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let first = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let evaluated = Arc::new(AtomicUsize::new(0));
    let compile = |request: &ExecutionRequest<'_, '_>| {
        native.prepare_home_program_contracts(
            request,
            first.basis().relational_basis(),
            native.config().schema.registry.clone(),
            vec![
                worth_relational::facade::runtime::CustomInvariantRegistration::new(
                    super::transition_executor::TargetRule(evaluated.clone()),
                )
                .unwrap(),
            ],
            Vec::new(),
        )
    };
    let baseline_memory = SerialMemoryBudget::new(None);
    let baseline_request = SerialRequest::from_memory(
        baseline_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let (baseline, total) = with_request_consult_observer_for_test(
        |_| {},
        || compile(&ExecutionRequest::serial(&baseline_request)),
    );
    drop(baseline.unwrap());
    assert_eq!(baseline_memory.charged_memory_bytes(), 0);
    assert!(
        total > 6,
        "the genuine compilation has multiple consultation boundaries"
    );
    for stop in [total / 3, total * 2 / 3, total] {
        let memory = SerialMemoryBudget::new(None);
        let cancellation = CancellationSource::new();
        let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
        let (outcome, observed) = with_request_consult_observer_for_test(
            move |ordinal| {
                if ordinal == stop {
                    cancellation.cancel();
                }
            },
            || compile(&ExecutionRequest::serial(&request)),
        );
        assert!(observed >= stop);
        assert!(matches!(
            outcome,
            Err(Denial::Request(WorkCeilingDenial::Stopped(
                MapKernelStop::Cancelled
            ))) | Err(Denial::Execution(Cause::Cancelled))
        ));
        assert_eq!(memory.charged_memory_bytes(), 0);
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap(),
            first
        );
    }
}

#[test]
fn nonempty_source_and_native_rule_compilation_obeys_the_original_memory_and_cancellation_policy() {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use worth_execution::{
        with_request_consult_observer_for_test, CancellationSource, MapKernelStop,
        SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
    };
    use worth_relational::facade::{
        runtime::RelationalHomeProgramContractDenial as Denial,
        transactions::RelationalExecutionDenialCause as Cause,
    };
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let first = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let evaluated = Arc::new(AtomicUsize::new(0));
    let mut previous_charge = 0;
    for routes in [0, 8] {
        let compile = |request: &ExecutionRequest<'_, '_>| {
            let (registry, invariants, sources) = source_metadata::inventory(
                &native.config().schema.registry,
                routes,
                evaluated.clone(),
            );
            native.prepare_home_program_contracts(
                request,
                first.basis().relational_basis(),
                registry,
                invariants,
                sources,
            )
        };
        let memory = SerialMemoryBudget::new(None);
        let request =
            SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
        let (prepared, total) = with_request_consult_observer_for_test(
            |_| {},
            || compile(&ExecutionRequest::serial(&request)),
        );
        let prepared = prepared.unwrap();
        let charged = memory.charged_memory_bytes();
        assert!(
            charged > previous_charge,
            "the wider actual Source inventory must retain its own backing"
        );
        previous_charge = charged;
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        drop(prepared);
        assert_eq!(memory.charged_memory_bytes(), 0);
        let limited = SerialMemoryBudget::new(Some(charged - 1));
        let request =
            SerialRequest::from_memory(limited.clone(), CancellationSource::new().token(), None);
        let denied = match compile(&ExecutionRequest::serial(&request)) {
            Ok(_) => panic!("the original finite budget cannot retain the measured compilation"),
            Err(denied) => denied,
        };
        let cause = match denied {
            Denial::Execution(cause) => cause,
            Denial::Request(cause) => Cause::from(cause),
            other => panic!("memory refusal must remain typed: {other:?}"),
        };
        assert!(matches!(cause, Cause::PolicyMemoryExhausted { .. }));
        assert_eq!(limited.charged_memory_bytes(), 0);
        for stop in [total / 4, total / 2, total * 3 / 4, total] {
            let memory = SerialMemoryBudget::new(None);
            let cancellation = CancellationSource::new();
            let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
            let (outcome, observed) = with_request_consult_observer_for_test(
                move |ordinal| {
                    if ordinal == stop {
                        cancellation.cancel();
                    }
                },
                || compile(&ExecutionRequest::serial(&request)),
            );
            assert!(observed >= stop);
            assert!(matches!(
                outcome,
                Err(Denial::Execution(Cause::Cancelled))
                    | Err(Denial::Request(WorkCeilingDenial::Stopped(
                        MapKernelStop::Cancelled
                    )))
            ));
            assert_eq!(memory.charged_memory_bytes(), 0);
        }
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap(),
            first
        );
    }
}

#[test]
fn fresh_native_installation_refuses_original_memory_before_publication_and_funds_real_runtime_forks(
) {
    use super::*;
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    use worth_relational::facade::{
        runtime::RelationalInitialSchemaInstallationDenialKind as Kind,
        transactions::RelationalExecutionDenialCause as Cause,
    };
    let directory = tempfile::tempdir().unwrap();
    let setup = crate::workload::serial_request();
    let setup = ExecutionRequest::serial(&setup);
    let mut home = fresh_installation::open(directory.path(), &setup);
    let memory = SerialMemoryBudget::new(Some(0));
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let attempt =
        fresh_installation::install(&mut home, &setup, &ExecutionRequest::serial(&request));
    let refused = match attempt.result {
        Ok(_) => panic!("the actual zero policy must refuse installation"),
        Err(refused) => refused,
    };
    assert!(matches!(
        refused.kind(),
        Kind::Request(Cause::PolicyMemoryExhausted { .. })
    ));
    assert_eq!(attempt.before, attempt.after);
    assert_eq!(memory.charged_memory_bytes(), 0);
    // A new genuine Fresh permission reads the actual medium and refuses if an
    // initial-input artifact already exists. This success independently proves
    // that the refused attempt neither published nor changed installed meaning.
    let memory = SerialMemoryBudget::new(None);
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let completed =
        fresh_installation::install(&mut home, &setup, &ExecutionRequest::serial(&request));
    assert_ne!(completed.before, completed.after);
    let (receipt, fork) = completed.result.unwrap();
    fresh_installation::release(home, receipt, fork, &memory);
}

#[test]
fn cancellation_during_fresh_native_rule_emission_refunds_backing_before_publication_or_cutover() {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use worth_execution::{
        with_request_consult_observer_for_test, CancellationSource, SerialMemoryBudget,
        SerialRequest,
    };
    use worth_relational::facade::{
        runtime::RelationalInitialSchemaInstallationDenialKind as Kind,
        transactions::RelationalExecutionDenialCause as Cause,
    };
    let directory = tempfile::tempdir().unwrap();
    let setup = crate::workload::serial_request();
    let setup = ExecutionRequest::serial(&setup);
    let mut home = fresh_installation::open(directory.path(), &setup);
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let observed = Arc::new(AtomicBool::new(false));
    let observation = observed.clone();
    let (attempt, _) = with_request_consult_observer_for_test(
        move |_| {
            if observation.load(Ordering::SeqCst) {
                return;
            }
            // Diagnostic fault selection observes the actual call stack at the
            // existing SDK consultation. It supplies no authority or replacement
            // path. Missing symbols fail the explicit observation assertion below.
            let trace = std::backtrace::Backtrace::force_capture().to_string();
            if trace.contains("NativeRelationIntegrityInventory::compile")
                && trace.contains("append_relation_integrity_registrations")
            {
                observation.store(true, Ordering::SeqCst);
                eprintln!(
                    "Cancelling the original token in admitted native rule emission:\n{trace}"
                );
                cancellation.cancel();
            }
        },
        || fresh_installation::install(&mut home, &setup, &ExecutionRequest::serial(&request)),
    );
    assert!(
        observed.load(Ordering::SeqCst),
        "the real admitted native emission frame must be observed"
    );
    let refused = match attempt.result {
        Ok(_) => panic!("the original cancellation must stop the real compiler"),
        Err(refused) => refused,
    };
    assert_eq!(refused.kind(), Kind::Request(Cause::Cancelled));
    assert_eq!(attempt.before, attempt.after);
    assert_eq!(memory.charged_memory_bytes(), 0);
    let memory = SerialMemoryBudget::new(None);
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let completed =
        fresh_installation::install(&mut home, &setup, &ExecutionRequest::serial(&request));
    let (receipt, fork) = completed
        .result
        .expect("genuine physical absence survives cancelled compilation");
    fresh_installation::release(home, receipt, fork, &memory);
}
