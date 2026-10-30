pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::history_retirement::*;

#[test]
fn a_held_read_keeps_only_its_own_commit_until_it_is_released() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    commit(&host, branch, 1, 0x2710_0001);
    let detached = replay(&host, branch, 1, 0x2710_0001);
    commit(&host, branch, 2, 0x2710_0002);
    let held = observe_head(runtime, branch);
    for value in 3..9 {
        commit(&host, branch, value, 0x2710_0100 + value);
    }
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW + 1, false),
        "the head, its parent and the held commit stay; everything else retired"
    );
    assert_eq!(read_at(runtime, branch, &held), 2);
    assert!(
        !selectable(runtime, branch, &detached),
        "a replay answer holds no history, so its retired commit is unavailable"
    );

    drop(held);
    commit(&host, branch, 9, 0x2710_0200);
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW, false),
        "the released history retired at the next publication"
    );
}

#[test]
fn a_held_receipt_keeps_its_commit_selectable_until_it_is_dropped() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let held_receipt = commit(&host, branch, 2, 0x2710_0402);
    for publication in 0..16 {
        commit(
            &host,
            branch,
            3 + publication % 6,
            0x2710_0410 + publication,
        );
        assert_eq!(read_at_commit(runtime, branch, &held_receipt), 2);
    }
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW + 1, false),
        "the receipt keeps only its own commit"
    );

    drop(held_receipt);
    commit(&host, branch, 9, 0x2710_0430);
    assert_eq!(
        retained_history(runtime, branch),
        (SETTLED_WINDOW, false),
        "a dropped receipt lets its history retire at the next publication"
    );
}

#[test]
fn inspection_of_a_retired_commit_fails_closed_and_its_replay_still_answers() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let key = 0x2710_0301;
    commit(&host, branch, 1, key);

    let replayed = replay(&host, branch, 1, key);
    assert!(selectable(runtime, branch, &replayed));

    commit(&host, branch, 2, 0x2710_0302);
    commit(&host, branch, 3, 0x2710_0303);
    assert!(
        !selectable(runtime, branch, &replayed),
        "inspection of a retired commit is unavailable"
    );
    match set_retention(&host, branch, 1, key).expect("the replay settles") {
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => assert_eq!(
            receipt, replayed,
            "a retired commit's replay answers its own receipt from evidence"
        ),
        other => panic!("a retired commit's replay never commits again: {other:?}"),
    }
    assert_eq!(
        read_retention(runtime, branch),
        3,
        "a retired replay never runs the request again"
    );
}
