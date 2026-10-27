//! Original controls, real before-due abort, and due retry keep exact custody.
use super::*;

pub(super) fn run() {
    let host = published();
    let cancelled = WorthQueryCancellationSource::new();
    let original = WorthQueryRequestScope::new(None, cancelled.token());
    let actor = host::authenticate(&host, 1, &original);
    cancelled.cancel();
    let before = host.contacts.resource.counts();
    let (none, cause) = host
        .application
        .runtime()
        .request(&actor, &original)
        .resources()
        .start::<Resource>(50)
        .err()
        .expect("cancelled before Start")
        .into_parts();
    assert!(none.is_none());
    assert_eq!(cause.kind(), WorthQueryResourceStopKind::Request);
    assert_eq!(host.contacts.resource.counts(), before);

    let after_contact = WorthQueryCancellationSource::new();
    let original = WorthQueryRequestScope::new(None, after_contact.token());
    let actor = host::authenticate(&host, 1, &original);
    *host.contacts.resource.cancel_on_start.lock().unwrap() = Some(after_contact);
    host.contacts.resource_clock.milliseconds(10);
    let (same, cause) = host
        .application
        .runtime()
        .request(&actor, &original)
        .resources()
        .start::<Resource>(51)
        .err()
        .expect("provider contacted, original then cancels")
        .into_parts();
    assert_eq!(cause.kind(), WorthQueryResourceStopKind::Request);
    let mut same = same.expect("actual Native/provider cleanup remains in the same issued row");
    assert_eq!(host.contacts.resource.counts(), (1, 0, 0));
    host.contacts.resource.send(0, 5151).unwrap();
    let fresh = request_scope();
    let actor = host::authenticate(&host, 1, &fresh);
    let request = host.application.runtime().request(&actor, &fresh);
    let resources = request.resources();
    resources.advance(&mut same).unwrap();
    assert_eq!(
        resources
            .with_result_governed(
                &same.receipt(),
                metadata::admit(&host, &actor, &fresh).unwrap(),
                |value| *value
            )
            .unwrap(),
        Some(5151)
    );
    resources.close(&mut same).unwrap();

    let mut retrying = start(&resources, 52);
    host.contacts.resource_clock.milliseconds(15);
    assert_eq!(
        resources.advance(&mut retrying).unwrap_err().kind(),
        WorthQueryResourceStopKind::Cleanup,
        "the actual before-due temporal handoff remains deferred"
    );
    assert_eq!(
        host.contacts.resource.counts().0,
        2,
        "retry is genuinely before due"
    );
    assert_eq!(
        resources
            .state_governed(
                &retrying.receipt(),
                metadata::admit(&host, &actor, &fresh).unwrap()
            )
            .unwrap(),
        WorthQueryResourceState::RetryPending
    );
    assert!(resources.close(&mut retrying).unwrap().detail().is_none());
    assert!(host.contacts.resource.send(1, 999).is_err());
    let mut successor = start(&resources, 53);
    host.contacts.resource_clock.milliseconds(20);
    assert_eq!(
        resources.advance(&mut successor).unwrap_err().kind(),
        WorthQueryResourceStopKind::Cleanup
    );
    assert_eq!(
        resources
            .state_governed(
                &successor.receipt(),
                metadata::admit(&host, &actor, &fresh).unwrap()
            )
            .unwrap(),
        WorthQueryResourceState::RetryPending
    );
    host.contacts.resource_clock.milliseconds(22);
    resources.advance(&mut successor).unwrap();
    assert_eq!(
        host.contacts.resource.counts().0,
        4,
        "actual due retry contacts the new provider occurrence"
    );
    assert!(
        host.contacts.resource.send(2, 999).is_err(),
        "obsolete attempt cannot fulfill its retry"
    );
    assert_eq!(
        host.contacts.resource.input(3),
        53,
        "retry retains actual original input"
    );
    host.contacts.resource.send(3, 5353).unwrap();
    resources.advance(&mut successor).unwrap();
    assert_eq!(
        resources
            .with_result_governed(
                &successor.receipt(),
                metadata::admit(&host, &actor, &fresh).unwrap(),
                |value| *value
            )
            .unwrap(),
        Some(5353)
    );
    assert!(
        host.contacts.resource.send(3, 1).is_err(),
        "committed occurrence cannot publish a duplicate"
    );
    resources.close(&mut successor).unwrap();
}
