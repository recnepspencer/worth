pub(crate) use worth_query_bank_worlds::facade::server::ordinary_reads::advancement_custody::*;

#[test]
fn page_resume_and_live_entries_refuse_before_their_own_reader() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let fixture = ordinary_read_world("advancement-public-query-doors", 0);
    let owner = fixture.authenticate(OWNER);
    let activity = || {
        fixture
            .world
            .runtime
            .account_activity(fixture.personal_account)
            .as_principal(&owner)
    };
    let controls = || BankReadControls::current(request_scope(), 1).unwrap();
    let live_controls = || LiveControls::bounded(request_scope(), 16, 8).unwrap();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for zero_memory in [false, true] {
            for entry in 0..4 {
                bound(None);
                let scope = request_scope();
                let resume = || Resume::new(NonZeroUsize::MIN, &scope);
                let before = reads();
                match entry {
                    0 => {
                        activity().page(controls()).unwrap();
                    }
                    1 => {
                        let (_, continuation) = activity().page(controls()).unwrap().into_parts();
                        let before = reads();
                        activity().resume(continuation.unwrap(), resume()).unwrap();
                        assert!(reads() > before, "admitted resume must contact its reader");
                    }
                    2 => {
                        activity().subscribe(live_controls()).unwrap().close();
                    }
                    3 => {
                        let mut live = activity().subscribe(live_controls()).unwrap();
                        let before = reads();
                        assert!(matches!(live.poll(&owner, &scope), Live::Pending));
                        assert!(reads() > before, "admitted next must contact its reader");
                    }
                    _ => unreachable!(),
                }
                assert!(
                    reads() > before,
                    "admitted entry {entry} must contact its reader"
                );
                let continuation = (entry == 1)
                    .then(|| activity().page(controls()).unwrap().into_parts().1.unwrap());
                let mut live = (entry == 3).then(|| activity().subscribe(live_controls()).unwrap());
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    Some(if zero_memory { 0 } else { 64 * 1_024 * 1_024 }),
                    Some(if zero_memory { 16_000_000 } else { 0 }),
                )));
                let before = reads();
                let cause = match entry {
                    0 => query_cause(activity().page(controls()).err().unwrap()),
                    1 => query_cause(
                        activity()
                            .resume(continuation.unwrap(), resume())
                            .err()
                            .unwrap(),
                    ),
                    2 => query_cause(activity().subscribe(live_controls()).err().unwrap()),
                    3 => match live.as_mut().unwrap().poll(&owner, &scope) {
                        Live::ExecutionRequest(cause) => cause,
                        other => panic!("live next lost its opening cause: {other:?}"),
                    },
                    _ => unreachable!(),
                };
                assert_eq!(reads(), before, "refused entry {entry} reached its reader");
                if !zero_memory {
                    assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
                } else {
                    let Denial::Resource(Resource::MemoryLimit {
                        level,
                        requested,
                        admitted,
                    }) = cause
                    else {
                        panic!("bytes lost: {cause:?}");
                    };
                    assert_eq!(level, Level::Policy);
                    assert!(requested > 0);
                    assert_eq!(admitted, 0);
                }
            }
        }
    }
}
