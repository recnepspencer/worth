//! Owner-boundary custody evidence for raw mandatory requests that Home cannot install.
use worth_foundational::facade::{HomeIdentity, OwnedAsyncObligationIdentity};
use worth_query_execution::facade::query_obligations::*;

fn append(
    ledger: &mut QueryObligationLedger,
    generations: &mut QueryObligationPublicationGenerations,
    record: QueryOwnedAsyncRecord,
) -> Vec<u8> {
    let frame = ledger.prepare(record, generations).unwrap();
    let bytes = frame.bytes().to_vec();
    ledger
        .readmit(QueryObligationFrame::readmit(&bytes).unwrap())
        .unwrap();
    bytes
}
#[test]
fn mandatory_owned_async_keeps_payload_generation_lineage_and_requires_fresh_owner() {
    let home = HomeIdentity::from_observed_bytes([63; 16]).unwrap();
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let identity = OwnedAsyncObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let original = QueryOwnedAsyncRequestGeneration {
        request: 9,
        generation: 4,
    };
    let current = QueryOwnedAsyncRequestGeneration {
        request: 10,
        generation: 5,
    };
    let payload = b"original request contents".to_vec();
    let declaration = QueryOwnedAsyncDefinition {
        identity: "mandatory-source".into(),
        retention: QueryOwnedAsyncRetention::Mandatory,
        payload_contract: 37,
        max_payload_bytes: 1024,
        retry_max_attempts: 3,
        retry_delay_ticks: 2,
        timeout_ticks: 11,
        managed_queue_capacity: std::num::NonZeroU64::new(3),
    };
    let retained = append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::Retain(QueryOwnedAsyncObligation {
            identity,
            declaration: declaration.clone(),
            payload: payload.clone(),
            original,
            current: original,
            lineage: Vec::new(),
            posture: QueryOwnedAsyncPosture::InFlight,
            queue_depth: 2,
            completion: None,
        }),
    );
    let retry = append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::Progress {
            identity,
            current,
            lineage: vec![original],
            posture: QueryOwnedAsyncPosture::Retryable(QueryOwnedAsyncRetryCause::TimedOut),
            queue_depth: 2,
            completion: None,
        },
    );
    let mut cold = QueryObligationLedger::empty(home);
    for bytes in [retained, retry] {
        cold.readmit(QueryObligationFrame::readmit(&bytes).unwrap())
            .unwrap();
    }
    let row = cold.owned_async().next().unwrap();
    assert_eq!(row.identity, identity);
    assert_eq!(row.declaration, declaration);
    assert_eq!(row.original, original);
    assert_eq!(row.current, current);
    assert_eq!(row.lineage, [original]);
    assert_eq!(row.payload, payload);
    assert_eq!(row.declaration.payload_contract, 37);
    assert_eq!(row.queue_depth, 2);
    assert_eq!(
        row.continuation(),
        QueryOwnedAsyncContinuation::FreshOwnerRequired
    );
    let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
    assert_eq!(
        cold.prepare(
            QueryOwnedAsyncRecord::ProcessLocalReleased {
                identity,
                terminal: QueryOwnedAsyncTerminal::Cancelled
            },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncMandatory { identity })
    );
    assert_eq!(
        cold.prepare(
            QueryOwnedAsyncRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncPending { identity })
    );
    // A fresh empty queue is not completion of the mandatory request.
    append(
        &mut cold,
        &mut generations,
        QueryOwnedAsyncRecord::QueueDrained { identity },
    );
    assert_eq!(
        cold.prepare(
            QueryOwnedAsyncRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncPending { identity })
    );
    append(
        &mut cold,
        &mut generations,
        QueryOwnedAsyncRecord::Progress {
            identity,
            current,
            lineage: vec![original],
            posture: QueryOwnedAsyncPosture::Terminal(QueryOwnedAsyncTerminal::Fulfilled),
            queue_depth: 2,
            completion: Some(b"actual completion data".to_vec()),
        },
    );
    assert_eq!(
        cold.prepare(
            QueryOwnedAsyncRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncQueuePending { identity })
    );
    append(
        &mut cold,
        &mut generations,
        QueryOwnedAsyncRecord::QueueDrained { identity },
    );
    assert_eq!(
        cold.owned_async().next().unwrap().continuation(),
        QueryOwnedAsyncContinuation::Terminal(QueryOwnedAsyncTerminal::Fulfilled)
    );
    append(
        &mut cold,
        &mut generations,
        QueryOwnedAsyncRecord::Discharge { identity },
    );
    assert!(cold.owned_async().next().is_none());
}
#[test]
fn opted_in_owned_async_release_requires_declared_queue_and_preserves_terminal() {
    let home = HomeIdentity::from_observed_bytes([64; 16]).unwrap();
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let identity = OwnedAsyncObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let request = QueryOwnedAsyncRequestGeneration {
        request: 1,
        generation: 1,
    };
    let mut row = QueryOwnedAsyncObligation {
        identity,
        declaration: QueryOwnedAsyncDefinition {
            identity: "opted-in-source".into(),
            retention: QueryOwnedAsyncRetention::ProcessLocal,
            payload_contract: 37,
            max_payload_bytes: 1024,
            retry_max_attempts: 3,
            retry_delay_ticks: 2,
            timeout_ticks: 11,
            managed_queue_capacity: None,
        },
        payload: Vec::new(),
        original: request,
        current: request,
        lineage: Vec::new(),
        posture: QueryOwnedAsyncPosture::InFlight,
        queue_depth: 0,
        completion: None,
    };
    assert_eq!(
        ledger.prepare(QueryOwnedAsyncRecord::Retain(row.clone()), &mut generations),
        Err(QueryObligationDenial::ObligationSubjectMismatch)
    );
    row.declaration.managed_queue_capacity = std::num::NonZeroU64::new(1);
    row.queue_depth = 1;
    append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::Retain(row),
    );
    assert_eq!(
        ledger.prepare(
            QueryOwnedAsyncRecord::ProcessLocalReleased {
                identity,
                terminal: QueryOwnedAsyncTerminal::Cancelled
            },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncQueuePending { identity })
    );
    append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::QueueDrained { identity },
    );
    let released = append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::ProcessLocalReleased {
            identity,
            terminal: QueryOwnedAsyncTerminal::Cancelled,
        },
    );
    assert_eq!(
        QueryObligationFrame::readmit(&released).unwrap().record(),
        &QueryObligationRecord::OwnedAsync(QueryOwnedAsyncRecord::ProcessLocalReleased {
            identity,
            terminal: QueryOwnedAsyncTerminal::Cancelled
        })
    );
    assert_eq!(
        ledger.owned_async().next().unwrap().continuation(),
        QueryOwnedAsyncContinuation::ReleasedProcessLocal(QueryOwnedAsyncTerminal::Cancelled)
    );
    append(
        &mut ledger,
        &mut generations,
        QueryOwnedAsyncRecord::Discharge { identity },
    );
    assert!(ledger.owned_async().next().is_none());
}
