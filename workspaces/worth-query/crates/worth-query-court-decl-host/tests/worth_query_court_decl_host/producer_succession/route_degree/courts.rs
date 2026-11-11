//! Degree comes from real current parameter routes, never from the whole store.
use super::{settlement::*, *};
use std::collections::BTreeSet;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::{identity::EntityId, runtime::CurrentOwnershipContactsForTest};

fn settle_parameter(app: &world::App, request: &Request<'_, '_, '_>, parameter: u64) {
    let mut operation = request
        .demand(producers::parameter_routes::Demand(
            "root".into(),
            parameter,
        ))
        .retain_program_outputs::<Program, program::ParameterTree>(app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(t) => panic!("{:?}", t.denial()),
        }
    }
    panic!("parameter route did not settle");
}
fn settle_twin(app: &world::App, request: &Request<'_, '_, '_>) {
    let mut operation = request
        .demand(producers::twin::Demand("root".into()))
        .retain_program_outputs::<Program, program::TwinTree>(app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(t) => panic!("{:?}", t.denial()),
        }
    }
    panic!("twin route did not settle");
}
fn subject(request: &Request<'_, '_, '_>, key: &str) -> EntityId {
    request.query(Read(key.into())).execute().unwrap().rows()[0].native_id
}
fn assert_touches(
    app: &world::App,
    request: &Request<'_, '_, '_>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    n: usize,
) {
    let root = subject(request, "root");
    let outside = subject(request, "outside");
    let expected = app
        .runtime()
        .current_ownership_contacts_for_record_for_test(root, scope)
        .unwrap();
    assert_eq!(expected.publication_rows.len(), n);
    assert_eq!(expected.presence_rows.len(), n);
    assert!(expected
        .publication_rows
        .iter()
        .all(|(owner, _)| *owner == root));
    assert!(expected
        .presence_rows
        .iter()
        .all(|(owner, _)| *owner == root));
    let foreign = app
        .runtime()
        .current_ownership_contacts_for_record_for_test(outside, scope)
        .unwrap();
    assert_eq!(
        foreign.publication_rows.len(),
        1,
        "real unrelated publication"
    );
    let reads = Arc::new(Mutex::new(Vec::<CurrentOwnershipContactsForTest>::new()));
    let sink = reads.clone();
    let guard = app
        .runtime()
        .record_current_ownership_contacts_for_test(scope, move |c| sink.lock().unwrap().push(c))
        .unwrap();
    resize(app, request, 9, 90);
    let updated = std::mem::take(&mut *reads.lock().unwrap());
    assert_eq!(updated.len(), 1, "one commit custody selection");
    assert!(
        updated[0].publication_rows.is_empty(),
        "field update has no member claims"
    );
    assert!(
        updated[0].presence_rows.is_empty(),
        "field update does not discover scoped routes"
    );
    assert_eq!((updated[0].scope_links, updated[0].position_links), (0, 0));
    // A touched owned member needs its one owning row and authenticated route
    // pairing. The other positions of that owner are read to find the route,
    // but their publication rows and member incidence are never expanded.
    let outcome = request
        .advanced()
        .mutate(write_fence::ExternalWrite {
            key: "root".into(),
            action: 3,
        })
        .idempotency(&92)
        .execute_in_program::<Program>(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let member = std::mem::take(&mut *reads.lock().unwrap());
    assert_eq!(member.len(), 1);
    assert_eq!(member[0].publication_rows.len(), 1);
    assert!(expected
        .publication_rows
        .contains(&member[0].publication_rows[0]));
    assert_eq!(
        member[0]
            .presence_rows
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        expected.presence_rows.iter().copied().collect()
    );
    assert_eq!(member[0].position_work, n as u64);
    let before = checkpoint::envelopes(&checkpoint::capture(app));
    let outcome = request
        .advanced()
        .mutate(write_fence::ExternalWrite {
            key: "root".into(),
            action: 2,
        })
        .idempotency(&91)
        .execute_in_program::<Program>(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let error = denial.authored_retirement_error().unwrap();
    assert!(
        matches!(error.authored_retirement_refusal(),Some(worth_relational::facade::publication::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade{owner,..}) if owner==root)
    );
    let destroyed = std::mem::take(&mut *reads.lock().unwrap());
    assert_eq!(destroyed.len(), 1, "one destructive custody selection");
    let expected_rows = expected
        .publication_rows
        .into_iter()
        .collect::<BTreeSet<_>>();
    let expected_positions = expected.presence_rows.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(
        destroyed[0]
            .publication_rows
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        expected_rows
    );
    assert_eq!(
        destroyed[0]
            .presence_rows
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        expected_positions
    );
    assert_eq!(
        destroyed[0].position_work, n as u64,
        "each current position is actually charged in the original meter"
    );
    assert_eq!(
        (destroyed[0].scope_links, destroyed[0].position_links),
        (1, n),
        "each source incidence is charged by the original request"
    );
    assert_eq!(
        checkpoint::envelopes(&checkpoint::capture(app)),
        before,
        "refusal has no effect"
    );
    assert_eq!(subject(request, "root"), root);
    assert_eq!(subject(request, "outside"), outside);
    drop(guard);
}
#[test]
fn parameter_route_degree_is_charged_only_for_destructive_source_changes() {
    let app = world::install_degree();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle_key(&app, &request, "outside"), &request);
    for partition in 1..=3 {
        resize(&app, &request, partition, partition);
        settle_parameter(&app, &request, partition);
    }
    let origins = checkpoint::envelopes(&checkpoint::capture(&app))
        .into_iter()
        .filter_map(|e| e.get("required_output_origin").cloned())
        .collect::<Vec<_>>();
    let routes = origins
        .iter()
        .filter(|o| o["meaning"]["family"] == "parameter-route")
        .map(|o| o["meaning"]["source_partition"].to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        routes.len(),
        3,
        "one family, three actual parameter partitions"
    );
    assert_touches(&app, &request, &scope, 3);
}
#[test]
fn two_families_at_one_source_contact_only_its_own_presence_rows() {
    let app = world::install_degree();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle_key(&app, &request, "outside"), &request);
    close(settle(&app, &request), &request);
    settle_twin(&app, &request);
    assert_touches(&app, &request, &scope, 2);
}
