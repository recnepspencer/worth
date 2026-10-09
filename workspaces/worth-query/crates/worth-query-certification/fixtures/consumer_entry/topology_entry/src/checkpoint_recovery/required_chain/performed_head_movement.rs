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
    let c_root = initial_c
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    let roots = [
        initial_a
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        initial_b
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
    ];
    let contacts_before =
        roots.map(|root| application.producer_contacts_at_root_on_this_thread_for_test(root));
    let reads_before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
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
    let reads_after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        reads_after.get(&c_root).copied().unwrap_or(0)
            - reads_before.get(&c_root).copied().unwrap_or(0),
        1,
        "C rebuilds its input once"
    );
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
    // A changed, B consumed the changed output and republished an equal value.
    // C compares its input once and cuts off without contacting its producer.
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        initial_c.producer_contacts_in_this_demand()
    );
    assert_eq!(
        reused.producer_contacts_in_this_demand(),
        settled.producer_contacts_in_this_demand()
    );
    assert_eq!(
        reused.observation().selected_commit(),
        after.selected_commit()
    );
    let contacts_after =
        roots.map(|root| application.producer_contacts_at_root_on_this_thread_for_test(root));
    assert_eq!(
        std::array::from_fn::<_, 3, _>(|i| contacts_after[i] - contacts_before[i]),
        [1, 1, 0],
        "A changes, B performs an equal republication, C cuts off"
    );
    // A+1/B+1/C+0 are owner contacts; a handle excludes upstream handlers driven by C.
    let WorthQueryApplicationOutputDemandProgress::Settled(current) = a.advance(&request).unwrap()
    else {
        panic!("the root remains settled")
    };
    assert_eq!(
        current.producer_contacts_in_this_demand(),
        initial_a.producer_contacts_in_this_demand()
    );
    let WorthQueryApplicationOutputDemandProgress::Settled(current_b) =
        b.advance(&request).unwrap()
    else {
        panic!("the middle remains settled")
    };
    assert_eq!(
        current_b.producer_contacts_in_this_demand(),
        initial_b.producer_contacts_in_this_demand()
    );
    // All three demand handles remain live.
    drop((a, b, c));
}
