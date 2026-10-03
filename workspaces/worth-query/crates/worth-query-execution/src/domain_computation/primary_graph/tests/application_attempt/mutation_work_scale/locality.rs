use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    status_parameter, AccountSummaryQuery,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryProductQueryControls,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LocalWork {
    source_query_before: usize,
    source_query_after: usize,
    mutation_decision_facts: usize,
    mutation_proposed_facts: usize,
    mutation_invariant_work: u64,
    mutation_invariant_executions: usize,
    mutation_touched_records: usize,
}

#[derive(Clone, Copy, Debug)]
struct LocalityRun {
    work: LocalWork,
    installation: Duration,
    population: Duration,
    local_edit: Duration,
}

#[test]
fn same_kind_unrelated_population_keeps_source_query_and_local_edit_work_flat() {
    assert_locality(&[1_000, 10_000]);
}

/// Scheduled 100k scale court, run by exact name with `--ignored --nocapture`.
#[test]
#[ignore = "100k native population is a separately budgeted scale court"]
fn hundred_thousand_same_kind_accounts_preserve_local_source_query_and_edit_work() {
    assert_locality(&[1_000, 100_000]);
}

fn assert_locality(populations: &[usize]) {
    let mut baseline = None;
    for &count in populations {
        let run = run_locality(count);
        eprintln!(
            "Query Account locality: unrelated={count} installation={:?} population={:?} local_edit={:?} work={:?}",
            run.installation, run.population, run.local_edit, run.work
        );
        if let Some(baseline) = baseline {
            assert_eq!(
                run.work, baseline,
                "same-kind population widened local work"
            );
        } else {
            baseline = Some(run.work);
        }
    }
}

fn run_locality(count: usize) -> LocalityRun {
    let installation_start = Instant::now();
    let world = installed_authorization_world(true);
    let installation = installation_start.elapsed();
    let population_start = Instant::now();
    grow_unrelated_accounts(&world, count);
    let population = population_start.elapsed();

    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let source_query_before = source_query_work(&world, &principal, &request);
    let unrelated = resolved_account(&world, "unrelated", &request);
    let program = admitted_program(&world, &principal, &unrelated, &request, "local-edit");
    let local_edit_start = Instant::now();
    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(101, 101));
    let local_edit = local_edit_start.elapsed();
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the unrelated Account edit must commit");
    };
    let mutation = receipt
        .mutation_work()
        .expect("the local edit carries native mutation work");
    let work = LocalWork {
        source_query_before,
        source_query_after: source_query_work(&world, &principal, &request),
        mutation_decision_facts: mutation.decision_fact_count(),
        mutation_proposed_facts: mutation.proposed_fact_count(),
        mutation_invariant_work: mutation.invariant_work_units(),
        mutation_invariant_executions: mutation.relational_invariant_execution_count(),
        mutation_touched_records: mutation.touched_record_count(),
    };
    LocalityRun {
        work,
        installation,
        population,
        local_edit,
    }
}

fn source_query_work(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        crate::domain_computation::primary_graph::tests::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> usize {
    let account = resolved_account(world, "open", request);
    let query = world
        .application
        .installed_schema()
        .certification_query(AccountSummaryQuery::reference())
        .expect("the source query is installed");
    let access = WorthQueryApplicationQueryAccessContext::new(principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_owned())
                .expect("the source parameter is canonical"),
            WorthQueryProductQueryControls::new(
                NonZeroUsize::new(10).unwrap(),
                NonZeroUsize::new(10_000).unwrap(),
                request,
            ),
        )
        .expect("the source query admits at every population scale");
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .expect("the source query executes at every population scale");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.rows()[0].label(), "primary");
    result.receipt().total_work_units()
}
