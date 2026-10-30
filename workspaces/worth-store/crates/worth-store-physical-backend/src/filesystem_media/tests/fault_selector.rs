use super::super::*;

#[test]
fn identified_operation_selector_rejects_unbound_recovery_helpers() {
    let rule = MediaFaultRule::for_certification(
        MediaOperationRole::SynchronizeFileState,
        3,
        MediaFaultDirective::FailBarrier {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    )
    .for_identified_operation();
    let binding = MediaOperationIdentityBinding {
        owner: None,
        runtime_incarnation: None,
        store: None,
    };
    let recovery_helper = MediaOperationContext::new(
        binding,
        MediaOperationRole::SynchronizeFileState,
        0,
        MediaOperationCoordinates::unbound(),
        3,
        None,
    );
    assert!(!rule.matches(recovery_helper));

    let record_artifact = MediaOperationContext::new(
        binding,
        MediaOperationRole::SynchronizeFileState,
        0,
        MediaOperationCoordinates::for_path(
            MediaOperationIdentity::for_test(17),
            MediaPathRole::ArtifactOwned,
            None,
        ),
        3,
        Some(1),
    );
    assert!(rule.matches(record_artifact));
}

#[test]
fn identified_operation_ordinal_ignores_unbound_raw_ordinal_drift() {
    let rule = MediaFaultRule::for_certification(
        MediaOperationRole::SynchronizeFileState,
        2,
        MediaFaultDirective::FailBarrier {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    )
    .for_identified_operation_ordinal();
    let binding = MediaOperationIdentityBinding {
        owner: None,
        runtime_incarnation: None,
        store: None,
    };
    let coordinates = |operation| {
        MediaOperationCoordinates::for_path(
            MediaOperationIdentity::for_test(operation),
            MediaPathRole::ArtifactOwned,
            None,
        )
    };
    let first_identified = MediaOperationContext::new(
        binding,
        MediaOperationRole::SynchronizeFileState,
        0,
        coordinates(17),
        2,
        Some(1),
    );
    let second_identified = MediaOperationContext::new(
        binding,
        MediaOperationRole::SynchronizeFileState,
        0,
        coordinates(18),
        9,
        Some(2),
    );
    assert!(!rule.matches(first_identified));
    assert!(rule.matches(second_identified));
}

#[test]
fn activated_selector_consumes_exactly_one_later_identified_operation() {
    let activation = CertificationMediaFaultActivation::for_certification();
    let rule = MediaFaultRule::for_certification(
        MediaOperationRole::PositionedRead,
        1,
        MediaFaultDirective::PauseBefore(MediaPauseGate::for_certification()),
    )
    .for_next_identified_operation_after_activation(activation.clone());
    let binding = MediaOperationIdentityBinding {
        owner: None,
        runtime_incarnation: None,
        store: None,
    };
    let context = |operation, ordinal| {
        MediaOperationContext::new(
            binding,
            MediaOperationRole::PositionedRead,
            8,
            MediaOperationCoordinates::for_path(
                MediaOperationIdentity::for_test(operation),
                MediaPathRole::ArtifactOwned,
                None,
            )
            .at_offset(0),
            ordinal,
            Some(ordinal),
        )
    };

    assert!(!rule.matches(context(17, 1)));
    activation.arm().unwrap();
    assert!(rule.matches(context(18, 2)));
    assert!(activation.is_consumed());
    assert!(!rule.matches(context(19, 3)));
    assert_eq!(
        activation.arm(),
        Err(MediaFaultActivationDenial::AlreadyConsumed)
    );
}

#[test]
fn activated_selector_can_choose_the_second_matching_identified_operation() {
    let activation = CertificationMediaFaultActivation::for_certification();
    let rule = MediaFaultRule::for_certification(
        MediaOperationRole::PositionedRead,
        1,
        MediaFaultDirective::PauseBefore(MediaPauseGate::for_certification()),
    )
    .for_nth_identified_operation_after_activation(
        activation.clone(),
        std::num::NonZeroU64::new(2).unwrap(),
    );

    assert!(!rule.matches(identified_read_context(17)));
    activation.arm().unwrap();
    assert!(!rule.matches(identified_read_context(18)));
    assert!(!activation.is_consumed());
    assert!(rule.matches(identified_read_context(19)));
    assert!(activation.is_consumed());
    assert!(!rule.matches(identified_read_context(20)));
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: activated selector is one shot under concurrent matching operations; source: worth-store-physical-backend::filesystem_media::tests::fault_selector::activated_selector_is_one_shot_under_concurrent_matching_operations; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=1, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn activated_selector_is_one_shot_under_concurrent_matching_operations() {
    let activation = CertificationMediaFaultActivation::for_certification();
    let rule = std::sync::Arc::new(
        MediaFaultRule::for_certification(
            MediaOperationRole::PositionedRead,
            1,
            MediaFaultDirective::PauseBefore(MediaPauseGate::for_certification()),
        )
        .for_next_identified_operation_after_activation(activation.clone()),
    );
    let ready = std::sync::Arc::new(std::sync::Barrier::new(3));
    activation.arm().unwrap();
    let matches = std::thread::scope(|scope| {
        let attempts = [41_u64, 42_u64].map(|operation| {
            let rule = std::sync::Arc::clone(&rule);
            let ready = std::sync::Arc::clone(&ready);
            scope.spawn(move || {
                ready.wait();
                rule.matches(identified_read_context(operation))
            })
        });
        ready.wait();
        attempts.map(|attempt| attempt.join().unwrap())
    });

    assert_eq!(matches.into_iter().filter(|matched| *matched).count(), 1);
    assert!(activation.is_consumed());
}

fn identified_read_context(operation: u64) -> MediaOperationContext {
    MediaOperationContext::new(
        MediaOperationIdentityBinding {
            owner: None,
            runtime_incarnation: None,
            store: None,
        },
        MediaOperationRole::PositionedRead,
        8,
        MediaOperationCoordinates::for_path(
            MediaOperationIdentity::for_test(operation),
            MediaPathRole::ArtifactOwned,
            None,
        )
        .at_offset(0),
        operation,
        Some(operation),
    )
}
