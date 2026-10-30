pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::mutation_free_external_effect::*;

#[test]
fn mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once() {
    let world = mutation_free_world("o2-mutation-free");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let binding = idempotency(61);
    let status_before = world.notice_status();
    assert_eq!(status_before, DeathNoticeStatus::NotificationRequested);

    let receipt = world.commit_with(binding.clone());
    assert!(
        receipt.co_committed_dispatch_outbox(),
        "declared external effect must co-commit its outbox even with zero domain writes"
    );
    assert_eq!(
        receipt.changed_record_count(),
        2,
        "scaffolding only: Query idempotency + dispatch outbox; zero domain field writes"
    );
    assert_eq!(
        world.notice_status(),
        status_before,
        "mutation-free retransmit must not change death-notice domain state"
    );
    assert_eq!(world.transport.attempts().len(), 1);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(WorthQueryPublishedExternalEffectFailure::LostResponse)
    );

    assert_equivalent_retry_skips_rail_and_unseen_request_consumes_sentinel(
        &world, binding, &receipt,
    );
}
