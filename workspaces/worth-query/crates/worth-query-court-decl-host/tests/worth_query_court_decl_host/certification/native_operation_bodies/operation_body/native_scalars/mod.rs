pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::native_scalars::*;

fn day(days_from_unix_epoch: i32) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch,
    }
}

fn nine_am() -> CanonicalTime {
    CanonicalTime::new(9 * 3_600 * 1_000_000_000).unwrap()
}

fn schedule(due: Option<CanonicalDate>, rest: bool) -> ScheduleInput<CanonicalDate> {
    ScheduleInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        due,
        opens: rest.then(nine_am),
        token: rest.then_some(*b"0123456789abcdef"),
        payload: rest.then(|| InlineBytes::try_new(vec![0, 1, 127, 128, 255]).unwrap()),
    }
}

fn row(input: &ScheduleInput<CanonicalDate>) -> ScheduleRow {
    ScheduleRow {
        due: input.due,
        opens: input.opens,
        token: input.token,
        payload: input.payload.clone(),
    }
}

fn denied_member(outcome: Outcome<WorthQueryOperationBodyDenial, ()>) -> String {
    match outcome {
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch { member }) => member,
        other => panic!("expected a typed input refusal: {other:?}"),
    }
}

#[test]
fn optional_native_date_initializes_updates_reads_back_and_clears() {
    let app = install::<MODE>();
    let absent = row(&schedule(None, false));
    assert_eq!(read(&app), absent);

    let initial = schedule(Some(day(20_413)), true);
    let first = assign(&app, 400, initial.clone());
    assert!(matches!(first, Outcome::Committed { .. }), "{first:?}");
    assert_eq!(read(&app), row(&initial));
    assert!(matches!(
        assign(&app, 400, initial.clone()),
        Outcome::RecoveredCommitted(_)
    ));

    // Replacing only the date keeps the sibling carriers byte for byte.
    let earlier = schedule(Some(day(-719_468)), true);
    assert!(matches!(
        assign(&app, 401, earlier.clone()),
        Outcome::Committed { .. }
    ));
    assert_eq!(read(&app), row(&earlier));

    let cleared = schedule(None, false);
    assert!(matches!(
        assign(&app, 402, cleared),
        Outcome::Committed { .. }
    ));
    assert_eq!(read(&app), absent);
}

#[test]
fn forged_dates_and_non_canonical_times_refuse_before_any_effect() {
    let app = install::<MODE>();
    let baseline = schedule(Some(day(5)), true);
    assert!(matches!(
        assign(&app, 410, baseline.clone()),
        Outcome::Committed { .. }
    ));
    let before = read(&app);
    assert_eq!(before, row(&baseline));

    let forged = |due| ScheduleInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        due: Some(ForgedDay(due)),
        opens: None,
        token: None,
        payload: None,
    };
    assert_eq!(
        denied_member(assign_forged(&app, 411, forged(1 << 40))),
        "due"
    );
    assert_eq!(read(&app), before);

    let mut late = baseline.clone();
    late.opens = Some(CanonicalTime {
        nanos_since_midnight: CanonicalTime::NANOS_PER_DAY,
    });
    late.due = Some(day(6));
    assert_eq!(denied_member(assign(&app, 412, late)), "opens");
    assert_eq!(read(&app), before, "no sibling assignment published");

    // The refused keys recorded nothing: lawful retries commit fresh effects.
    assert!(matches!(
        assign_forged(&app, 411, forged(7)),
        Outcome::Committed { .. }
    ));
    assert_eq!(read(&app).due, Some(day(7)));
    let lawful = schedule(Some(day(8)), true);
    assert!(matches!(
        assign(&app, 412, lawful.clone()),
        Outcome::Committed { .. }
    ));
    assert_eq!(read(&app), row(&lawful));
}
