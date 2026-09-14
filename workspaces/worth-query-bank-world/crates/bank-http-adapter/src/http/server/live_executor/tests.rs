use tokio_stream::StreamExt;

use crate::http::protocol::{
    BankHttpQueryBasis, BankHttpQueryBasisPosture, BankHttpQueryDisclosure,
    BankHttpQueryDisclosurePosture, BankHttpQueryOmissionPosture, BankHttpQueryPublication,
};

use super::*;

#[tokio::test]
async fn full_consumer_queue_closes_with_one_explicit_overflow() {
    let (events, receiver) = mpsc::channel(1);
    let (terminal, terminal_receiver) = oneshot::channel();
    let mut terminal = Some(terminal);
    assert!(send_live_event(
        &events,
        &mut terminal,
        BankHttpAccountActivityEvent::Opened {
            request_id: "slow-consumer".to_owned(),
        },
        "slow-consumer",
    ));
    assert!(!send_live_event(
        &events,
        &mut terminal,
        BankHttpAccountActivityEvent::Update {
            request_id: "slow-consumer".to_owned(),
            activity: BankHttpAccountActivity {
                account: "account-1".to_owned(),
                entries: Vec::new(),
            },
            publication: test_publication(),
        },
        "slow-consumer",
    ));
    drop(events);
    let mut stream = BankHttpLiveEventStream {
        events: receiver,
        terminal: terminal_receiver,
        events_closed: false,
        terminal_closed: false,
    };
    assert!(matches!(
        stream.next().await,
        Some(BankHttpAccountActivityEvent::Opened { .. })
    ));
    assert!(matches!(
        stream.next().await,
        Some(BankHttpAccountActivityEvent::Overflow {
            missed_commit_batches: 1,
            ..
        })
    ));
    assert!(stream.next().await.is_none());
}

fn test_publication() -> BankHttpQueryPublication {
    BankHttpQueryPublication {
        query_identity: "query-1".to_owned(),
        parameter_binding_identity: "binding-1".to_owned(),
        basis: BankHttpQueryBasis {
            runtime_instance: 1,
            branch: "main".to_owned(),
            snapshot: 1,
            version: 1,
            posture: BankHttpQueryBasisPosture::SelectedProduct,
        },
        capability_purpose: BankHttpQueryCapabilityPurpose::AccountActivityReview,
        disclosure: BankHttpQueryDisclosure {
            posture: BankHttpQueryDisclosurePosture::Governed,
            omission: BankHttpQueryOmissionPosture::NoOmission,
            decision_count: 1,
            disclosed_value_count: 1,
            omitted_value_count: 0,
            authorization_decision_fact_count: 1,
        },
    }
}
