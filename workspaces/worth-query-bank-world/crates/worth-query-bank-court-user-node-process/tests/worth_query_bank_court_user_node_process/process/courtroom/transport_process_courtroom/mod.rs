pub(crate) use worth_query_bank_worlds::facade::courtroom::transport_process_courtroom::*;

pub(crate) mod rail_completion;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs Docker: cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml -p worth-query-bank-court-user-node-process --test worth_query_bank_court_user_node_process transport_process_courtroom:: -- --ignored"]
async fn independently_authenticated_nodes_cross_real_process_and_tcp_boundaries() {
    let mut world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    elevation::assert_multi_actor_elevation(
        &world.client,
        world.peer_address,
        world.approver_address,
        world.reviewer_address,
    )
    .await;
    assert_malformed_node_request_is_typed(&world.client, world.primary_address).await;
    assert_lawful_summary(&world.client, world.primary_address).await;
    assert_node_request_saturation(&world.client, world.primary_address).await;
    assert_cross_user_denial(&world.client, world.peer_address).await;
    assert_mutation_response_loss_and_cross_user_denial(
        &world.client,
        world.primary_address,
        world.peer_address,
    )
    .await;
    live::assert_live_stream_lifecycle_and_revocation(&world.client, world.primary_address).await;
    world.authenticate_primary().await;
    recovery::assert_opaque_recovery_is_owned_by_the_authenticated_specialist(&world).await;
    let first = activity_page(&world.client, world.primary_address, "process-page-1").await;
    let token = activity_token(&first).to_owned();
    assert_eq!(
        activity_page(&world.client, world.primary_address, "process-page-1").await,
        first,
        "a lost first-page response must replay exactly"
    );
    assert_peer_cannot_resume(&world.client, world.peer_address, &token).await;

    let restarted_address = world.crash_and_restart_primary().await;
    assert_restarted_node_has_no_session(&world.client, restarted_address).await;
    world.authenticate_primary().await;
    let resumed =
        activity_resume(&world.client, restarted_address, "process-resume-1", &token).await;
    assert_terminal_resume(&resumed);
    assert_eq!(
        activity_resume(&world.client, restarted_address, "process-resume-1", &token,).await,
        resumed,
        "a lost resume response must replay exactly"
    );
    credential_lifecycle::assert_expired_session_fails_closed(&world).await;
    world.shutdown().await;
}
