//! Lost transport replies cancel their actual queued work before closing its reply channel.
use super::{AdmittedAccountSummaryRequest, BankHttpExecutionQueue};
use std::{
    future::Future,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    task::{Context, Wake, Waker},
    time::{Duration, Instant},
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationToken;

struct CancellationWhenReplyCloses {
    cancellation: WorthQueryCancellationToken,
    observed: AtomicU8,
}

impl Wake for CancellationWhenReplyCloses {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.observed.store(
            if self.cancellation.is_cancelled() {
                2
            } else {
                1
            },
            Ordering::SeqCst,
        );
    }
}

#[tokio::test]
async fn abandoned_response_cancels_before_closing_worker_reply() {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let queue = BankHttpExecutionQueue { sender };
    // The dispatch worker is deliberately absent: this tests transport custody,
    // never authenticates these wire credentials and grants no application authority.
    let request = AdmittedAccountSummaryRequest {
        request_id: "cancel-before-reply-close".to_owned(),
        credential: serde_json::from_value(serde_json::json!({
            "id_token": "not-authenticated",
            "access_token": "not-authenticated",
            "nonce": "not-authenticated",
        }))
        .unwrap(),
        controls: super::super::super::protocol::BankHttpRequestControls::new(60_000, 1, 1),
        account: bank_domain::model::AccountId::new(1).unwrap(),
        deadline: Instant::now() + Duration::from_secs(60),
    };
    let mut pending = Box::pin(queue.execute(request));
    assert!(pending
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
        .is_pending());
    let mut job = receiver
        .try_recv()
        .expect("the real queue accepted the job");
    let observer = Arc::new(CancellationWhenReplyCloses {
        cancellation: job.cancellation.token(),
        observed: AtomicU8::new(0),
    });
    let waker = Waker::from(Arc::clone(&observer));
    let mut context = Context::from_waker(&waker);
    let closed = job.response.closed();
    tokio::pin!(closed);
    assert!(closed.as_mut().poll(&mut context).is_pending());
    assert_eq!(observer.observed.load(Ordering::SeqCst), 0);

    drop(pending);

    assert!(closed.as_mut().poll(&mut context).is_ready());
    assert_eq!(
        observer.observed.load(Ordering::SeqCst),
        2,
        "the worker must see cancellation when reply-channel closure wakes it"
    );
}
