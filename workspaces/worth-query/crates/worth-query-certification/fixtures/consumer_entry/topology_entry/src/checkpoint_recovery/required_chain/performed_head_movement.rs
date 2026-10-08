//! A real upstream effect must move the caller's exact accepted output basis.

use super::*;
use worth_query_consumer_values::PositiveLength;

#[test]
fn one_caller_advance_rebinds_chain_after_actual_upstream_publication() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    // One original source meter funds all three actual producers. Install
    // the same finite allowance that their public demand controls request.
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        support::seed_cycle,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".to_owned()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let initial_a = (0..256)
        .find_map(|_| match a.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual upstream initially settles");
    let initial_b = (0..256)
        .find_map(|_| match b.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual middle consumer initially settles");
    let initial_c = (0..256)
        .find_map(|_| match c.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the actual downstream initially settles");
    assert_eq!(initial_a.producer_contacts_in_this_demand(), 1);
    assert_eq!(initial_b.producer_contacts_in_this_demand(), 1);
    assert_eq!(initial_c.producer_contacts_in_this_demand(), 1);

    let selected = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(5),
        })
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&0x9176_3002_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(selected);
    let before = request.retain_read().unwrap();
    let settled = match c
        .advance(&request)
        .expect("the caller advances the real required chain after an input change")
    {
        WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("the finite performed chain needs another caller advance")
        }
    };
    let after = request.retain_read().unwrap();
    assert_ne!(before.selected_commit(), after.selected_commit());
    assert_eq!(
        settled.observation().selected_commit(),
        after.selected_commit()
    );
    assert_eq!(
        settled
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        initial_c
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
    );
    let actual_a = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(PositiveLength::get(&actual_a.rows()[0].value), 6);
    let reused = match c
        .advance(&request)
        .expect("the promoted caller remains usable")
    {
        WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("a just-settled caller cannot lose its successor interest")
        }
    };
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        reused.observation().selected_commit(),
        after.selected_commit()
    );
    // All three demand handles remain live; C may now own its authentic successor.
    drop((a, b, c));
}
