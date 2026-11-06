//! A new build executes its actual discovery rather than trusting the old child's roster.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let (checkpoint, original, revision) = fixture::checkpoint();
    let host = world::restore_program_checkpoint::<program::Program>(checkpoint, worker, true);
    assert_eq!(
        *host.owned_revision(),
        revision,
        "pure declaration is unchanged; implementation build changed"
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut seed = request
        .demand(second::Demand("a".into()))
        .retain_program_outputs::<program::Program, original::SeedTree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    loop {
        seed = match seed.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => {
                panic!("new-build seed terminated")
            }
        };
    }
    let refreshed = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(refreshed.native_id, original.1.native_id);
    assert_eq!(refreshed.value, 2003);
    assert!(request
        .query(query::child_output::Read("downstream:m".into()))
        .execute()
        .is_err());
    let document = world::attach(&host);
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .with_output_tree::<program::Program, original::SeedTree>(
                    &phase,
                    &request,
                    &host,
                    second::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let before = document.current_step(&request).unwrap().id();
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "changed-discovery-meaning-future".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::super::program::MemberEditAction>(
            &host,
        )
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "qualified recording must finish");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(done) => {
                edit = done;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let document = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document;
    let current = document.selection(&request).unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected =
        super::super::super::courts::checkout(&document, &request, before, current, 0xd1214_40)
            .unwrap_or_else(|cause| panic!("changed discovery: {cause:?}"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "actual root fallback and new child producer"
    );
    let at_d = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at_d.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original.0]
    );
    assert_eq!(
        at_d.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[refreshed]
    );
    let children = at_d
        .query(query::child_output::Read("downstream:m".into()))
        .execute()
        .unwrap();
    assert_eq!(
        children.rows()[0].value,
        500,
        "only current discovery identifies this source and family"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn changed_discovery_build_runs_real_parent_fallback_and_discovers_its_new_child() {
    exercise(false);
}
#[test]
fn worker_only_changed_discovery_build_runs_real_parent_fallback_and_discovers_its_new_child() {
    exercise(true);
}
