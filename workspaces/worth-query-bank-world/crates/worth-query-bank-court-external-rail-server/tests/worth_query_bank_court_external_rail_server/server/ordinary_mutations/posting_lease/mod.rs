pub(crate) use worth_query_bank_testkit_server::facade::ordinary_mutations::posting_lease::*;

#[test]
fn posting_lease_keeps_the_serial_verdict_and_work() {
    const MODE: &str = "WORTH_BANK_POSTING_EQUIVALENCE";
    if let Ok(mode) = std::env::var(MODE) {
        let case = run(
            match mode.as_str() {
                "serial" => false,
                "leased" => true,
                other => panic!("invalid posting posture {other}"),
            },
            None,
            false,
        );
        println!(
            "BANK_POSTING_EQUIVALENCE:{}:{:?}",
            committed(&case),
            case.work
        );
        return;
    }
    // Separate original installations begin with the same outcome frontier.
    // A process-global outcome minted by the first run is not the same input.
    let name = format!(
        "{}::posting_lease_keeps_the_serial_verdict_and_work",
        module_path!().split_once("::").unwrap().1
    );
    let mut baseline = None;
    for mode in ["serial", "leased"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(MODE, mode)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "posting {mode}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let records = stdout
            .lines()
            .filter(|line| line.starts_with("BANK_POSTING_EQUIVALENCE:"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1, "one actual posting observation");
        if let Some(expected) = &baseline {
            assert_eq!(records[0], expected);
        } else {
            baseline = Some(records[0].to_owned());
        }
    }
}

#[test]
fn posting_lease_without_work_room_keeps_the_request_stop() {
    let stopped = run(true, Some(0), false);
    let Err(denial) = stopped.outcome else {
        panic!("{:?}", stopped.outcome)
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(
            WorthQueryAdvancementDenial::Resource(Resource::WorkExhausted)
        ) | worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(Resource::WorkExhausted)
            )
        )
    ));
    assert_eq!(stopped.balance_after, stopped.balance_before);
}

#[test]
fn posting_lease_cancelled_transfer_keeps_the_cancellation() {
    let cancelled = run(true, None, true);
    let Err(denial) = cancelled.outcome else {
        panic!("{:?}", cancelled.outcome)
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        )
    ));
}
