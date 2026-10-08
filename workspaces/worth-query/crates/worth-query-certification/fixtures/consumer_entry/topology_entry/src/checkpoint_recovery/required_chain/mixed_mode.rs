//! A program output wave consumes a dependent previously produced by selected-program demand.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;

#[test]
fn program_wave_rejoins_selected_program_required_successor_without_changing_its_mode() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<CheckpointProgram>(
        None,
        profile,
        4_096,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let initial_a = (0..256)
        .find_map(|_| match a.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(v) => Some(v),
        })
        .unwrap();
    assert_eq!(initial_a.producer_contacts_in_this_demand(), 1);
    let old_b = (0..256)
        .find_map(|_| match b.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(v) => Some(v),
        })
        .unwrap();
    assert_eq!(old_b.producer_contacts_in_this_demand(), 1);
    let mut outputs = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .unwrap();
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3301_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(_) = outcome else {
        panic!("real source revision must perform")
    };
    let settled = (0..256)
        .find_map(|_| match outputs.advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(v) => Some(v),
        })
        .expect("the real program wave must settle after selected-program production");
    let observation = request.retain_read().unwrap();
    request
        .at(&observation)
        .require_current_program_output(&settled, NonZeroUsize::new(1_000_000).unwrap())
        .unwrap();
    let reused = (0..256)
        .find_map(|_| match b.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(v) => Some(v),
        })
        .unwrap();
    assert_eq!(
        reused.producer_contacts_in_this_demand(),
        0,
        "the selected-program caller rejoins the actual program-wave refresh"
    );
    request
        .at(&observation)
        .require_current_output_demand(&reused, NonZeroUsize::new(1_000_000).unwrap())
        .unwrap();
    assert_ne!(
        reused.application_commit_receipt(),
        old_b.application_commit_receipt()
    );
}
