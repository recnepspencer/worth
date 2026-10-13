//! Leased application hooks keep accounting and refusal at the public door.
use super::super::fixture::isolated_request_owner;
use super::idempotency;
use crate::domain_computation::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementDenial as Advancement, WorthQueryExecutionPlacementForTest as Placement,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use std::num::NonZeroUsize;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[path = "checked_invariant_hooks/schema.rs"]
mod schema;
use schema::HookSchema;
#[path = "checked_invariant_hooks/accounting_rule.rs"]
mod accounting_rule;
#[path = "checked_invariant_hooks/program.rs"]
mod program;
#[path = "checked_invariant_hooks/world.rs"]
mod world;
use accounting_rule::{AccountingRule, Contacts, EVALUATION_UNITS, PREPARATION_UNITS};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
struct Case {
    outcome: WorthQueryApplicationCommitOutcome,
    opening: Result<(), Advancement>,
    work: Option<u64>,
    contacts: [usize; 4],
    publications: usize,
}

fn run_case(work: bool, ceiling: Option<u64>, cancel_in_preparation: bool) -> Case {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let source = WorthQueryCancellationSource::new();
    let contacts = Contacts::default();
    let world = world::install(AccountingRule {
        checked: true,
        work,
        cancellation: cancel_in_preparation.then(|| source.clone()),
        contacts: contacts.clone(),
    });
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + Duration::from_secs(60)),
        source.token(),
    );
    let program = program::prepare(&world, &scope);
    // Installation completes before the rule control targets this commit.
    contacts
        .armed
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let before = contacts.snapshot();
    reports();
    place(Placement::Leased(NonZeroUsize::MIN));
    bound(ceiling.map(|work| {
        worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, work)
    }));
    let mut outcome = None;
    let opening = world
        .application
        .with_application_advancement(&scope, |phase| {
            outcome = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        idempotency(198, 198),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        });
    let requests = reports();
    assert_eq!(
        requests.len(),
        1,
        "the installed caller funds the whole attempt"
    );
    let work = match &requests[0] {
        Ok(report) => Some(report.charged_work()),
        Err(_) => None,
    };
    let after = contacts.snapshot();
    Case {
        outcome: outcome.expect("the public commit door ran"),
        opening,
        work,
        contacts: std::array::from_fn(|index| after[index] - before[index]),
        publications: world
            .application
            .primary_provider
            .published_application_commit_count(),
    }
}

#[test]
fn checked_application_rule_charges_its_exact_work_and_refuses_one_unit_short() {
    let _isolation = isolated_request_owner();
    // The zero-body control has the same schema, plan, scope and installation.
    // Its owner work is an independent baseline; the authored visit counts,
    // rather than a measured total from the paid rule, define the exact limit.
    let control = run_case(false, None, false);
    assert!(matches!(
        control.outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_eq!(control.opening, Ok(()));
    assert_eq!(control.contacts, [0, 1, 1, 0]);
    let authored_work = PREPARATION_UNITS + EVALUATION_UNITS;
    let exact_ceiling = control.work.unwrap() + authored_work;
    let exact = run_case(true, Some(exact_ceiling), false);
    assert!(
        matches!(
            exact.outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "{:?}",
        exact.outcome
    );
    assert_eq!(exact.opening, Ok(()));
    assert_eq!(exact.work, Some(exact_ceiling));
    assert_eq!(exact.contacts, [0, 1, 1, authored_work as usize]);
    assert_eq!(exact.publications, 1);
    let short = run_case(true, Some(exact_ceiling - 1), false);
    let WorthQueryApplicationCommitOutcome::Denied(denial) = short.outcome else {
        panic!(
            "one unit short must be a typed stop: {:?}, {:?}",
            short.outcome, short.opening
        );
    };
    assert!(
        matches!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                ..
            }
        ),
        "{denial:?}"
    );
    assert_eq!(short.publications, 0);
    assert_eq!(short.contacts[0], 0);
}

#[test]
fn checked_application_rule_keeps_cancellation_raised_in_preparation() {
    let _isolation = isolated_request_owner();
    let cancelled = run_case(true, None, true);
    let WorthQueryApplicationCommitOutcome::Denied(denial) = cancelled.outcome else {
        panic!(
            "cancellation inside the checked hook must reach the commit door: {:?}",
            cancelled.outcome
        );
    };
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(Control::Cancelled))
    );
    assert_eq!(cancelled.contacts, [0, 1, 0, 0]);
    assert_eq!(cancelled.publications, 0);
}

#[test]
fn undeclared_application_rule_is_refused_at_installation_before_any_rule_work() {
    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;
    use crate::domain_computation::WorthQueryProviderSessionDenialKind;

    let _isolation = isolated_request_owner();
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let contacts = Contacts::default();
    let result = world::try_install(AccountingRule {
        checked: false,
        work: true,
        cancellation: None,
        contacts: contacts.clone(),
    });
    let Err(denial) = result else {
        panic!("an unchecked rule must not publish an application runtime");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
            kind: WorthQueryProviderSessionDenialKind::ExecutionUncheckedCustomKernel {
                partition_identity: None,
            },
        }
    );
    assert_eq!(contacts.snapshot(), [0, 0, 0, 0]);
}
