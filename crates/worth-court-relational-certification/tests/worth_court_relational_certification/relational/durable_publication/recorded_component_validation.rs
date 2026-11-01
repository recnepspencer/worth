//! Original validation belongs to the source commit, not its reconstructive execution.
use super::*;
use std::sync::{Arc, Mutex};
use worth_testkit_relational::durable_publication::{initial_schema::installed_runtime, schema};

#[test]
fn recorded_component_recovery_preserves_original_validation_in_historical_and_current_roots() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for checkpoint in [false, true] {
        let medium = TestDurableMedium::new();
        let source_calls = Arc::new(Mutex::new(Vec::new()));
        let (source, _) = installed_runtime(&medium, schema(), source_calls.clone());
        create_entity(execution, &source, "original-activation");
        create_entity(execution, &source, "original-seed");
        let third = create_entity_outcome(execution, &source, "original-genesis");
        let historical = current(execution, &source);
        assert!(!third.semantic_result().validation().is_empty());
        assert!(source_calls
            .lock()
            .unwrap()
            .iter()
            .any(|(version, _)| { *version == third.commit.version_id }));
        release_test_commit_snapshot(&source, &third);
        drop(third);
        if checkpoint {
            medium.file_checkpoint(capture(execution, &source));
        }
        let fourth = create_entity_outcome(execution, &source, "later-writer");
        let latest = current(execution, &source);
        assert!(source_calls
            .lock()
            .unwrap()
            .iter()
            .any(|(version, _)| { *version == fourth.commit.version_id }));
        release_test_commit_snapshot(&source, &fourth);
        drop(fourth);
        let original_calls = source_calls.lock().unwrap().clone();
        let original_offers = medium.offered().len();
        drop(source);

        let target_calls = Arc::new(Mutex::new(Vec::new()));
        let (mut runtime, _) = installed_runtime(&medium, schema(), target_calls.clone());
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                medium.recovery_source(),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let (_, authority) = runtime
            .durability_recovery()
            .recover_with_authority(execution, plan)
            .unwrap();
        let port = runtime.owner_component_services().basis_port();
        for recorded in [&historical, &latest] {
            let basis = authority
                .admit_recorded_component(execution, &runtime, recorded)
                .unwrap_or_else(|refusal| {
                    panic!(
                        "checkpoint={checkpoint}, recorded={:?}: {:?}",
                        recorded.target(),
                        refusal.cause()
                    )
                })
                .into_basis();
            assert_recorded_basis(&runtime, &basis, recorded);
            assert_eq!(
                port.export_recorded_retained_component(execution, &basis)
                    .unwrap(),
                *recorded,
                "the original full artifact digest includes its genuine validation image"
            );
            if recorded == &latest {
                assert!(port.compare_current_exact(&basis).is_ok());
            } else {
                assert!(port.compare_current_exact(&basis).is_err());
            }
        }
        assert!(
            target_calls.lock().unwrap().is_empty(),
            "recovery and root admission never execute the installed callback"
        );
        assert_eq!(*source_calls.lock().unwrap(), original_calls);
        assert_eq!(
            medium.offered().len(),
            original_offers,
            "no Native effects are republished"
        );
    }
}
