use super::*;

pub(crate) fn published_outputs_hold_no_hidden_read_lease(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let mut demands = Vec::new();

    for (ordinal, source) in ["anchor-a", "sibling-a", "remote-a"]
        .into_iter()
        .enumerate()
    {
        let mut demand = request
            .demand(PlanarOutputDemand::new(source))
            .controls(controls())
            .start()
            .expect("the independent output demand starts");
        let mut published = false;
        for _ in 0..8 {
            match demand.advance(&request).expect("publication progresses") {
                WorthQueryApplicationOutputDemandProgress::Pending => {}
                WorthQueryApplicationOutputDemandProgress::Settled(_) => {
                    panic!("the checkpoint inspection must happen before readiness settles")
                }
            }
            let (outputs, pinned) = world
                .application
                .output_checkpoint_snapshot_state_for_test();
            assert_eq!(
                pinned, 0,
                "published output receipts cannot pin a read snapshot"
            );
            if outputs == ordinal + 1 {
                published = true;
                break;
            }
        }
        assert!(
            published,
            "the producer publishes within its bounded advances"
        );
        demands.push(demand);
    }
    assert_eq!(
        world
            .application
            .output_checkpoint_snapshot_state_for_test(),
        (3, 0)
    );

    for mut demand in demands {
        let mut settled = false;
        for _ in 0..32 {
            match demand
                .advance(&request)
                .expect("readiness progresses without a hidden lease")
            {
                WorthQueryApplicationOutputDemandProgress::Pending => {}
                WorthQueryApplicationOutputDemandProgress::Settled(_) => {
                    settled = true;
                    break;
                }
            }
        }
        assert!(
            settled,
            "every independent published output must become ready"
        );
        demand.close();
    }
}
