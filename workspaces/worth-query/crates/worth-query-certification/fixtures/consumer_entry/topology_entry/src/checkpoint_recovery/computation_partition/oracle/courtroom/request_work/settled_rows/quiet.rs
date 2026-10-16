//! Open versus lawfully closed read interests at one unchanged Native world.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandHandle, WorthQueryApplicationRequest,
    WorthQueryProgramOutputOperationAdvance, WorthQuerySettledProgramOutputOperation,
};
use worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime;

type Application = WorthQueryProgramApplicationRuntime<CheckpointSchema, ScaleProgram>;
type Settled = WorthQuerySettledProgramOutputOperation<CheckpointSchema, ScaleProgram, OracleRoot>;

fn settle_read_tree(app: &Application, key: &str) -> Settled {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    let mut operation = request
        .demand(PlanarOutputDemand::new(key))
        .controls(Default::default())
        .retain_program_outputs::<ScaleProgram, OracleRoot>(app)
        .unwrap_or_else(|failure| panic!("read-tree construction: {:?}", failure.denial()));
    // Follow only genuine Pending custody. A refusal or terminal is a setup
    // failure, never a successful locality result or an automatic retry.
    loop {
        let (scope, principal) = authenticate(app);
        let request = app.request(&principal, &scope);
        operation = match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => return settled,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(owner) => {
                panic!("{:?}", owner.denial())
            }
        };
    }
}

fn required_members(app: &Application) -> usize {
    app.required_custody_breakdown_for_test()
        .into_iter()
        .find(|(kind, _, _)| *kind == "members")
        .expect("the actual registry reports its required members")
        .1
}

fn measure(
    app: &Application,
    output: &mut WorthQueryApplicationOutputDemandHandle<'_, CheckpointSchema, RegionOutputDemand>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, CheckpointSchema>,
) -> u64 {
    // Queue length counts physical hints, including inactive/acknowledged
    // tokens until a queue pop removes them. Settled does not mean globally
    // drained. This selected current caller must leave those hints untouched.
    let queued = app.queued_required_work_for_test();
    let contacts = app.producer_contacts_on_this_thread_for_test();
    app.caller_request_work_for_test();
    app.caller_request_navigation_for_test();
    let rows = app.whole_registry_rows_visited_for_test();
    assert!(matches!(
        output.advance(request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let work = app.caller_request_work_for_test();
    assert_eq!(work.len(), 1);
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts,
        "a measured current caller executes no producer"
    );
    assert_eq!(
        app.queued_required_work_for_test(),
        queued,
        "the selected current caller leaves unrelated physical queue custody untouched"
    );
    assert_eq!(
        app.whole_registry_rows_visited_for_test() - rows,
        0,
        "a measured current caller visits no whole-registry rows"
    );
    for (navigation, operations) in app.caller_request_navigation_for_test() {
        assert!(navigation <= operations * primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<CheckpointSchema>::maximum_ordered_descent_work_for_test());
    }
    work[0]
}

pub(super) fn assert_same_world_current_work(
    app: &Application,
    root: &mut WorthQueryApplicationOutputDemandHandle<'_, CheckpointSchema, PlanarOutputDemand>,
    output: &mut WorthQueryApplicationOutputDemandHandle<'_, CheckpointSchema, RegionOutputDemand>,
) {
    // Existing triangles outside both the edited triangle and the caller's
    // scope. Each actual OracleRoot contains the root and both declared leaves.
    let foreign = [
        settle_read_tree(app, "unrelated-12"),
        settle_read_tree(app, "unrelated-15"),
    ];
    for tree in &foreign {
        // The root is retained separately; output_count counts its two leaves.
        assert_eq!(tree.settlement().output_count(), 2);
    }
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    // Foreign setup may publish. Warm in dependency order before fixing the
    // world and caller used for BOTH measurements.
    assert!(matches!(
        root.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert!(matches!(
        output.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let original = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let members_open = required_members(app);
    let records_open = app.registry_row_count_for_test();
    assert!(
        app.queued_required_work_for_test() > 0,
        "foreign tree setup exercises locality with actual queued hints"
    );
    let open_work = measure(app, output, &request);
    let contacts = app.producer_contacts_on_this_thread_for_test();
    for tree in foreign {
        let closed = tree
            .close(&request)
            .unwrap_or_else(|failure| panic!("original read-tree close: {:?}", failure.denial()));
        assert!(closed.terminal_denial().is_none());
        assert!(closed.construction_denial().is_none());
    }
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        contacts,
        "read close does not replay any producer"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &original,
        "read close must leave the physical graph world unchanged"
    );
    let members_closed = required_members(app);
    assert!(
        members_closed < members_open,
        "actual native required interests were discharged: {members_open} -> {members_closed}"
    );
    // Read close releases actual interests, not the global scheduling queue.
    // A deactivated queued membership remains a physical token until popped.
    // Healthy Ready can remain cached after close: both measured requests keep
    // the same graph commit and record lookup population. Each must preserve
    // its own queue count; neither measurement drains the foreign hints.
    assert_eq!(app.registry_row_count_for_test(), records_open);
    let closed_work = measure(app, output, &request);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &original
    );
    assert_eq!(
        open_work, closed_work,
        "settled unrelated open interests add zero caller work at the same world"
    );
}
