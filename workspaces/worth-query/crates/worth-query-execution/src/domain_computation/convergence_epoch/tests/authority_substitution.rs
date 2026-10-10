use std::sync::Arc;

use super::fixture::{
    direct_admission_fixture, DirectAdmissionFixture, FixtureDisposition, FixtureFamilyMismatch,
};
use crate::domain_computation::{
    WorthQueryConvergenceEpochDenialKind, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryExecutionRuntimeInstaller,
    WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
};

#[test]
fn foreign_runtime_cannot_admit_an_exact_operation_contract_and_run() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let fixture = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        let foreign = WorthQueryExecutionRuntimeInstaller::new()
            .install(
                worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
                std::iter::empty(),
            )
            .unwrap()
            .into_parts()
            .0;
        let DirectAdmissionFixture {
            runtime,
            operation,
            alternate_basis_operation: _,
            contract,
            managed,
            graph,
            bridge: _,
        } = fixture;
        let rejection =
            match foreign.admit_direct_convergence_epoch(&operation, contract, managed, graph) {
                Ok(_) => panic!("foreign runtime admitted convergence authorities"),
                Err(rejection) => rejection,
            };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ForeignQueryRuntime
        );
        let (contract, managed, graph) = rejection.into_parts();
        finish(execution, runtime, operation, contract, managed, graph);
    });
}

#[test]
fn copied_contract_meaning_cannot_substitute_for_installed_contract_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        let second = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        let DirectAdmissionFixture {
            runtime: first_runtime,
            operation: first_operation,
            alternate_basis_operation: _,
            contract: first_contract,
            managed: first_managed,
            graph: first_graph,
            bridge: _,
        } = first;
        let DirectAdmissionFixture {
            runtime: second_runtime,
            operation: second_operation,
            alternate_basis_operation: _,
            contract: second_contract,
            managed: second_managed,
            graph: second_graph,
            bridge: _,
        } = second;
        let rejection = match first_runtime.admit_direct_convergence_epoch(
            &first_operation,
            second_contract,
            first_managed,
            first_graph,
        ) {
            Ok(_) => panic!("foreign installed convergence contract substituted by meaning"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ContractOperationMismatch
        );
        let (second_contract, first_managed, first_graph) = rejection.into_parts();
        finish(
            execution,
            first_runtime,
            first_operation,
            first_contract,
            first_managed,
            first_graph,
        );
        finish(
            execution,
            second_runtime,
            second_operation,
            second_contract,
            second_managed,
            second_graph,
        );
    });
}

#[test]
fn managed_run_and_graph_authorities_cannot_cross_installed_worlds() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        let second = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        let DirectAdmissionFixture {
            runtime: first_runtime,
            operation: first_operation,
            alternate_basis_operation: _,
            contract: first_contract,
            managed: first_managed,
            graph: first_graph,
            bridge: _,
        } = first;
        let DirectAdmissionFixture {
            runtime: second_runtime,
            operation: second_operation,
            alternate_basis_operation: _,
            contract: second_contract,
            managed: second_managed,
            graph: second_graph,
            bridge: _,
        } = second;
        let rejection = match first_runtime.admit_direct_convergence_epoch(
            &first_operation,
            first_contract,
            second_managed,
            first_graph,
        ) {
            Ok(_) => panic!("foreign managed run entered convergence"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ManagedRunOperationMismatch
        );
        let (first_contract, second_managed, first_graph) = rejection.into_parts();
        let graph_rejection = match first_runtime.admit_direct_convergence_epoch(
            &first_operation,
            first_contract,
            first_managed,
            second_graph,
        ) {
            Ok(_) => panic!("foreign graph authority entered convergence admission"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            graph_rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::GraphOperationMismatch
        );
        let (first_contract, first_managed, second_graph) = graph_rejection.into_parts();
        finish(
            execution,
            first_runtime,
            first_operation,
            first_contract,
            first_managed,
            first_graph,
        );
        finish(
            execution,
            second_runtime,
            second_operation,
            second_contract,
            second_managed,
            second_graph,
        );
    });
}

#[test]
fn same_installed_operation_cannot_substitute_a_different_semantic_basis() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let DirectAdmissionFixture {
            runtime,
            operation,
            alternate_basis_operation,
            contract,
            managed,
            graph,
            bridge: _,
        } = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
        assert_ne!(
            operation.binding_identity(),
            alternate_basis_operation.binding_identity()
        );

        let rejection = match runtime.admit_direct_convergence_epoch(
            &alternate_basis_operation,
            contract,
            managed,
            graph,
        ) {
            Ok(_) => panic!("managed run crossed its admitted semantic basis"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ManagedRunOperationMismatch
        );
        assert_eq!(
            rejection
                .denial()
                .counters()
                .managed_run_authority_check_count(),
            1
        );
        assert_eq!(
            rejection.denial().counters().graph_authority_check_count(),
            0
        );
        let (contract, managed, graph) = rejection.into_parts();
        finish(execution, runtime, operation, contract, managed, graph);
    });
}

