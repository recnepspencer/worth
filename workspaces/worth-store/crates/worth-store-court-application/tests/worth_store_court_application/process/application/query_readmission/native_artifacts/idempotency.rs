//! Native administrative binding adversaries do not manufacture Query receipts.
use super::*;
use worth_foundational::facade::AspectValue;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;

pub(super) fn intent() -> String {
    format!("v2:{}:source=-:operation=-:scope=-:precondition=-:input=-:proposal=-:conditional-definition=-", "01".repeat(32))
}
pub(super) fn row(
    native: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    client: &str,
    key: &str,
    intent: &str,
    outcome: u64,
) {
    administration::create_fields(
        native,
        execution,
        "worth-query-provider-idempotency",
        client,
        &[
            (
                "idempotency",
                "key",
                AspectValue::String(key.to_owned().into()),
            ),
            (
                "idempotency",
                "intent",
                AspectValue::String(intent.to_owned().into()),
            ),
            (
                "idempotency",
                "outcome-identity",
                AspectValue::UInt64(outcome),
            ),
            (
                "idempotency",
                "emitted-effect-count",
                AspectValue::UInt64(0),
            ),
            // Match the current seven-field Native registration contract
            // (provider_idempotency revision 4). This administrative row has
            // no World product affinity or output-settlement occurrence.
            (
                "idempotency",
                "product-affinity",
                AspectValue::String("memory-only".into()),
            ),
            (
                "idempotency",
                "attempt-association-source",
                AspectValue::UInt64(1),
            ),
            ("idempotency", "output-settlement", AspectValue::UInt64(0)),
        ],
    );
}

#[test]
fn actual_binding_rows_refuse_malformed_keys_intents_duplicate_keys_and_outcomes() {
    for (case, expected) in [
        ("key", "Home idempotency key is not canonical"),
        ("root", "provider idempotency intent is malformed"),
        ("optional", "provider idempotency intent is malformed"),
        ("unknown", "provider idempotency intent is malformed"),
        ("duplicate-key", "Home idempotency contains duplicate keys"),
        (
            "duplicate-outcome",
            "Home idempotency contains duplicate outcome identities",
        ),
        ("zero", "Home idempotency outcome identity is invalid"),
        ("max", "Home idempotency outcome identity is invalid"),
    ] {
        let path = tempfile::tempdir().unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = ExecutionRequest::leased(&lease);
        fresh_closed(path.path(), execution);
        with_bootstrap_advancement_from_request(execution, |phase| {
            with_native(path.path(), &phase, |native| {
                let key = if case == "key" {
                    "not-canonical".into()
                } else {
                    "01".repeat(32)
                };
                let mut intent = intent();
                match case {
                    "root" => intent = intent.replacen(&"01".repeat(32), "-", 1),
                    "optional" => intent.push_str(":mutation-binding=-"),
                    "unknown" => intent.push_str(&format!(":not-a-slot={}", "02".repeat(32))),
                    _ => (),
                }
                let outcome = match case {
                    "zero" => 0,
                    "max" => u64::MAX,
                    _ => 31,
                };
                row(native, &execution, "first-binding", &key, &intent, outcome);
                if case == "duplicate-key" {
                    row(native, &execution, "second-binding", &key, &intent, 32);
                }
                if case == "duplicate-outcome" {
                    row(
                        native,
                        &execution,
                        "second-binding",
                        &"02".repeat(32),
                        &intent,
                        31,
                    );
                }
            });
        })
        .unwrap();
        let refused = reopen_refusal(path.path(), execution);
        match &refused {
            WorthQueryApplicationHomeOpenRefused::ExistingNativeFinalization { query, .. } => {
                assert_eq!(query.reason().kind(), Kind::CheckpointRecoveryRejected);
                assert_eq!(query.reason().subject(), expected, "{case}");
            }
            other => panic!("{case} must refuse the actual binding inventory first: {other:?}"),
        }
        refused
            .close(&execution)
            .unwrap_or_else(|refused| panic!("owning idempotency refusal close: {refused:?}"));
    }
}

#[test]
fn max_minus_one_recovery_leaves_the_first_new_outcome_mint_exhausted() {
    // The real process-global frontier must not poison unrelated court cases.
    const CHILD: &str = "WORTH_QUERY_HOME_OUTCOME_EXHAUSTION_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = format!(
            "{}::max_minus_one_recovery_leaves_the_first_new_outcome_mint_exhausted",
            module_path!().split_once("::").unwrap().1
        );
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationCommitDenialKind as Denial,
        WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted,
    };
    let path = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(path.path(), execution);
    let before = with_bootstrap_advancement_from_request(execution, |phase| {
        with_native(path.path(), &phase, |native| {
            row(
                native,
                &execution,
                "last-admissible-outcome",
                &"03".repeat(32),
                &intent(),
                u64::MAX - 1,
            );
            let rows = administration::rows(native, &execution, "worth-query-provider-idempotency");
            let [row] = rows.as_slice() else {
                panic!("one actual durable frontier row")
            };
            assert_eq!(
                administration::field(row, "idempotency", "outcome-identity"),
                &AspectValue::UInt64(u64::MAX - 1)
            );
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .unwrap();
            native
                .history()
                .branch_head_for_observation(&basis.observation())
                .unwrap()
                .unwrap()
        })
    })
    .unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let request = fixture::request_with_initializer(
        path.path().to_path_buf(),
        build(),
        contacts.clone(),
        |_, _, _| panic!("frontier recovery cannot initialize"),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual MAX-1 Home admission: {:.1800}", refused.detail()));
    assert_eq!(balance(home.runtime()), 7);
    match set(home.runtime(), 8) {
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => {
            assert_eq!(denial.kind(), Denial::ProviderRejected);
            assert_eq!(denial.stage(), Stage::ProposalBinding);
        }
        other => panic!("the actual restored outcome frontier cannot mint: {other:?}"),
    }
    assert_eq!(balance(home.runtime()), 7);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("actual exhausted Home close: {refused:?}"));
    let after = with_bootstrap_advancement_from_request(execution, |phase| {
        with_native(path.path(), &phase, |native| {
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .unwrap();
            native
                .history()
                .branch_head_for_observation(&basis.observation())
                .unwrap()
                .unwrap()
        })
    })
    .unwrap();
    assert_eq!(
        after, before,
        "exhaustion published no Native business commit"
    );
}
