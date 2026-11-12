//! C's old output key must not stand in for a fallback's newly created generation.
use super::*;
macro_rules! source_court { ($exercise:ident, $program:ident, $prefix:literal, $protected:literal, $discovery:literal) => {
fn $exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker { world::worker_limits() } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::$program::Program> = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert!(request.query(query::first_output::Read("first:a".into())).execute().is_err());
    let document = world::attach(&host);
    let before = document.current_step(&request).unwrap().id();
    let document = host.with_host_advancement(|phase| document
        .with_output_trees::<program::$program::Program, program::$program::Root>(
            &phase, &request, &host, first::Demand("a".into()), WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .with_output_tree::<program::$program::Program, program::SeedRoot>(
            &phase, &request, &host, first::Demand("a".into()), WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))).unwrap();
    // Give the later protected recording a genuine grant at its pre-edit C.
    // Keep `before` at the original empty-output step, which navigation selects.
    let document=if $protected {
    let mut seed=document.edit(member_edit::Intent{key:"m".into(),value:6})
        .idempotency_owned(&request,"generated-source-seed".into()).unwrap()
        .retain_in_program::<program::$program::Program,program::SeedAction>(&host)
        .unwrap_or_else(|(_,cause)|panic!("seed retention: {cause:?}"));
    for count in 0..128 {
        assert!(count<127,"seed did not finish");
        seed=match seed.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next)=>next,
            WorthQueryHistoryEditAdvance::Ready(next)=>{seed=next;break;},
            WorthQueryHistoryEditAdvance::Refused{denial,..}=>panic!("seed: {denial:?}"),
        };
    }
    let seeded=seed.finish(&request).unwrap_or_else(|(_,cause)|panic!("seed finish: {cause:?}"));
    assert!(matches!(&seeded.outcome,WorthQueryFixedHistoryEditOutcome::Recorded{..}),"the grant seed is a genuine recorded edit");
    let document=seeded.document;
    drop(seeded.outcome);
    let seed_selection=document.selection(&request).unwrap();
    let seeded_request=host.request(&principal,&scope).on_branch(seed_selection.physical_endpoint().product_branch());
    assert_eq!(seeded_request.query(query::first_discovery::Read("first:a".into())).execute().unwrap().rows()[0].value,30,"C already grants the old generation before the protected edit");

    document
    } else { document };

    let mut edit = document.edit(edit::Intent { key: "a".into(), value: 5 })
        .idempotency_owned(&request, "generated-source-edit".into()).unwrap()
        .retain_in_program::<program::$program::Program, program::$program::Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("fixed edit retention: {cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "fixed edit did not finish: {:?}", edit.posture());
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => { edit = next; break; }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("fixed edit: {denial:?}"),
        };
    }
    let finish = edit.finish(&request).unwrap_or_else(|(_, cause)| panic!("fixed edit finish: {cause:?}"));
    assert!(matches!(&finish.outcome, WorthQueryFixedHistoryEditOutcome::Recorded { .. }));
    let document = finish.document;
    drop(finish.outcome);
    let future = document.current_step(&request).unwrap().id();
    assert_ne!(future, before);
    let at_future = document.selection(&request).unwrap();
    let at_c = host.request(&principal, &scope).on_branch(at_future.physical_endpoint().product_branch());
    let old_source = at_c.query(query::first_output::Read("first:a".into())).execute().unwrap().rows()[0].clone();
    assert_eq!(old_source.value, 50);
    let old_child = at_c.query(query::child_output::Read(concat!($prefix, ":first:a").into())).execute().unwrap().rows()[0].clone();
    assert_eq!(old_child.value, 5000);
    // C really grants the protected query to the OLD generation. It cannot
    // authorize the distinct source later created privately by fallback.
    assert_eq!(at_c.query(query::first_discovery::Read("first:a".into())).execute().unwrap().rows()[0], old_source);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let result = super::super::courts::checkout(&document, &request, before, at_future.clone(), 0xd1214_50);
    if $protected {
        let refusal = result.expect_err("C's old-generation grant cannot authorize a new private source");
        assert_eq!(document.selection(&request).unwrap(), at_future);
        assert_eq!(reads.discovery_reads(), 0, "no protected source projection");
        let ("advance", WorthQueryHistoryNavigationAdvanceDenial::Outputs(
            WorthQueryHistoryOutputDenial::Preparation(WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                WorthQueryApplicationRequestQueryDenial::Admission(cause))))) = &refusal
        else { panic!("expected the child source's C admission refusal: {refusal:?}"); };
        assert_eq!(cause.historical_source(), None, "no new D generation in a denied C audience");
        let family = if $discovery { "first: FirstDiscovery" } else { "restricted-from-first: FirstDiscovery" };
        assert_eq!(cause.subject(), family, "the installed query purpose names the true family");
        assert!(cause.subject().contains("FirstDiscovery"), "true protected source: {cause:?}");
        let unchanged = at_c.query(query::first_output::Read("first:a".into())).execute().unwrap();
        assert_eq!(unchanged.rows()[0], old_source);
    } else {
        let selected = result.unwrap_or_else(|cause| panic!("new fallback source generation: {cause:?}"));
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts + 2);
        let at_d = host.request(&principal, &scope).on_branch(selected.physical_endpoint().product_branch());
        let new_source = at_d.query(query::first_output::Read("first:a".into())).execute().unwrap().rows()[0].clone();
        assert_ne!(new_source.native_id, old_source.native_id, "the full generation is new");
        assert_eq!(new_source.value, 30);
        let child = at_d.query(query::child_output::Read(concat!($prefix, ":first:a").into())).execute().unwrap().rows()[0].clone();
        assert_eq!(child.value, 3000, "the producer consumed D's new source");
        assert_ne!(child.native_id, old_child.native_id);
        let redo_contacts = host.producer_contacts_on_this_thread_for_test();
        let redone = super::super::courts::checkout(&document, &request, future, selected, 0xd1214_51)
            .unwrap_or_else(|cause| panic!("select original recorded output generations: {cause:?}"));
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), redo_contacts);
        let at_redo = host.request(&principal, &scope).on_branch(redone.physical_endpoint().product_branch());
        assert_eq!(at_redo.query(query::first_output::Read("first:a".into())).execute().unwrap().rows()[0], old_source);
        assert_eq!(at_redo.query(query::child_output::Read(concat!($prefix, ":first:a").into())).execute().unwrap().rows()[0], old_child);
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap()).unwrap();
}
}; }
source_court!(public_source, public, "from-first", false, false);
source_court!(
    protected_source,
    protected,
    "restricted-from-first",
    true,
    false
);
#[test]
fn fallback_created_source_uses_its_exact_private_generation_and_redo_restores_original_outputs() {
    public_source(false);
}
#[test]
fn worker_only_fallback_created_source_uses_its_exact_private_generation_and_redo_restores_original_outputs(
) {
    public_source(true);
}
#[test]
fn fallback_created_source_cannot_borrow_the_current_older_generations_permission() {
    protected_source(false);
}
#[test]
fn worker_only_fallback_created_source_cannot_borrow_the_current_older_generations_permission() {
    protected_source(true);
}

source_court!(
    protected_discovery,
    denied_discovery,
    "restricted-from-first",
    true,
    true
);
#[test]
fn shared_discovery_and_child_source_binding_names_the_parent_family_on_c_refusal() {
    protected_discovery(false);
}
#[test]
fn worker_only_shared_discovery_and_child_source_binding_names_the_parent_family_on_c_refusal() {
    protected_discovery(true);
}
