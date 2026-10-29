pub(crate) use worth_query_worlds::facade::certification::application_graph::adoption::signal_fork_inheritance::*;

#[test]
fn a_fork_of_both_components_inherits_the_law_its_source_was_running() {
    let host = publish_on_second_program();
    let main = host.current_world();
    let fork = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().fork_signal())
                .create(&host_phase)
                .expect("the Relational and Signal fork must publish")
        })
        .expect("installed host call opens");

    assert_eq!(
        settle(set_retention(&host, fork, HIGH_RETENTION, 0x9175_0051)),
        RetentionVerdict::Performed(HIGH_RETENTION),
        "the fork runs the second program its source was running"
    );
    assert_eq!(read_retention(host.runtime(), fork), HIGH_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), main),
        SEED_RETENTION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_retention(&host, fork, LOW_RETENTION, 0x9175_0052)),
        RetentionVerdict::violated("document-retention-v2"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_retention(host.runtime(), fork), HIGH_RETENTION);
    assert_eq!(read_retention(host.runtime(), main), SEED_RETENTION);
}
