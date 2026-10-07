//! Refused constructor adapters preserve native repair and successor custody.
use super::checkpoint_transition::*;
use super::*;
use application_installation::{
    WorthQueryApplicationOpenDenial as Denial, WorthQueryHomeOpening as Opening,
    WorthQueryOpenAdoptionResources as Resources, WorthQueryRefusedHome as RefusedHome,
};

#[test]
fn open_adoption_deferred_settlement_holds_the_home_in_repair_without_rerunning_authoring() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let refusal = adopt_through_adapter(
        source,
        configuration(),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            calls += 1;
            writer.bind_entity(record("repaired-record", 89))?;
            writer.fail_next_durable_append_for_test();
            Ok(())
        },
    )
    .err()
    .expect("performed but unacknowledged adoption must expose no World");
    assert!(matches!(refusal.denial, Denial::AdoptionDeferred(_)));
    let pending = match refusal.home {
        RefusedHome::InRepair(pending) => pending,
        other => panic!("expected exact native repair custody, got {other:?}"),
    };
    pending.fail_next_durable_append_for_test();
    let pending = pending
        .repair()
        .expect_err("a refused repair retains the same capsule");
    let home = pending
        .repair()
        .expect("native repair acknowledges the existing performed adoption");
    let reopened = reopen(home).expect("only the repaired home may create a World");
    assert_eq!(
        reopened.opening(),
        &Opening::Resumed {
            installed: *validated_program().revision()
        }
    );
    assert_record_resolves(&reopened, 89);
    assert_eq!(calls, 1, "repair must not rerun typed adoption authoring");
}

#[test]
fn open_adoption_returns_the_successor_home_after_a_later_installation_denial() {
    let (source, predecessor) = source();
    let (mut incomplete,) = configuration();
    incomplete.install_route = false;
    let refusal = adopt_through_adapter(
        source,
        (incomplete,),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| writer.bind_entity(record("acknowledged-record", 101)),
    )
    .err()
    .expect("incomplete conditional routes must refuse the open after native acknowledgment");
    assert!(matches!(refusal.denial, Denial::ConditionalPublication(_)));
    let home = match refusal.home {
        RefusedHome::Successor(home) => home,
        other => panic!("acknowledged adoption effects must return the successor home: {other:?}"),
    };
    let reopened =
        reopen(home).expect("a corrected configuration resumes the actual acknowledged successor");
    assert_record_resolves(&reopened, 101);
}

#[test]
fn a_stopped_successor_capture_repairs_without_repeating_adoption() {
    use application_installation::{
        WorthQueryApplicationOpenDenial as Denial, WorthQueryOpenAdoption as Adoption,
        WorthQueryOpenAdoptionResources as Resources, WorthQueryRefusedHome as RefusedHome,
    };
    let (source, predecessor) = source();
    let mut calls = 0;
    let refusal = application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .adopt_on_open(
        Adoption::new(
            predecessor,
            Resources::bounded(512, 32, 4096).unwrap(),
            |writer, _| {
                calls += 1;
                writer.bind_entity(record("capture-repaired", 137))
            },
        )
        .fail_next_checkpoint_capture_for_test(),
    )
    .open(home_of(&source))
    .err()
    .expect("a stopped capture retains acknowledged repair custody");
    assert!(matches!(refusal.denial, Denial::AdoptionCaptureStopped(_)));
    let pending = match refusal.home {
        RefusedHome::InRepair(pending) => pending,
        other => panic!("capture stop must retain repair custody: {other:?}"),
    };
    let reopened = reopen(
        pending
            .repair()
            .expect("capture repair returns the acknowledged successor"),
    )
    .expect("the repaired successor resumes");
    assert_record_resolves(&reopened, 137);
    assert_eq!(calls, 1, "capture repair must not repeat authoring");
}
