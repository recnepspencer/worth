//! Discovery returns a deleted-at-C child; only its stale output contacts a producer.
use super::*;
macro_rules! member_child_court { ($exercise:ident, $program:ident, $prefix:literal, $denied:literal) => {
fn $exercise(worker: bool, stale_root: bool, stale_child: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker { world::worker_limits() } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::$program::Program> = world::install_program_with_value(limits, 1);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut outputs = request.demand(first::Demand("a".into()))
        .retain_program_outputs::<program::$program::Program, program::$program::Tree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    loop {
        outputs = match outputs.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request).unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("initial tree terminated"),
        };
    }
    let original = request.query(query::child_output::Read(concat!($prefix, ":m").into())).execute().unwrap().rows()[0].clone();
    assert_eq!(original.value, 500);
    let member = request.query(query::member_public::Read("m".into())).execute().unwrap().rows()[0].clone();
    let document = world::attach(&host);
    macro_rules! record { ($document:expr, $input:expr, $action:ty, $key:expr) => {{
        let mut edit = $document.edit($input).idempotency_owned(&request, $key.into()).unwrap()
            .retain_in_program::<program::$program::Program, $action>(&host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        let mut advances = 0;
        loop {
            advances += 1;
            assert!(advances < 512, "edit did not progress: {:?}", edit.posture());
            edit = match edit.advance(&request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => { edit = next; break; }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            };
        }
        let WorthQueryHistoryEditFinish { document, outcome } = edit.finish(&request).unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        assert!(matches!(outcome, WorthQueryHistoryEditOutcome::Recorded { .. }));
        drop(outcome);
        document
    }}; }
    let document = if stale_child { record!(document, member_edit::Intent { key: "m".into(), value: 6 }, super::super::super::program::MemberEditAction, "member-child-edit") } else { document };
    let document = if stale_root {
        record!(document, edit::Intent { key: "a".into(), value: 3 }, super::super::super::program::EditAction, "member-child-root-edit")
    } else { document };
    let before_delete = document.current_step(&request).unwrap().id();
    let document = host.with_host_advancement(|phase| document
        .with_output_trees::<program::$program::Program, program::$program::Tree>(
            &phase, &request, &host, first::Demand("a".into()), WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))).unwrap();
    let document = record!(document, authored_member::retire::Intent, super::super::super::program::AuthoredRetireAction, "member-child-retire");
    let deleted = document.current_step(&request).unwrap().id();
    let current = document.selection(&request).unwrap();
    let at_c = host.request(&principal, &scope).on_branch(current.physical_endpoint().product_branch());
    assert!(at_c.query(query::member_public::Read("m".into())).execute().is_err());
    assert!(at_c.query(query::child_output::Read(concat!($prefix, ":m").into())).execute().is_err());
    assert!(at_c.query(query::members::Read("a".into())).execute().unwrap().rows()[0].members.is_empty());
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let selected = super::super::super::courts::checkout(&document, &request, before_delete, current.clone(), 0xd1214_30);
    if $denied {
        let refusal = selected.expect_err("D grants cannot authorize a historical-only child");
        assert_eq!(reads.protected_member_reads(), 0, "no protected D source projection before C authorization");
        assert_eq!(document.selection(&request).unwrap(), current);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        let ("advance", WorthQueryHistoryNavigationAdvanceDenial::Outputs(
            WorthQueryHistoryOutputDenial::Preparation(WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause))))) = &refusal
        else { panic!("unexpected denial: {refusal:?}"); };
        assert!(matches!(cause.kind(), WorthQueryEntityResolutionDenialKind::Refused(WorthQueryEntityResolutionRefusalKind::UnknownEntity)), "{cause:?}");
    } else {
        let selected = selected.unwrap_or_else(|cause| panic!("historical-only child fallback: {cause:?}"));
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts + u64::from(stale_root) + u64::from(stale_child));
        if stale_root { assert!(reads.reads() > 0, "stale root opens its genuine source"); }
        else { assert_eq!(reads.reads(), 0, "healthy model source remains unopened"); }
        if stale_child { assert!(reads.public_member_reads() > 0, "stale child opens its genuine source"); }
        else { assert_eq!(reads.public_member_reads(), 0, "healthy child source remains unopened"); }
        let at_d = host.request(&principal, &scope).on_branch(selected.physical_endpoint().product_branch());
        let restored = at_d.query(query::member_public::Read("m".into())).execute().unwrap();
        assert_eq!(restored.rows()[0].native_id, member.native_id);
        assert_eq!(restored.rows()[0].value, if stale_child { 6 } else { 5 });
        let discovered = at_d.query(query::members::Read("a".into())).execute().unwrap();
        assert_eq!(discovered.rows()[0].members, restored.rows());
        let output = at_d.query(query::child_output::Read(concat!($prefix, ":m").into())).execute().unwrap();
        assert_eq!(output.rows()[0].native_id, original.native_id);
        assert_eq!(output.rows()[0].value, if stale_child { 600 } else { 500 });
        let root = at_d.query(query::first_output::Read("first:a".into())).execute().unwrap();
        assert_eq!(root.rows()[0].value, if stale_root { 30 } else { 10 });
        let before_redo = host.producer_contacts_on_this_thread_for_test();
        let redo = super::super::super::courts::checkout(&document, &request, deleted, selected, 0xd1214_31)
            .unwrap_or_else(|cause| panic!("redo with an empty discovered set: {cause:?}"));
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), before_redo + if stale_root { 1 } else { 0 });
        let at_redo = host.request(&principal, &scope).on_branch(redo.physical_endpoint().product_branch());
        assert!(at_redo.query(query::member_public::Read("m".into())).execute().is_err());
        assert!(at_redo.query(query::child_output::Read(concat!($prefix, ":m").into())).execute().is_err());
        assert!(at_redo.query(query::members::Read("a".into())).execute().unwrap().rows()[0].members.is_empty());
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap()).unwrap();
}
}; }
member_child_court!(public_child, public, "downstream", false);
member_child_court!(protected_child, protected, "restricted-owned", true);
#[test]
fn restored_member_child_regenerates_without_opening_healthy_model_root() {
    public_child(false, false, true);
}
#[test]
fn worker_only_restored_member_child_regenerates_without_opening_healthy_model_root() {
    public_child(true, false, true);
}
#[test]
fn restored_member_child_uses_a_genuinely_regenerated_parent() {
    public_child(false, true, true);
}
#[test]
fn worker_only_restored_member_child_uses_a_genuinely_regenerated_parent() {
    public_child(true, true, true);
}
#[test]
fn historical_only_child_cannot_borrow_a_d_permission() {
    protected_child(false, false, true);
}
#[test]
fn worker_only_historical_only_child_cannot_borrow_a_d_permission() {
    protected_child(true, false, true);
}

#[test]
fn healthy_restored_member_child_never_opens_its_source() {
    public_child(false, false, false);
}
#[test]
fn worker_only_healthy_restored_member_child_never_opens_its_source() {
    public_child(true, false, false);
}
#[test]
fn healthy_restored_child_verifies_beside_a_genuine_parent_fallback() {
    public_child(false, true, false);
}
#[test]
fn worker_only_healthy_restored_child_verifies_beside_a_genuine_parent_fallback() {
    public_child(true, true, false);
}
