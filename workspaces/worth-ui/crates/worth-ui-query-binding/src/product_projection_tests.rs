use crate::WorthUiScalarProjectionSourceRecord;

#[test]
fn authored_action_changes_status_without_consuming_the_next_source_revision() {
    let owner = crate::WorthUiStatusSourceOwner::install().expect("authored program installs");
    let source = owner
        .publish_source(
            crate::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("first external source revision"),
        )
        .expect("first source edit commits");
    let source_version = source.query_receipt().inspect().basis().version();
    let action = owner
        .execute_action(
            crate::WorthUiStatusActionRequest::new(1, "Deployed", 7, 1)
                .expect("real product action request"),
        )
        .expect("action commits through the same authored program");
    let crate::WorthUiStatusActionOutcome::Committed(action) = action else {
        panic!("the current source revision must commit the authored action");
    };
    let (action, evidence) = action.into_parts();
    assert!(matches!(
        action.live_close(),
        Some(worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_))
    ));
    assert_eq!(action.value().status, "Deployed");
    assert_eq!(action.value().revision, 1);
    assert_eq!(evidence.source_revision(), 1);
    assert_eq!(evidence.status(), "Deployed");
    assert!(!evidence.query_receipt_digest().is_empty());
    assert_eq!(evidence.affected_live_view_ids().len(), 1);
    assert!(action.query_receipt().inspect().basis().version() > source_version);

    let next = owner
        .publish_source(
            crate::WorthUiScalarProjectionSourceRecord::new("SYNCHRONIZED", 2)
                .expect("second external source revision"),
        )
        .expect("the next file revision remains admissible after the action");
    assert_eq!(next.value().status, "SYNCHRONIZED");
    assert_eq!(next.value().revision, 2);
}

#[test]
fn authored_stale_action_is_denied_by_query_without_changing_status() {
    let owner = crate::WorthUiStatusSourceOwner::install().expect("authored program installs");
    owner
        .publish_source(
            crate::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("the first source revision is valid"),
        )
        .expect("the first source revision commits");
    let before = owner
        .read_status()
        .expect("read the Query source before denial");
    let action = crate::WorthUiStatusActionRequest::new(0, "Deployed", 8, 1)
        .expect("the stale action request is valid");
    let outcome = owner
        .execute_action(action)
        .expect("the stale action reaches Query through the ordinary path");
    let crate::WorthUiStatusActionOutcome::DeniedRevisionMismatch {
        active_revision,
        submitted_revision,
        ..
    } = outcome
    else {
        panic!("Query must deny the stale revision without committing");
    };
    assert_eq!(active_revision, 1);
    assert_eq!(submitted_revision, 0);
    let after = owner
        .read_status()
        .expect("read the unchanged Query source");
    assert_eq!(after.value(), before.value());
    assert_eq!(
        after.query_receipt().inspect().basis().version(),
        before.query_receipt().inspect().basis().version()
    );
}

#[test]
fn authored_status_source_updates_the_real_application_root() {
    let owner = crate::WorthUiStatusSourceOwner::install()
        .expect("the authored UI source and query install together");
    let initial = owner.read_status().expect("the pending source is readable");
    assert_eq!(initial.value().identity, "platform.pulse.status");
    assert_eq!(initial.value().status, "PENDING");
    assert_eq!(initial.value().revision, 0);
    let (registration, pending) = owner
        .initial_projection()
        .expect("the UI registration and pending observation come from Query");
    let crate::UiProjectionObservation::ApplicationScalar(pending) = pending else {
        panic!("the authored status root must issue an application scalar observation");
    };
    assert!(registration.admits(pending.fact()));

    let online = owner
        .publish_source(WorthUiScalarProjectionSourceRecord::new("ONLINE", 1).unwrap())
        .expect("the first source edit publishes through Query");
    assert!(matches!(
        online.live_close(),
        Some(worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_))
    ));
    assert_eq!(online.value().status, "ONLINE");
    assert_eq!(online.value().revision, 1);
    assert_eq!(online.query_receipt().inspect().result_count(), 1);
    assert!(online
        .query_receipt()
        .inspect()
        .terminal_resources_released());
    let online_version = online.query_receipt().inspect().basis().version();
    let current = online
        .into_projection_observation()
        .expect("Query result has an admitted UI identity");
    let crate::UiProjectionObservation::ApplicationScalar(current) = current else {
        panic!("the first accepted edit must carry the authored scalar fact");
    };
    assert!(registration.admits(current.fact()));

    let updated = owner
        .publish_source(WorthUiScalarProjectionSourceRecord::new("UPDATED-LONG", 2).unwrap())
        .expect("the second source edit replaces the same Query record");
    assert_eq!(updated.value().status, "UPDATED-LONG");
    assert_eq!(updated.value().revision, 2);
    assert!(updated.query_receipt().inspect().basis().version() > online_version);
    let close = owner.close();
    assert!(close.owner_terminal());
    assert_eq!(close.remaining_live_consumers(), 0);
}
