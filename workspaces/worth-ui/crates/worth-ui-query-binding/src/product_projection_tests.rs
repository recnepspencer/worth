use crate::{
    UiPresentProjection, UiProjectionAvailability, UiProjectionUnavailableKind,
    WorthUiScalarProjectionActionOutcome, WorthUiScalarProjectionActionRequest,
    WorthUiScalarProjectionHostPlan, WorthUiScalarProjectionSourceRecord,
};

#[test]
fn legacy_scalar_host_does_not_install_presentation_async_meaning() {
    let plan = WorthUiScalarProjectionHostPlan::prepare().expect("scalar host prepares");
    let (request, _) = plan.into_parts();
    let packages = request.into_packages();
    let [package] = packages.as_slice() else {
        panic!("scalar projection installs exactly its own Query domain");
    };
    assert_eq!(
        package.package().domain_identity().owner(),
        "WORTH.ui.runtime"
    );
}

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
    let before = owner
        .read_status()
        .expect("read the Query source before denial");
    let outcome = owner
        .execute_stale_action("Deployed", 8, 1)
        .expect("the deliberate stale action reaches Query");
    let crate::WorthUiStatusActionOutcome::DeniedStaleRevision {
        active_revision,
        submitted_revision,
    } = outcome
    else {
        panic!("Query must deny the stale revision without committing");
    };
    assert_eq!(active_revision, 0);
    assert_eq!(submitted_revision, 1);
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
}

#[test]
fn product_host_installation_and_move_only_source_owner_reach_pending_and_current() {
    let plan = WorthUiScalarProjectionHostPlan::prepare().expect("product plan prepares");
    let (request, completion) = plan.into_parts();
    let installation =
        worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(request.generation(), request.into_packages())
            .expect("host installs the exact admitted packages");
    let installed = completion
        .complete(installation)
        .expect("binding completion opens the production Query owner");

    let (registration, initial) = installed.into_parts();
    assert_eq!(
        registration.view().identity().as_str(),
        "platform.pulse.status"
    );
    assert_pending(initial.observation());
    assert_eq!(initial.observation().owner_order(), 1);
    let (observation, completion) = initial.into_parts();
    let fact = scalar_fact(observation);
    assert_pending_fact(&fact);
    let owner = completion
        .admit_publication(fact.into_observation())
        .expect("the exact returned pending fact readmits its owner");

    let current = owner
        .advance(
            WorthUiScalarProjectionSourceRecord::new("ONLINE", 1).expect("native source record"),
        )
        .expect("owner-issued refresh reaches Query");
    assert_eq!(current.observation().owner_order(), 2);
    let (observation, completion) = current.into_parts();
    let observation = scalar_observation(observation);
    match observation.fact().availability() {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => {
            assert_eq!(value.as_str(), "ONLINE")
        }
        other => panic!("expected current native Query value, got {other:?}"),
    }
    let owner = completion
        .admit_publication(observation)
        .expect("the exact current observation readmits its owner");
    let updated = owner
        .advance(
            WorthUiScalarProjectionSourceRecord::new("UPDATED-LONG", 2)
                .expect("second native source record"),
        )
        .expect("owner-issued revalidation reaches Query");
    assert_eq!(updated.observation().owner_order(), 5);
    let (observation, completion) = updated.into_parts();
    let observation = scalar_observation(observation);
    match observation.fact().availability() {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => {
            assert_eq!(value.as_str(), "UPDATED-LONG")
        }
        other => panic!("expected updated native Query value, got {other:?}"),
    }
    let owner = completion
        .admit_publication(observation)
        .expect("the exact updated observation readmits its owner");
    let closed = owner.close().expect("the exact Query owner closes");
    assert!(closed.owner_terminal());
    assert_eq!(closed.live_source_count(), 0);
    assert_eq!(closed.live_attempt_count(), 0);
    assert_eq!(closed.live_resource_count(), 0);
    assert_eq!(closed.live_consumer_lease_count(), 0);
    assert_eq!(closed.retained_projection_count(), 0);
    assert_eq!(closed.projection_receipt_count(), 0);
}

