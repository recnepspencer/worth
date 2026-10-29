pub(crate) use worth_query_bank_worlds::facade::courtroom::transport_process_courtroom::recorded_correction::*;

pub(crate) mod reapplication;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the real Docker identity provider and cold Bank/user-node processes"]
async fn closed_money_correction_survives_response_loss_and_exact_replay() {
    let world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    let teller = world.peer_address;
    let recipient = world.primary_address;
    let source_before = balance(&world, teller, "fixture:101", "source-before").await;
    let recipient_before = balance(&world, recipient, "fixture:100", "recipient-before").await;
    let original = send(&world, "closed-original", 30).await;
    let intervening = send(&world, "closed-intervening", 7).await;
    let original_source_activity = activity(&world, teller, "fixture:101", "source-original").await;
    let original_recipient_activity =
        activity(&world, recipient, "fixture:100", "recipient-original").await;
    let original_journal = &original_source_activity
        .iter()
        .find(|entry| {
            entry.purpose == BankHttpPostingPurpose::Transfer && entry.amount_minor == -30
        })
        .expect("original thirty movement has its own journal")
        .journal;
    let intervening_journal = &original_source_activity
        .iter()
        .find(|entry| entry.purpose == BankHttpPostingPurpose::Transfer && entry.amount_minor == -7)
        .expect("intervening seven movement has its own journal")
        .journal;
    assert_ne!(original_journal, intervening_journal);

    // Copying a locator into another authenticated session cannot disclose or reverse it.
    let copied = create(
        "copied-source",
        "copied-key",
        serde_json::json!({
            "operation": "correct_send_money", "source": original,
        }),
    );
    assert!(
        matches!(forwarded(node_mutation(&world.client, recipient, &copied).await),
        BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    let mixed = BankHttpCorrectionSource {
        world_owner: intervening.world_owner,
        world_commit: intervening.world_commit,
        relational: original.relational.clone(),
    };
    let mixed_request = create(
        "mixed-source",
        "mixed-key",
        serde_json::json!({
            "operation": "correct_send_money", "source": mixed,
        }),
    );
    assert!(
        matches!(forwarded(node_mutation(&world.client, teller, &mixed_request).await),
        BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    assert_eq!(
        balance(&world, teller, "fixture:101", "source-denied").await,
        source_before - 37
    );

    let correction = create(
        "lost-correction",
        "correct-original",
        serde_json::json!({
            "operation": "correct_send_money", "source": original,
        }),
    );
    drop(
        world
            .client
            .post(format!("http://{teller}/v1/mutations"))
            .json(&correction)
            .send()
            .await
            .expect("receive headers, then discard the correction response body"),
    );
    assert_eq!(
        balance(&world, teller, "fixture:101", "source-corrected").await,
        source_before - 7
    );
    assert_eq!(
        balance(&world, recipient, "fixture:100", "recipient-corrected").await,
        recipient_before + 7
    );
    let source_after = activity(&world, teller, "fixture:101", "source-after").await;
    let recipient_after = activity(&world, recipient, "fixture:100", "recipient-after").await;
    let reversal = assert_original_preserved(
        &original_source_activity,
        &source_after,
        original_journal,
        30,
    );
    assert_eq!(
        reversal,
        assert_original_preserved(
            &original_recipient_activity,
            &recipient_after,
            original_journal,
            -30
        )
    );
    assert_ne!(reversal, original_journal);
    assert_ne!(reversal, intervening_journal);

    let replay = create(
        "correction-replay",
        "correct-original",
        serde_json::json!({
            "operation": "correct_send_money", "source": original,
        }),
    );
    assert!(matches!(
        forwarded(node_mutation(&world.client, teller, &replay).await),
        BankHttpMutationOutcome::Pending { sequence: 1, .. }
    ));
    let resume = serde_json::json!({ "command": "resume", "request_id": "replay-delivery",
        "custody_id": "correction-replay", "sequence": 2, "controls": controls() });
    let crossed = forwarded(node_mutation(&world.client, recipient, &resume).await);
    assert!(
        matches!(crossed, BankHttpMutationOutcome::Refused { denial, .. }
        if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    let replayed = forwarded(node_mutation(&world.client, teller, &resume).await);
    let BankHttpMutationOutcome::Completed {
        source: replayed,
        recovered: true,
        ..
    } = replayed
    else {
        panic!("same-key retry delivers the actual original correction");
    };
    let original_status = forwarded(
        node_mutation(
            &world.client,
            teller,
            &serde_json::json!({
                "command": "resume", "request_id": "original-correction-status",
                "custody_id": "lost-correction", "sequence": 2, "controls": controls(),
            }),
        )
        .await,
    );
    let BankHttpMutationOutcome::Completed {
        source: performed,
        recovered: false,
        ..
    } = original_status
    else {
        panic!("the worker retained the original fresh correction before its response was lost");
    };
    assert_eq!(performed, replayed);
    assert_ne!(performed, original);
    let drift = create(
        "intent-drift",
        "correct-original",
        serde_json::json!({
            "operation": "correct_send_money", "source": intervening,
        }),
    );
    assert!(
        matches!(forwarded(node_mutation(&world.client, teller, &drift).await),
        BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Stale)
    );
    assert_eq!(
        activity(&world, teller, "fixture:101", "source-replayed").await,
        source_after
    );
    assert_eq!(
        activity(&world, recipient, "fixture:100", "recipient-replayed").await,
        recipient_after
    );
    send(&world, "after-correction", 3).await;
    assert_eq!(
        balance(&world, teller, "fixture:101", "later-source").await,
        source_before - 10
    );
    assert_eq!(
        balance(&world, recipient, "fixture:100", "later-recipient").await,
        recipient_before + 10
    );
    for custody in ["lost-correction", "correction-replay"] {
        let read = serde_json::json!({
            "command": "read", "request_id": format!("retained-{custody}"),
            "custody_id": custody, "sequence": 3, "account": "fixture:101",
            "controls": {"deadline_milliseconds": 5_000, "maximum_results": 16},
        });
        let BankHttpMutationOutcome::Delivered {
            activity,
            publication,
            ..
        } = forwarded(node_mutation(&world.client, teller, &read).await)
        else {
            panic!("fresh authority reads the actual retained correction publication");
        };
        assert_eq!(activity.entries, source_after);
        assert_eq!(publication.basis.version, performed.relational.version_id);
        close(&world, custody, 4).await;
    }
    world.shutdown().await;
}
