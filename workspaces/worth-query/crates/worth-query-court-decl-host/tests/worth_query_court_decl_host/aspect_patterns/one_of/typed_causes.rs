//! Original application values cross installation and commit without text matching.
use super::*;
use rule::{Refusal, RefusalBinding};

fn from_installation(
    denial: &WorthQueryInMemoryApplicationDenial,
) -> WorthQueryCustomInvariantDenial {
    let WorthQueryInMemoryApplicationDenial::Publication(denial) = denial else {
        panic!("wrong seed publication phase: {denial:?}")
    };
    denial
        .custom_invariant_denial()
        .expect("original installed invariant cause")
        .clone()
}
fn assert_cause(denial: &WorthQueryCustomInvariantDenial, expected: &Refusal) {
    match (denial, expected) {
        (
            WorthQueryCustomInvariantDenial::Violation { identity, .. },
            Refusal::Membership { .. },
        ) => assert_eq!(identity.rule_id.as_str(), rule::OneOf::IDENTIFIER),
        (
            WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase,
                failure,
                ..
            },
            Refusal::Capacity { .. },
        ) => {
            assert_eq!(
                identity.semantic_identity().rule_id.as_str(),
                rule::OneOf::IDENTIFIER
            );
            assert_eq!(*phase, CustomInvariantFailurePhase::Execution);
            assert_eq!(*failure, ResultCustomInvariantFailureKind::ExecutionError);
        }
        _ => panic!("wrong typed outcome: {denial:?}"),
    }
    assert_eq!(
        denial.decode_refusal::<RefusalBinding>().unwrap(),
        *expected
    );
    // A second decode uses the original retained canonical bytes, without a rule rerun.
    assert_eq!(
        denial.decode_refusal::<RefusalBinding>().unwrap(),
        *expected
    );
}
#[test]
fn installation_retains_violation_and_evaluation_failure_causes() {
    for (seed, expected) in [
        (
            None,
            Refusal::Membership {
                subject: "record".into(),
                present: 0,
            },
        ),
        (
            Some(256),
            Refusal::Capacity {
                field: "Record.First.FirstValue".into(),
                maximum: 255,
                observed: 256,
            },
        ),
    ] {
        let denial = match world::try_install_seed(None, seed) {
            Err(denial) => denial,
            Ok(_) => panic!("damaged policy must not install"),
        };
        assert_cause(&from_installation(&denial), &expected);
    }
    let host = world::try_install_seed(None, Some(255)).unwrap();
    assert_eq!(world::read(&host).first, Some(255));
}
#[test]
fn commit_retains_the_same_violation_and_evaluation_failure_causes() {
    let host = world::install();
    for (change, expected, key) in [
        (
            authoring::Change::Zero,
            Refusal::Membership {
                subject: "record".into(),
                present: 0,
            },
            "cause-violation",
        ),
        (
            authoring::Change::OverCapacity,
            Refusal::Capacity {
                field: "Record.First.FirstValue".into(),
                maximum: 255,
                observed: 256,
            },
            "cause-failure",
        ),
    ] {
        let outcome = world::edit(&host, change, key);
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("must refuse at publication: {outcome:?}")
        };
        assert_cause(denial.custom_invariant_denial().unwrap(), &expected);
        assert_eq!(
            denial.decode_invariant_refusal::<RefusalBinding>().unwrap(),
            expected
        );
        assert_eq!(world::read(&host).first, Some(11));
    }
    assert!(matches!(
        world::edit(&host, authoring::Change::Second, "cause-lawful"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(world::read(&host).second, Some(22));
}