#[test]
fn product_action_enters_query_refreshes_the_exact_live_target_and_closes() {
    let plan = WorthUiScalarProjectionHostPlan::prepare().expect("product plan prepares");
    let (request, completion) = plan.into_parts();
    let installation =
        worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(request.generation(), request.into_packages())
            .expect("host installs the exact admitted packages");
    let installed = completion
        .complete(installation)
        .expect("binding completion opens the production Query owner")
        .into_action_installation();

    let (_, initial) = installed.into_parts();
    let owner = publish_action_advance(initial);
    let current = owner
        .advance_source(
            WorthUiScalarProjectionSourceRecord::new("ONLINE", 1).expect("native source record"),
        )
        .expect("source truth reaches Query");
    let owner = publish_action_advance(current);

    let stale = WorthUiScalarProjectionActionRequest::new(0, "STALE").unwrap();
    let WorthUiScalarProjectionActionOutcome::Denied(denied) = owner.execute_action(stale) else {
        panic!("stale source revision must deny before Query execution");
    };
    assert_eq!(denied.active_revision(), 1);
    assert_eq!(denied.submitted_revision(), 0);
    let owner = denied.into_owner();

    let owner = execute_and_publish_action(owner, 1, "ACTION 1");
    let owner = execute_and_publish_action(owner, 1, "ACTION 2");
    assert_zero_close(owner.close().expect("action Query owner closes"));
}

fn execute_and_publish_action(
    owner: crate::WorthUiScalarProjectionActionLiveOwner,
    source_revision: u64,
    status: &str,
) -> crate::WorthUiScalarProjectionActionLiveOwner {
    let action = WorthUiScalarProjectionActionRequest::new(source_revision, status).unwrap();
    let executed = match owner.execute_action(action) {
        WorthUiScalarProjectionActionOutcome::Executed(executed) => executed,
        WorthUiScalarProjectionActionOutcome::Denied(denied) => panic!(
            "current product action denied: active={}, submitted={}",
            denied.active_revision(),
            denied.submitted_revision()
        ),
        WorthUiScalarProjectionActionOutcome::Indeterminate(indeterminate) => {
            let detail = indeterminate.detail().to_string();
            let _ = indeterminate.close();
            panic!("current product action became indeterminate: {detail}");
        }
    };
    assert_eq!(executed.evidence().source_revision(), source_revision);
    assert_eq!(executed.evidence().status(), status);
    assert!(!executed.evidence().query_receipt_digest().is_empty());
    assert_eq!(
        executed.evidence().affected_live_view_ids(),
        &["platform.pulse.status".to_string()]
    );
    let (_, action_advance) = executed.into_parts();
    let (observation, completion) = action_advance.into_parts();
    let observation = scalar_observation(observation);
    match observation.fact().availability() {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => {
            assert_eq!(value.as_str(), status)
        }
        other => panic!("expected Query-backed action value, got {other:?}"),
    }
    completion
        .admit_publication(observation)
        .expect("the exact action observation readmits its owner")
}

fn assert_pending(observation: &crate::UiProjectionObservation) {
    let crate::UiProjectionObservation::Scalar(observation) = observation else {
        panic!("product scalar owner must not issue collection evidence")
    };
    assert!(matches!(
        observation.fact().availability(),
        UiProjectionAvailability::Unavailable(unavailable)
            if unavailable.kind() == UiProjectionUnavailableKind::Pending
    ));
}

fn assert_pending_fact(fact: &crate::UiScalarProjectionFactReceipt) {
    assert!(matches!(
        fact.availability(),
        UiProjectionAvailability::Unavailable(unavailable)
            if unavailable.kind() == UiProjectionUnavailableKind::Pending
    ));
}

fn scalar_fact(
    observation: crate::UiProjectionObservation,
) -> crate::UiScalarProjectionFactReceipt {
    scalar_observation(observation).into_fact()
}

fn scalar_observation(
    observation: crate::UiProjectionObservation,
) -> crate::UiScalarProjectionObservation {
    match observation {
        crate::UiProjectionObservation::Scalar(observation) => observation,
        crate::UiProjectionObservation::ApplicationScalar(_) => {
            panic!("legacy scalar owner must not issue authored application evidence")
        }
        crate::UiProjectionObservation::Collection(_) => {
            panic!("product scalar owner must not issue collection evidence")
        }
    }
}

fn publish_action_advance(
    advance: crate::WorthUiScalarProjectionActionAdvance,
) -> crate::WorthUiScalarProjectionActionLiveOwner {
    let (observation, completion) = advance.into_parts();
    completion
        .admit_publication(scalar_observation(observation))
        .expect("the exact action-capable observation readmits its owner")
}

fn assert_zero_close(closed: crate::WorthUiScalarProjectionSourceCloseReceipt) {
    assert!(closed.owner_terminal());
    assert_eq!(closed.live_source_count(), 0);
    assert_eq!(closed.live_attempt_count(), 0);
    assert_eq!(closed.live_resource_count(), 0);
    assert_eq!(closed.live_consumer_lease_count(), 0);
    assert_eq!(closed.retained_projection_count(), 0);
    assert_eq!(closed.projection_receipt_count(), 0);
}
