//! Real installed status operations retain original custody across fresh-request stops.
use super::*;
use super::native::Step;
use super::advancement::Live;
use super::super::application_authentication::{authenticate, request_scope};
use crate::WorthUiStatusSourceOwner;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryCancellationSource,
    application_contribution::{WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption},
    application_entry::{WorthQueryApplicationLiveLimits, WorthQueryApplicationRequestExt},
};
fn history() -> NonZeroUsize { NonZeroUsize::new(64).unwrap() }
fn record(status: &str, revision: u64) -> WorthUiScalarProjectionSourceRecord {
    WorthUiScalarProjectionSourceRecord::new(status, revision).unwrap()
}
fn require_source(result: WorthUiStatusMutationResult) -> WorthUiStatusPublication {
    match result { WorthUiStatusMutationResult::Source(publication) => publication, _ => panic!("source result expected") }
}
fn observe(owner: &WorthUiStatusSourceOwner) -> Live<'_> {
    let scope = request_scope();
    let principal = authenticate(owner.application.installed_schema(), &scope).unwrap();
    owner.application.request(&principal, &scope).query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
        .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64)).unwrap()
}
#[test]
fn original_status_operation_escapes_its_application_construction_frame() {
    let original = {
        let owner = WorthUiStatusSourceOwner::install().unwrap();
        owner.prepare_source(record("TRANSFERRED", 1), &request_scope()).unwrap()
    };
    let publication = require_source(original.advance(&request_scope(), history()).expect("authentic owned allocation survives construction"));
    assert_eq!(publication.value().status, "TRANSFERRED");
    assert_eq!(publication.value().revision, 1);
    assert!(matches!(publication.live_close(), Some(WorthQueryApplicationLiveCloseOutcome::Completed(_))));
}
#[test]
fn cancelled_original_status_operation_retries_with_fresh_authority_once() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let before = owner.read_status().unwrap();
    let mut original = owner.prepare_source(record("ONLINE", 1), &request_scope()).unwrap();
    let mut live = observe(&owner);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(owner.application.installed_schema(), &stopped).expect("valid authentication precedes cancellation");
    cancellation.cancel();
    let denial = original.state.advance_with_live(&owner.application.request(&principal, &stopped), &mut live, history())
        .expect_err("the real retained operation must stop");
    original.state.live_close = Some(live.close());
    let original = original.refused(denial);
    assert!(matches!(original.stop(), Some(WorthUiStatusMutationStop::Advance(WorthQuerySourceOnlyDenial::Execution(
        WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)
    )))));
    let unchanged = owner.read_status().unwrap();
    assert_eq!(unchanged.value(), before.value());
    assert_eq!(unchanged.query_receipt().inspect().basis().version(), before.query_receipt().inspect().basis().version());
    let published = require_source(original.advance(&request_scope(), history()).expect("fresh authentication resumes the same original"));
    assert_eq!(published.value().status, "ONLINE");
    assert_eq!(published.value().revision, 1);
    let current = owner.read_status().unwrap();
    assert_eq!(current.value(), published.value());
    assert_eq!(current.query_receipt().inspect().basis().version(), published.query_receipt().inspect().basis().version());
}
#[test]
fn completed_status_close_refusal_retains_the_actual_disclosed_publication() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (mut original, mut live) = completed_source(&owner, record("RETAINED", 1));
    let scope = request_scope();
    let principal = authenticate(owner.application.installed_schema(), &scope).unwrap();
    original.state.deliver(&owner.application.request(&principal, &scope), &mut live).unwrap();
    let delivered_version = original.state.publication.as_ref().unwrap().query_receipt().inspect().basis().version();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(owner.application.installed_schema(), &stopped).unwrap();
    cancellation.cancel();
    let denial = match original.state.close_native(&owner.application.request(&principal, &stopped)) {
        Err(denial) => denial,
        Ok(_) => panic!("actual cleanup close must refuse"),
    };
    original.state.live_close = Some(live.close());
    let original = original.refused(WorthUiStatusMutationStop::Close(denial));
    assert!(matches!(original.stop(), Some(WorthUiStatusMutationStop::Close(WorthQueryOperationCloseDenial::Execution(
        WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)
    )))));
    assert_eq!(owner.read_status().unwrap().value().revision, 1, "source is saved before close refusal");
    let published = require_source(original.advance(&request_scope(), history()).expect("fresh current retained disclosure and actual close succeed"));
    assert_eq!(published.query_receipt().inspect().basis().version(), delivered_version);
    assert!(matches!(published.live_close(), Some(WorthQueryApplicationLiveCloseOutcome::Completed(_))));
}
#[test]
fn interrupted_live_delivery_retries_by_real_current_retained_disclosure() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (mut original, mut live) = completed_source(&owner, record("FIRST", 1));
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(owner.application.installed_schema(), &stopped).unwrap();
    cancellation.cancel();
    let denial = original.state.deliver(&owner.application.request(&principal, &stopped), &mut live).expect_err("real live observation terminates on cancellation");
    original.state.live_close = Some(live.close());
    let original = original.refused(denial);
    assert!(matches!(original.stop(), Some(WorthUiStatusMutationStop::LiveDelivery(WorthUiStatusLiveDeliveryStop::Cancelled))));
    assert!(original.live_close().is_some());
    let published = require_source(original.advance(&request_scope(), history()).expect("same retained commit is freshly disclosed without manufacturing a live update"));
    assert_eq!(published.value().status, "FIRST");
}
#[test]
fn completed_close_retry_cannot_return_a_superseded_cached_publication() {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (mut original, mut live) = completed_source(&owner, record("ORIGINAL", 1));
    let scope = request_scope();
    let principal = authenticate(owner.application.installed_schema(), &scope).unwrap();
    original.state.deliver(&owner.application.request(&principal, &scope), &mut live).unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(owner.application.installed_schema(), &stopped).unwrap();
    cancellation.cancel();
    assert!(original.state.close_native(&owner.application.request(&principal, &stopped)).is_err(), "actual close refuses");
    original.state.live_close = Some(live.close());
    let successor = owner.publish_source(record("SUCCESSOR", 2), history()).unwrap();
    let original = original.advance(&request_scope(), history()).expect_err("cached disclosure cannot become current evidence");
    assert!(matches!(original.stop(), Some(WorthUiStatusMutationStop::PublicationMismatch)));
    assert_eq!(owner.read_status().unwrap().value(), successor.value());
    let closed = original.close(&request_scope()).expect("real completed owner remains closable");
    assert!(closed.operation.is_some());
    assert!(closed.live.is_some());
}
fn completed_source(owner: &WorthUiStatusSourceOwner, input: WorthUiScalarProjectionSourceRecord) -> (WorthUiStatusMutation, Live<'_>) {
    let scope = request_scope();
    let mut original = owner.prepare_source(input, &scope).unwrap();
    let live = observe(owner);
    let principal = authenticate(owner.application.installed_schema(), &scope).unwrap();
    let (native, result) = original.state.native.take().unwrap().advance(&owner.application.request(&principal, &scope), history());
    original.state.native = Some(native);
    assert!(matches!(result, Ok(Step::Ready)), "actual native source completes with retained commit evidence");
    (original, live)
}
