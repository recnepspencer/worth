pub(crate) use worth_query_worlds::facade::certification::source_only::retained::*;

pub(crate) mod native_postures;

#[test]
fn lost_reply_bounded_retry_reads_original_commit_despite_new_current_truth() {
    let host = host::install_program::<RetainedProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let (original_commit, replay) = {
        let request = host.runtime().request(&principal, &scope);
        let first = match operation(&host, &request, 18, 5018).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            _ => panic!("fresh retained commit"),
        };
        let original_commit = observation(first.evidence()).selected_commit().clone();
        first
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("first close: {cause:?}"));
        let later = match operation(&host, &request, 25, 5025).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            _ => panic!("later source commit"),
        };
        later
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("later close: {cause:?}"));
        assert_eq!(read_retention(host.runtime(), host.current_world()), 25);
        let replay = match operation(&host, &request, 18, 5018).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Progress(owner) => owner,
            _ => panic!("replay owns actual recovered evidence"),
        };
        assert_eq!(
            replay.posture(),
            WorthQuerySourceOnlyPosture::RetainedDeliveryPending
        );
        let replay = match replay.advance(&request, Some(NonZeroUsize::new(1).unwrap())) {
            WorthQuerySourceOnlyAdvance::Refused {
                operation,
                denial:
                    WorthQuerySourceOnlyDenial::History(
                        WorthQueryApplicationHistorySelectionDenial::CommitUnavailable,
                    ),
            } => operation,
            _ => panic!("bounded miss retains original evidence"),
        };
        (original_commit, replay)
    };
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let completed = match replay.advance(&fresh, None) {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        _ => panic!("funded fresh retained replay"),
    };
    assert_eq!(
        observation(completed.evidence()).selected_commit(),
        &original_commit
    );
    let historical = fresh
        .at(observation(completed.evidence()))
        .query(ReadRetention {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(historical.rows()[0].retention_days, 18);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            publication: None,
            ..
        }
    ));
    assert_eq!(host.contacts.counts(), (2, 2));
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, cause)| panic!("replay close: {cause:?}"));
}