#[test]
fn stale_operation_generation_denies_before_graph_or_provider_work() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let DirectAdmissionFixture {
        mut runtime,
        operation,
        alternate_basis_operation: _,
        contract,
        managed,
        graph,
        bridge: _,
    } = direct_admission_fixture(FixtureDisposition::Converged, resource_request);
    let successor = Arc::new(runtime.installed_packages().successor_generation());
    runtime
        .commit_successor_installation(successor)
        .expect("fixture successor must advance the runtime root");

    let rejection =
        match runtime.admit_direct_convergence_epoch(&operation, contract, managed, graph) {
            Ok(_) => panic!("stale operation generation entered convergence"),
            Err(rejection) => rejection,
        };
    assert_eq!(
        rejection.denial().kind(),
        WorthQueryConvergenceEpochDenialKind::StaleInstallationGeneration
    );
    assert_eq!(
        rejection.denial().counters().graph_authority_check_count(),
        0
    );
    let (_contract, managed, _graph) = rejection.into_parts();
    cleanup_unstarted_run(managed);
}

#[test]
fn every_installed_provider_family_mismatch_denies_before_epoch_construction() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    for mismatch in [
        FixtureFamilyMismatch::Universe,
        FixtureFamilyMismatch::Termination,
        FixtureFamilyMismatch::Feasibility,
        FixtureFamilyMismatch::Comparison,
        FixtureFamilyMismatch::Incumbent,
        FixtureFamilyMismatch::Progress,
        FixtureFamilyMismatch::Comparator,
        FixtureFamilyMismatch::RepeatedState,
    ] {
        let DirectAdmissionFixture {
            runtime,
            operation,
            alternate_basis_operation: _,
            contract,
            managed,
            graph,
            bridge: _,
        } = direct_admission_fixture(
            FixtureDisposition::FamilyMismatch(mismatch),
            resource_request,
        );

        let rejection =
            match runtime.admit_direct_convergence_epoch(&operation, contract, managed, graph) {
                Ok(_) => panic!("mismatched installed semantic family entered convergence"),
                Err(rejection) => rejection,
            };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ConvergenceProviderFamilyMismatch
        );
        assert_eq!(
            rejection.denial().counters().graph_authority_check_count(),
            1
        );
        let (_contract, managed, _graph) = rejection.into_parts();
        cleanup_unstarted_run(managed);
    }
}

fn finish(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    runtime: crate::domain_computation::WorthQueryExecutionRuntime,
    operation: crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    contract: worth_query_admission::facade::convergence_epoch::WorthQueryAdmittedConvergenceContract,
    managed: crate::domain_computation::WorthQueryAdmittedDirectRun,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) {
    let epoch = admit(&runtime, &operation, contract, managed, graph);
    finish_epoch(execution, epoch);
}

fn admit(
    runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    contract: worth_query_admission::facade::convergence_epoch::WorthQueryAdmittedConvergenceContract,
    managed: crate::domain_computation::WorthQueryAdmittedDirectRun,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) -> crate::domain_computation::WorthQueryIteratingDirectConvergenceEpoch {
    match runtime.admit_direct_convergence_epoch(operation, contract, managed, graph) {
        Ok(epoch) => epoch.start(),
        Err(_) => panic!("recovered exact authorities must admit"),
    }
}

fn finish_epoch(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    epoch: crate::domain_computation::WorthQueryIteratingDirectConvergenceEpoch,
) {
    let started = match epoch.begin_iteration(execution, call("substitution-recovery")) {
        Ok(started) => started,
        Err(_) => panic!("recovered exact graph authority must start"),
    };
    let outcome = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("fixture provider must complete and rejoin"),
    };
    let terminal = match outcome {
        WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("recovered fixture must converge"),
    };
    if terminal.cleanup().is_err() {
        panic!("recovered convergence terminal must clean up");
    }
}

fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

fn cleanup_unstarted_run(managed: crate::domain_computation::WorthQueryAdmittedDirectRun) {
    let terminal = match managed.start().completed() {
        Ok(terminal) => terminal,
        Err(_) => panic!("an unstarted managed run must remain cleanly terminalizable"),
    };
    if terminal.cleanup().is_err() {
        panic!("denied convergence admission must return managed cleanup authority");
    }
}
