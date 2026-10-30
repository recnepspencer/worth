pub(crate) use worth_query_bank_testkit_server::facade::parallel_history::deadline::*;

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
        let outcome = outcome.advance(&request, Some(std::num::NonZeroUsize::new(20_000).unwrap()));
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
            .args(["--exact", "oracle::server_owned_effects::server::parallel_history::assessment_history::assessment_history_settles_at_each_request_width", "--nocapture"])
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
