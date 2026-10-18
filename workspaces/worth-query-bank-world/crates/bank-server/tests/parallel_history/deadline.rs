//! Dispatch-time deadlines run at every posture through the real Bank doors.
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial as Admission,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::WorthQueryApplicationOutputDemandDenial as DemandDenial,
    primary_graph::{
        advancement_observations_on_this_thread_for_test as observations,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionObservation, WorthQueryExecutionPlacementForTest as Placement,
        WorthQueryOutputDemandDenialKind as Kind,
    },
};

fn zero_bill() {
    let records = observations();
    assert_eq!(
        records.len(),
        1,
        "one deadline-stopped Bank advancement is observed"
    );
    assert_eq!(
        (
            records[0].report.charged_work(),
            records[0].report.charged_span()
        ),
        (0, 0),
        "a present Bank deadline dispatches and charges no work"
    );
    assert_eq!(
        records[0].applied.len(),
        0,
        "a present Bank deadline applies no commit or publication"
    );
    assert_eq!(
        records[0].workers, None,
        "deadline refusal precedes worker dispatch"
    );
}

pub(super) fn assessment(
    width: usize,
    action: impl FnOnce() -> Result<
        bank_server::BankApprovedPaymentAssessmentProgress,
        bank_server::BankApprovedPaymentWorkflowError,
    >,
) {
    observations();
    let Err(bank_server::BankApprovedPaymentWorkflowError::AssessmentDemand(cause)) = action()
    else {
        panic!("expired assessment keeps the demand's typed refusal")
    };
    let DemandDenial::Demand(cause) = *cause else {
        panic!("expired assessment keeps its advancement refusal")
    };
    assert_eq!(
        cause.kind(),
        Kind::ExecutionRequest(Admission::Interrupted(Interruption::DeadlineExceeded)),
        "assessment deadline kind is asserted by value"
    );
    zero_bill();
    println!("BANK_DEADLINE:assessment:width={width}:work=0:span=0:commits=0:DeadlineExceeded");
}

#[test]
fn present_deadlines_run_in_both_bank_histories() {
    let _owned = WorthQueryExecutionObservation::install_on_this_thread();
    super::unavailable::rows();
    let world = super::seeded_world::World::new(super::seeded_world::SEED);
    let actor = world.authenticate(1);
    let mut run = 0;
    for width in [0, 1, 2, 5] {
        let previous =
            place(std::num::NonZeroUsize::new(width).map_or(Placement::World, Placement::Leased));
        let cancellation = WorthQueryCancellationSource::new();
        let expired =
            WorthQueryRequestScope::new(Some(std::time::Instant::now()), cancellation.token());
        observations();
        let outcome = world
            .world
            .runtime
            .mutate(bank_server::mutations::send_money(
                bank_domain::schema::SendMoney {
                    from: world.accounts[0],
                    recipient: super::seeded_world::principal(2),
                    amount: bank_domain::model::Money::from_minor(1).unwrap(),
                },
            ))
            .as_principal(&actor)
            .controls(bank_server::BankMutationControls::new(
                WorthQueryRequestScope::new(None, cancellation.token()),
                super::seeded_world::key(900),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("deadline construction: {cause:?}"));
        // Capture is admitted work; only the fresh stopped advance is billed below.
        observations();
        let request = world.world.runtime.request(&actor, &expired);
        let outcome = outcome.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap());
        assert!(
            matches!(
                outcome,
                worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(Admission::Interrupted(Interruption::DeadlineExceeded)), .. }
            ),
            "transfer deadline kind is asserted by value"
        );
        zero_bill();
        println!("BANK_DEADLINE:transfer:width={width}:work=0:span=0:commits=0:DeadlineExceeded");
        place(previous);
        run += 1;
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "parallel_history::assessment_history::assessment_history_settles_at_each_request_width", "--nocapture"])
            .env("WORTH_BANK_ASSESSMENT_WIDTH", width.to_string()).env("WORTH_BANK_PRESENT_DEADLINE", "1").output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "assessment deadline width {width}: {stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            stdout
                .lines()
                .filter(|line| line.starts_with("BANK_DEADLINE:assessment:"))
                .count(),
            1,
            "one actual assessment deadline row per posture"
        );
        println!(
            "BANK_DEADLINE_ROW:assessment:posture={width}:{}",
            stdout
                .lines()
                .find(|line| line.starts_with("BANK_DEADLINE:assessment:"))
                .unwrap()
        );
        run += 1;
    }
    assert_eq!(
        run, 8,
        "both real histories execute their deadline at all four postures"
    );
}
