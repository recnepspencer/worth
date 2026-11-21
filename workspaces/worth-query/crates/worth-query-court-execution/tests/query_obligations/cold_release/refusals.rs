//! The cold route refuses live rows, other folds' witnesses, inconsistent
//! evidence, released rows and owner-held mandatory requests.
use super::*;

fn active_run(journal: &mut Journal) -> QueryColdReleaseSubject {
    retain(journal, Pending::Run(QueryManagedRunPosture::Active))
}

#[test]
fn a_live_row_and_another_folds_release_refuse_the_cold_route() {
    let mut ended = Journal::new(home());
    let cold = active_run(&mut ended);
    let (mut journal, witness) = ended.reopen();
    // Retained by the serving incarnation, after the fold's fence.
    let live = active_run(&mut journal);
    assert_eq!(
        witness.cold_release(&journal.ledger, live),
        Err(QueryObligationDenial::IncarnationNotEnded)
    );
    let release = witness.cold_release(&journal.ledger, cold).unwrap();
    assert_eq!(release.fence(), witness.fence());
    assert_eq!(
        witness.into_cold_releases(&journal.ledger),
        Ok(vec![release.clone()]),
        "only the ended incarnation's row"
    );
    // A later open's ledger, or a ledger no fold ended, admits no release
    // minted by another fold, even for the same cold row.
    let (mut later, _) = journal.reopen();
    assert_eq!(
        later
            .ledger
            .prepare(release.clone(), &mut later.generations),
        Err(QueryObligationDenial::IncarnationNotEnded)
    );
    let mut unfenced = Journal::new(home());
    for bytes in &journal.frames {
        unfenced
            .ledger
            .readmit(QueryObligationFrame::readmit(bytes).unwrap())
            .unwrap();
    }
    let mut generations = QueryObligationPublicationGenerations::from_journal(&unfenced.ledger);
    assert_eq!(
        unfenced.ledger.prepare(release.clone(), &mut generations),
        Err(QueryObligationDenial::IncarnationNotEnded)
    );
    journal.append(release);
    let QueryColdReleaseSubject::ManagedRun(identity) = live else {
        unreachable!()
    };
    assert_eq!(
        journal
            .ledger
            .prepare(discharge(live), &mut journal.generations),
        Err(QueryObligationDenial::ManagedRunPending { identity }),
        "the live row still needs its live owner's release"
    );
}

#[test]
fn inconsistent_evidence_and_released_rows_are_not_cold_released() {
    let mut ended = Journal::new(home());
    let identity =
        ManagedCleanupObligationIdentity::from_recorded(ended.ledger.home(), ended.next());
    ended.append(QueryManagedCleanupRecord::Retain(
        QueryManagedCleanupObligation {
            identity,
            run: "inconsistent run".into(),
            failure: WorthQueryMutationCleanupFailureKind::AffinityInconsistent,
            original: None,
            release: None,
            cold_release: None,
        },
    ));
    let released = active_run(&mut ended);
    let QueryColdReleaseSubject::ManagedRun(run) = released else {
        unreachable!()
    };
    ended.append(QueryManagedRunRecord::Handoff {
        identity: run,
        posture: QueryManagedRunPosture::DirectCleanup {
            terminal: Terminal::Failed,
            disposition: Disposition::CleanupComplete,
            resources: released_resources(true),
        },
        attempt: "attempt".into(),
    });
    let (journal, witness) = ended.reopen();
    assert_eq!(
        witness.cold_release(
            &journal.ledger,
            QueryColdReleaseSubject::ManagedCleanup(identity)
        ),
        Err(QueryObligationDenial::CleanupInconsistent { identity }),
        "fresh authority never repairs inconsistent evidence"
    );
    assert_eq!(
        witness.cold_release(&journal.ledger, released),
        Err(QueryObligationDenial::ObligationSubjectMismatch),
        "a live release needs no cold release"
    );
    assert_eq!(witness.into_cold_releases(&journal.ledger), Ok(Vec::new()));
}

#[test]
fn a_mandatory_owned_async_request_stays_pending_beside_the_opted_in_process_local_control() {
    let mut ended = Journal::new(home());
    let mandatory = OwnedAsyncObligationIdentity::from_recorded(ended.ledger.home(), ended.next());
    let generation = QueryOwnedAsyncRequestGeneration {
        request: 9,
        generation: 4,
    };
    ended.append(QueryOwnedAsyncRecord::Retain(QueryOwnedAsyncObligation {
        identity: mandatory,
        declaration: resource_original(QueryOwnedAsyncRetention::Mandatory).declaration,
        payload: b"original request contents".to_vec(),
        original: generation,
        current: generation,
        lineage: Vec::new(),
        posture: QueryOwnedAsyncPosture::InFlight,
        queue_depth: 1,
        completion: None,
    }));
    let control = retain(&mut ended, Pending::Resource { result: false });
    let (mut journal, witness) = ended.reopen();
    assert_eq!(
        witness.cold_release(
            &journal.ledger,
            QueryColdReleaseSubject::Resource(mandatory)
        ),
        Err(QueryObligationDenial::OwnedAsyncMandatory {
            identity: mandatory
        }),
        "the cold door cannot admit the owner-held mandatory request"
    );
    let releases = witness.into_cold_releases(&journal.ledger).unwrap();
    assert_eq!(
        releases
            .iter()
            .map(QueryColdRelease::subject)
            .collect::<Vec<_>>(),
        [control],
        "only the opted-in process-local request is cold-released"
    );
    for release in releases {
        journal.append(release);
    }
    journal.append(discharge(control));
    let request = journal.ledger.owned_async().next().unwrap();
    assert_eq!(request.identity, mandatory);
    assert_eq!(request.posture, QueryOwnedAsyncPosture::InFlight);
    assert_eq!(
        request.continuation(),
        QueryOwnedAsyncContinuation::FreshOwnerRequired
    );
    assert_eq!(
        journal.ledger.prepare(
            QueryOwnedAsyncRecord::Discharge {
                identity: mandatory
            },
            &mut journal.generations
        ),
        Err(QueryObligationDenial::OwnedAsyncPending {
            identity: mandatory
        })
    );
}
