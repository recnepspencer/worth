//! Real retained-effect delivery keeps admitted nested values through its receipt and ACK.
#[path = "live/cause.rs"]
pub(super) mod cause;
#[path = "live/native.rs"]
mod native;
#[path = "live/publish.rs"]
pub(super) mod publish;
#[path = "live/query.rs"]
pub(super) mod query;

use super::{host, opaque, schema::AdmissionSchema};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    primary_graph::*,
};

type Subscription<'host> =
    WorthQueryApplicationLiveSubscription<'host, AdmissionSchema, query::ReadLive>;

#[test]
fn live_nested_decode_requires_original_funding_before_receipt_and_cause_handoff() {
    let funded = host::install(super::FUNDED_WORK);
    let mut live = open(&funded);
    publish(&funded, 78_001);
    let scope = request_scope();
    let principal = authenticate_operator(funded.installed_schema(), &scope);
    let request = funded
        .runtime()
        .request(&principal, &scope)
        .on_branch(funded.current_world());
    reset();
    let WorthQueryApplicationLiveOutcome::Delivered(update) = live.next(&request).unwrap() else {
        panic!("funded actual retained cause must deliver its nondefault nested value");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(update.result().counter, 0);
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(update.receipt().result_buffer().unwrap().released());
    assert!(matches!(
        live.next(&request).unwrap(),
        WorthQueryApplicationLiveOutcome::Pending
    ));
    assert!(matches!(
        live.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    drop(funded);

    let narrow = host::install(super::FUNDED_WORK);
    let mut live = native::open(&narrow);
    publish(&narrow, 78_002);
    reset();
    let (baseline, funded) = native::next(&narrow, &mut live, super::FUNDED_WORK);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = baseline else {
        panic!("the same native delivery helper must genuinely deliver before narrowing");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    publish(&narrow, 78_003);
    use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;
    let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Raw(wire),
    ) = super::binding::NestedBinding::encode(&opaque::SwallowingValue(super::value())).unwrap()
    else {
        panic!("the installed canonical binding has a string carrier");
    };
    // The actual public bill includes the last canonical wire comparison.
    // Sixteen 256-byte nested tags dominate the admitted seed itself. Removing
    // that comparison and half a wire reaches this seed, after read one, rather
    // than underfunding the unrelated source publication. Assertions qualify it.
    let work = funded
        .charged_work()
        .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
        .expect("a genuine live delivery funds more than its last decoder tail");
    reset();
    let (denied, _) = native::next(&narrow, &mut live, work);
    let WorthQueryApplicationLiveOutcome::ProjectionDenied(denial) = denied else {
        panic!("the swallowed second admitted decode must not hand out a live result or receipt");
    };
    assert_eq!(
        denial,
        WorthQueryApplicationProjectionDenialKind::ValueDecode(
            WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling),
        )
    );
    assert_eq!(
        returned(),
        1,
        "the actual live projector caught the error and returned success"
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the actual nested seed swallowed native admission refusal"
    );
    assert!(matches!(
        native::next(&narrow, &mut live, super::FUNDED_WORK).0,
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(live);
    assert_eq!(
        host::read(&narrow, "root").value,
        super::value(),
        "a fresh affordable request has no latched predecessor refusal"
    );
    let mut fresh = native::open(&narrow);
    publish(&narrow, 78_004);
    reset();
    let (retried, _) = native::next(&narrow, &mut fresh, super::FUNDED_WORK);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = retried else {
        panic!("a fresh funded native live request must not inherit the swallowed refusal");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(matches!(
        fresh.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

#[test]
fn live_late_original_cancellation_cannot_ack_and_a_fresh_live_request_succeeds() {
    let host = host::install(super::FUNDED_WORK);
    let mut live = open(&host);
    publish(&host, 78_101);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(request_scope().deadline(), cancellation.token());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world());
    let guard = CancelAfterProjection::arm(cancellation);
    let outcome = live.next(&request).unwrap();
    assert_eq!(
        returned(),
        1,
        "cancellation lands after real nondefault decoding"
    );
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationLiveOutcome::ProjectionDenied(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            ) | WorthQueryApplicationLiveOutcome::Cancelled
        ),
        "a late original cancellation may not produce a delivery/receipt"
    );
    assert!(matches!(
        live.next(&request).unwrap(),
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(guard);
    drop(live);

    let mut fresh = open(&host);
    publish(&host, 78_102);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world());
    reset();
    let WorthQueryApplicationLiveOutcome::Delivered(update) = fresh.next(&request).unwrap() else {
        panic!("a fresh funded original request must deliver through the same live owner");
    };
    assert_eq!(update.result().value, super::value());
    assert_eq!(returned(), 1);
    assert!(update.receipt().result_buffer().unwrap().released());
    assert!(matches!(
        fresh.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

fn open(host: &host::Host) -> Subscription<'_> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .query(query::ReadLive {
            identity: "root".to_owned(),
        })
        .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, Some(5_120)))
        .expect("ordinary installed nested live query opens without projecting a value")
}

fn publish(host: &host::Host, idempotency: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .advanced()
        .mutate(publish::Publish {
            changed: cause::Changed {
                identity: "root".to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the ordinary installed effect-only mutation executes");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "only a genuine native publication may supply the live cause"
    );
}

thread_local! {
    static RETURNED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static CANCEL_AFTER_PROJECTION: std::cell::RefCell<Option<WorthQueryCancellationSource>> = const { std::cell::RefCell::new(None) };
}
fn reset() {
    RETURNED.set(0);
    opaque::reset();
}
fn returned() -> u32 {
    RETURNED.get()
}
fn projector_returned() -> bool {
    RETURNED.set(RETURNED.get() + 1);
    CANCEL_AFTER_PROJECTION.with_borrow(|source| {
        if let Some(source) = source {
            source.cancel();
            true
        } else {
            false
        }
    })
}
struct CancelAfterProjection;
impl CancelAfterProjection {
    fn arm(source: WorthQueryCancellationSource) -> Self {
        reset();
        CANCEL_AFTER_PROJECTION.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some(source);
        });
        Self
    }
}
impl Drop for CancelAfterProjection {
    fn drop(&mut self) {
        CANCEL_AFTER_PROJECTION.with_borrow_mut(|slot| *slot = None);
    }
}
