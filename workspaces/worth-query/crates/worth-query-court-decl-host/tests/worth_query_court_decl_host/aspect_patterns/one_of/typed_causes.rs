//! Original application values cross installation and commit without text matching.
use super::*;
use rule::{Refusal, RefusalBinding};

fn from_installation(
    denial: &WorthQueryApplicationPreparationDenial,
) -> WorthQueryCustomInvariantDenial {
    let WorthQueryApplicationPreparationDenial::Publication(denial) = denial else {
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
fn installation_refuses_missing_required_case_and_preserves_custom_capacity_cause() {
    use worth_relational::facade::transactions::{
        ConflictClass, InvariantViolationFields, TransactionCommitError,
    };

    let Err(WorthQueryApplicationPreparationDenial::Publication(missing)) =
        world::try_install_seed(None, None)
    else {
        panic!("missing required case must fail publication");
    };
    let Some(TransactionCommitError::Conflict { error, .. }) = missing.native_commit_denial()
    else {
        panic!("expected the original Native conflict: {missing:?}");
    };
    assert!(matches!(
        &error.class,
        ConflictClass::InvariantViolation {
            fields: InvariantViolationFields::EntityAspectExclusion {
                required: true,
                active_count: 0,
                ..
            },
            ..
        }
    ));
    let denial = match world::try_install_seed(None, Some(256)) {
        Err(denial) => denial,
        Ok(_) => panic!("custom capacity rule must refuse the seed"),
    };
    assert_cause(&from_installation(&denial), &capacity_refusal());
    let host = world::try_install_seed(None, Some(255)).unwrap();
    assert_eq!(world::read(&host).first, Some(255));
}

#[test]
fn commit_retains_custom_capacity_failure_cause() {
    let host = world::install();
    let outcome = world::edit(&host, authoring::Change::OverCapacity, "cause-failure");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("must refuse at publication: {outcome:?}");
    };
    let expected = capacity_refusal();
    assert_cause(denial.custom_invariant_denial().unwrap(), &expected);
    assert_eq!(
        denial.decode_invariant_refusal::<RefusalBinding>().unwrap(),
        expected
    );
    assert_eq!(world::read(&host).first, Some(11));
}

fn capacity_refusal() -> Refusal {
    Refusal::Capacity {
        field: "Record.First.FirstValue".into(),
        maximum: 255,
        observed: 256,
    }
}
