//! A real authored restoration creates the child source only on recording D.
use super::*;
mod program;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn settle(host: &Host, request: &family::Request<'_, '_, '_>) {
    let mut tree = request
        .demand(upstream::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for count in 0..128 {
        assert!(count < 127, "original tree did not settle");
        tree = match tree.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("tree terminated"),
        };
    }
}
/// Records the restoration and returns how often the model source was read
/// while it was recorded. The document reads its own model either way.
fn restoration(worker: bool, forest: bool) -> u64 {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let original_member = request
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let original_parent = request
        .query(query::member_public::Read("upstream:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let document = record!(
        request,
        host,
        world::attach(&host),
        authored_member::retire::Intent,
        super::super::super::program::AuthoredRetireAction,
        "recording-retire-member-source"
    );
    let selection = document.selection(&request).unwrap();
    let at_c = host
        .request(&principal, &scope)
        .on_branch(selection.physical_endpoint().product_branch());
    assert!(at_c
        .query(query::member_public::Read("m".into()))
        .execute()
        .is_err());
    let document = if forest {
        host.with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    upstream::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap()
    } else {
        document
    };
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let document = record!(
        request,
        host,
        document,
        authored_member::restore::Intent {
            key: "m".into(),
            value: 6
        },
        super::super::super::program::AuthoredRestoreAction,
        "recording-restore-new-member-source"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + u64::from(forest),
        "only the new child's producer runs"
    );
    let model_reads = reads.model_reads();
    assert_eq!(
        reads.public_member_reads() > 0,
        forest,
        "child's real source is read at D"
    );
    drop(reads);
    let selected = document.selection(&request).unwrap();
    let at_d = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    let member = at_d
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_ne!(
        member.native_id, original_member.native_id,
        "restoration created a full new generation"
    );
    assert_eq!(member.value, 6);
    assert_eq!(
        at_d.query(query::members::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .members,
        vec![member]
    );
    assert_eq!(
        at_d.query(query::member_public::Read("upstream:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original_parent]
    );
    if forest {
        assert_eq!(
            at_d.query(query::child_output::Read("downstream:m".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            600
        );
    }
    assert!(
        at_c.query(query::member_public::Read("m".into()))
            .execute()
            .is_err(),
        "C stays unchanged"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    model_reads
}
/// The parent's source is the model the document itself reads, so a healthy
/// parent is one whose forest adds no model read to the document's own.
fn exercise(worker: bool) {
    assert_eq!(
        restoration(worker, true),
        restoration(worker, false),
        "healthy model root remains unopened"
    );
}
#[test]
fn recording_verifies_a_healthy_parent_and_produces_a_child_whose_source_exists_only_at_d() {
    exercise(false);
}
#[test]
fn worker_only_recording_verifies_a_parent_and_produces_a_child_whose_source_exists_only_at_d() {
    exercise(true);
}
