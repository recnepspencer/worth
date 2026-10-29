pub(crate) use worth_query_worlds::facade::certification::application_graph::adoption::branch_divergence::*;

#[test]
fn two_programs_over_one_schema_carry_two_revisions() {
    let first = validated_first();
    let second = validated_second();
    assert_ne!(
        first.revision(),
        second.revision(),
        "programs declaring different rules cannot share a canonical revision"
    );
    assert_eq!(
        first.revision(),
        validated_first().revision(),
        "revalidating the same authored meaning must mint the same revision"
    );
    assert_eq!(second.revision(), validated_second().revision());
}

#[test]
fn a_first_program_occurrence_enforces_the_first_law() {
    let host = publish_on_first_program();
    let branch = host.current_world();

    assert_eq!(
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0001)),
        RetentionVerdict::Performed(LOW_RETENTION)
    );
    assert_eq!(read_retention(host.runtime(), branch), LOW_RETENTION);

    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0002)),
        RetentionVerdict::violated("document-retention-v1"),
        "the value the second law admits must be refused by the first program's law"
    );
    assert_eq!(
        read_retention(host.runtime(), branch),
        LOW_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_second_program_occurrence_enforces_the_second_law() {
    let host = publish_on_second_program();
    let branch = host.current_world();

    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0011)),
        RetentionVerdict::Performed(HIGH_RETENTION)
    );
    assert_eq!(read_retention(host.runtime(), branch), HIGH_RETENTION);

    assert_eq!(
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0012)),
        RetentionVerdict::violated("document-retention-v2"),
        "the value the first law admits must be refused by the second program's law"
    );
    assert_eq!(
        read_retention(host.runtime(), branch),
        HIGH_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    assert!(
        host.supported_program::<RetentionProgramP1>().is_some(),
        "the peer program this host rostered must be nameable"
    );
    assert!(
        host.supported_program::<UnrosteredRetentionProgram>()
            .is_none(),
        "a program this host never rostered cannot be nameable"
    );

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0021)),
        RetentionVerdict::violated("document-retention-v1"),
        "a rostered peer's law cannot govern an occurrence running another program"
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(
        read_retention(host.runtime(), branch),
        SEED_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
    assert_eq!(
        before.selected_commit(),
        after.selected_commit(),
        "a refused candidate cannot have moved the commit head"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active_on_a_second_program_host() {
    let host = publish_on_second_program();
    let branch = host.current_world();
    assert!(
        host.supported_program::<RetentionProgramP0>().is_some(),
        "the peer program this host rostered must be nameable"
    );

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0031)),
        RetentionVerdict::violated("document-retention-v2")
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION);
    assert_eq!(before.selected_commit(), after.selected_commit());
}

#[test]
fn a_forked_branch_inherits_the_law_its_source_was_running() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let fork = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the Relational fork must publish")
        })
        .expect("installed host call opens");

    assert_eq!(
        settle(set_retention(&host, fork, FORK_RETENTION, 0x9175_0041)),
        RetentionVerdict::Performed(FORK_RETENTION),
        "the fork runs the program its source was running"
    );
    assert_eq!(read_retention(host.runtime(), fork), FORK_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), main),
        SEED_RETENTION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_retention(&host, fork, LOW_RETENTION, 0x9175_0042)),
        RetentionVerdict::Performed(LOW_RETENTION),
        "the other rostered program's law does not reach this fork"
    );
    assert_eq!(read_retention(host.runtime(), fork), LOW_RETENTION);

    assert_eq!(
        settle(set_retention(&host, fork, HIGH_RETENTION, 0x9175_0043)),
        RetentionVerdict::violated("document-retention-v1"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_retention(host.runtime(), fork), LOW_RETENTION);

    assert_eq!(read_retention(host.runtime(), main), SEED_RETENTION);
}
