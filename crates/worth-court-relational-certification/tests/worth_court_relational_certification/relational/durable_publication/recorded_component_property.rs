//! C1.3: recovery readmits exactly the recorded component and refuses any
//! single changed commitment without grants or head substitution. The domain
//! is small, so every (component, commitment) pair runs.
use super::*;

/// Each recorded commitment a readmission must match, as a serialized path.
const COMMITMENTS: [&str; 10] = [
    "/descriptor/branch_id",
    "/descriptor/reference",
    "/descriptor/truth_version",
    "/descriptor/schema_commitment",
    "/descriptor/visibility_commitment",
    "/target/Canonical/receipt/commit_id",
    "/target/Canonical/position",
    "/target/Canonical/artifact_digest",
    "/target/Canonical/roots/truth_root",
    "/target/Canonical/roots/schema_root",
];

#[test]
fn recovery_readmits_only_the_exact_recorded_component() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let mut recorded = vec![current(execution, &runtime)];
    for name in ["predecessor", "successor", "later-head"] {
        create_entity(execution, &runtime, name);
        recorded.push(current(execution, &runtime));
    }
    drop(runtime);
    let mut recovered = durable_runtime(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    for (component, original) in recorded.iter().enumerate() {
        let basis = authority
            .admit_recorded_component(execution, &recovered, original)
            .expect("the exact recorded component readmits")
            .into_basis();
        assert_recorded_basis(&recovered, &basis, original);
        assert_eq!(
            &recovered
                .owner_component_services()
                .basis_port()
                .export_recorded_retained_component(execution, &basis)
                .unwrap(),
            original,
            "readmission re-exports the component captured before the crash"
        );
        drop(basis);
        for path in COMMITMENTS {
            // An Empty target records no Canonical commitments.
            let Some(changed) = change_one(original, path) else {
                continue;
            };
            let refused = authority
                .admit_recorded_component(execution, &recovered, &changed)
                .err()
                .unwrap_or_else(|| panic!("{path} changed, yet component {component} readmitted"));
            assert!(
                !refused.retains_acquired_custody(),
                "a refusal returns no grant: {:?}",
                refused.cause()
            );
        }
    }
}

/// Change exactly one recorded value; `None` when the path is not recorded.
fn change_one(recorded: &Recorded, path: &str) -> Option<Recorded> {
    let mut value = serde_json::to_value(recorded).unwrap();
    let field = value.pointer_mut(path)?;
    change(field);
    let changed: Recorded = serde_json::from_value(value).unwrap();
    assert_ne!(
        &changed, recorded,
        "{path} must change the recorded meaning"
    );
    Some(changed)
}

fn change(value: &mut serde_json::Value) {
    use serde_json::Value;
    match value {
        Value::Number(number) => *value = Value::from(number.as_u64().unwrap() + 1),
        // A digest stays well formed: change its first hex character.
        Value::String(text) if text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit()) => {
            let first = if text.starts_with('0') { "1" } else { "0" };
            text.replace_range(0..1, first);
        }
        Value::String(text) => text.push_str("-other"),
        Value::Array(items) => change(&mut items[0]),
        Value::Object(fields) => change(fields.values_mut().next().unwrap()),
        Value::Bool(flag) => *flag = !*flag,
        Value::Null => panic!("a recorded commitment is never null"),
    }
}
