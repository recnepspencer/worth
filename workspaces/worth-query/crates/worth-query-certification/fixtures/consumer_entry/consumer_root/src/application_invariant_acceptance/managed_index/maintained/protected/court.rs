//! Current grant and READ decisions gate every notification and hidden-field omission.
mod continuation;
mod exact_refinement;
use super::super::super::super::{authentication, managed_index::lifecycle};
use super::{
    super::{authoring::Edit, host, prefix, rule, schema::Catalog},
    access, query, status,
};
use stats_alloc::Region;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn governed_predicate_watch_readmits_current_capability_and_read_before_metadata_or_ack() {
    lifecycle::isolated_allocation_case(run, concat!(module_path!(), "::governed_predicate_watch_readmits_current_capability_and_read_before_metadata_or_ack"));
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let closed = region.change();
    assert_eq!(
        closed.bytes_allocated, closed.bytes_deallocated,
        "governed use payloads, current decisions and watch queues all close"
    );
}
fn input(prefix: &str) -> query::Find {
    query::Find {
        directory: "catalog".into(),
        prefix: prefix.into(),
    }
}
fn purpose() -> access::ReadAccess {
    access::ReadAccess {
        directory: "catalog".into(),
    }
}
fn callbacks(calls: &prefix::Calls) -> (usize, usize) {
    (
        calls.probe_validations.load(Ordering::SeqCst),
        calls.matched_sources.load(Ordering::SeqCst),
    )
}
fn set_status(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
    active: bool,
    readable: bool,
    key: u64,
) {
    let changed = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(status::SetStatus {
            identity: "watch-access".into(),
            active,
            readable,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
fn create(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
    identity: &str,
    name: &str,
    key: u64,
) {
    let changed = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(Edit::Item {
            directory: "catalog".into(),
            identity: identity.into(),
            name: name.into(),
        })
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
fn journey() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls.clone(), rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    create(
        &world,
        &principal,
        &scope,
        "protected-initial",
        "initial",
        0x9198_1000,
    );
    let before = callbacks(&calls);
    let error = request
        .query(input("future"))
        .watch_index::<prefix::PrefixIndex>()
        .err()
        .expect("current capability is required");
    assert!(
        matches!(error, WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired)
    );
    let error = request
        .query(query::FindBlocked {
            directory: "catalog".into(),
            prefix: "future".into(),
        })
        .watch_index_governed::<prefix::PrefixIndex, _, _, _>(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .err()
        .expect(
            "implicit provider Name input requires LiveMembership even when its output is omitted",
        );
    assert!(
        matches!(error, WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid)
    );
    assert_eq!(
        callbacks(&calls),
        before,
        "influence and capability refusal precede provider/Source use"
    );
    let (visible, original) = request
        .query(input("initial"))
        .watch_index_governed::<prefix::PrefixIndex, _, _, _>(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .unwrap();
    assert_eq!(visible.rows().len(), 1);
    assert_eq!(visible.rows()[0].identity, "protected-initial");
    assert!(matches!(
        &visible.rows()[0].name,
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    let public_input = || super::super::query::FindPrefix {
        directory: "catalog".into(),
        prefix: "future".into(),
    };
    let (_, mut public_watch) = request
        .query(public_input())
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let (empty, mut watch) = request
        .query(input("future"))
        .watch_index_governed::<prefix::PrefixIndex, _, _, _>(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .unwrap();
    assert!(empty.rows().is_empty());
    create(
        &world,
        &principal,
        &scope,
        "protected-future",
        "future",
        0x9198_1001,
    );
    let notification = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let error = request
        .query(query::FindAlias {
            directory: "catalog".into(),
            prefix: "future".into(),
        })
        .next_watch_governed(
            &mut watch,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .err()
        .expect("nominal alias cannot take this watch");
    assert!(
        matches!(error, WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) if matches!(cause.cause(), WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::ForeignContext)))
    );
    set_status(&world, &principal, &scope, false, true, 0x9198_1010);
    let before = callbacks(&calls);
    let error = request
        .query(input("future"))
        .next_watch_governed(
            &mut watch,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .err()
        .expect("revoked current capability refuses");
    assert!(
        matches!(error, WorthQueryApplicationRequestQueryDenial::CapabilityAdmission(cause) if cause.kind() == WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing))
    );
    assert_eq!(callbacks(&calls), before);
    set_status(&world, &principal, &scope, true, true, 0x9198_1011);
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = request
        .query(input("future"))
        .next_watch_governed(
            &mut watch,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .unwrap()
    else {
        panic!("fresh regrant preserves the original queued notification");
    };
    assert_eq!(update.notification().composite_commit(), &notification);
    let WorthQueryApplicationBasisSelectionIdentity::Product(read_at) =
        update.read().receipt().basis_identity().selection()
    else {
        panic!("governed delivery keeps an actual current World receipt");
    };
    assert_eq!(
        read_at.selected_commit(),
        request.retain_current_read().unwrap().selected_commit(),
        "C is the actual freshly reauthorized World after regrant, independently of B"
    );
    assert_eq!(update.read().rows().len(), 1);
    assert!(matches!(
        &update.read().rows()[0].name,
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    assert!(update.read().receipt().basis_released());
    let WorthQueryManagedIndexWatchOutcome::Delivered(public_update) = request
        .query(public_input())
        .next_watch(&mut public_watch)
        .unwrap()
    else {
        panic!(
            "Public and governed registrations on the same Source/scope retain independent cursors"
        );
    };
    assert_eq!(
        public_update.notification().composite_commit(),
        &notification
    );
    assert_eq!(public_update.read().rows()[0].name, "future");
    public_watch.close().unwrap();

    create(
        &world,
        &principal,
        &scope,
        "protected-next",
        "future-next",
        0x9198_1002,
    );
    let notification = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    set_status(&world, &principal, &scope, true, false, 0x9198_1012);
    let before = callbacks(&calls);
    let error = request
        .query(input("future"))
        .next_watch_governed(
            &mut watch,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .err()
        .expect("capability cannot replace independent Parent READ");
    assert!(
        matches!(error, WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::Authorization(WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::PermissionDenied)))
    );
    assert_eq!(callbacks(&calls), before);
    set_status(&world, &principal, &scope, true, true, 0x9198_1013);
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = request
        .query(input("future"))
        .next_watch_governed(
            &mut watch,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
        )
        .unwrap()
    else {
        panic!("restored READ cannot lose the unacknowledged notification");
    };
    assert_eq!(update.notification().composite_commit(), &notification);
    let WorthQueryApplicationBasisSelectionIdentity::Product(read_at) =
        update.read().receipt().basis_identity().selection()
    else {
        panic!("governed delivery keeps an actual current World receipt");
    };
    assert_eq!(
        read_at.selected_commit(),
        request.retain_current_read().unwrap().selected_commit(),
        "C is the actual freshly reauthorized World after regrant, independently of B"
    );
    assert_eq!(
        update
            .read()
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        vec!["protected-future", "protected-next"]
    );
    assert!(update
        .read()
        .rows()
        .iter()
        .all(|row| matches!(&row.name, WorthQueryApplicationDisclosed::Omitted(_))));
    assert!(matches!(
        request
            .query(input("future"))
            .next_watch_governed(
                &mut watch,
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                purpose()
            )
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    original.close().unwrap();
    watch.close().unwrap();
    assert_eq!(visible.rows()[0].identity, "protected-initial");
}
