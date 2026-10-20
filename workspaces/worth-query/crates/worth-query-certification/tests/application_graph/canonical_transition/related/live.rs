//! Actual retained-effect delivery crosses the changed-codec installation boundary.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    declaration::application_schema::ApplicationScalarValueBinding,
    primary_graph::*,
};
fn publish_old(host: &host::Host, id: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .advanced()
        .mutate(publish::Publish {
            changed: cause::Changed {
                identity: "derived-report".into(),
            },
        })
        .without_source()
        .idempotency(&id)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
fn publish_target(host: &successor::host::Host, id: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .advanced()
        .mutate(successor::publish::Publish {
            changed: successor::cause::Changed {
                identity: "derived-report".into(),
            },
        })
        .without_source()
        .idempotency(&id)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
#[test]
fn held_v1_live_cause_uses_original_decoder_and_v2_live_uses_regenerated_output() {
    let original = host::install(model::authored_value()).unwrap();
    let _settled = host::produce(&original);
    let scope = request_scope();
    let principal = authenticate_operator(original.installed_schema(), &scope);
    let mut old = original
        .runtime()
        .request(&principal, &scope)
        .on_branch(original.runtime().current_world())
        .query(live_query::ReadLive {
            identity: "derived-report".into(),
        })
        .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, Some(65_536)))
        .unwrap();
    publish_old(&original, 180301);
    let target = journey::successor(host::capture(&original));
    assert_eq!(successor::output::decodes(), 0);
    let fresh = request_scope();
    let principal = authenticate_operator(original.installed_schema(), &fresh);
    let request = original
        .runtime()
        .request(&principal, &fresh)
        .on_branch(original.runtime().current_world());
    let WorthQueryApplicationLiveOutcome::Delivered(update) = old.next(&request).unwrap() else {
        panic!("held original V1 cause delivers through its original owner")
    };
    assert_eq!(update.result().children.len(), 2);
    for (_, value) in &update.result().children {
        assert_eq!(output::literals(value).1, 12);
    }
    assert_eq!(successor::output::decodes(), 0);
    assert!(update.receipt().result_buffer().unwrap().released());
    assert!(matches!(
        old.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    let _new = successor::host::produce(&target);
    let mut live = successor::live_native::open(&target);
    publish_target(&target, 180302);
    let (outcome, _) =
        successor::live_native::next(&target, &mut live, &request_scope(), 8_000_000);
    let WorthQueryApplicationLiveOutcome::Delivered(update) = outcome else {
        panic!("V2 genuine cause uses V2 producer output")
    };
    for (_, value) in &update.result().children {
        assert_eq!(successor::output::literals(value).1, 1012);
    }
    assert!(matches!(
        live.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    assert!(original.runtime().close_live_delivery().owner_terminal());
}
#[test]
fn changed_codec_live_swallowed_refusal_and_cancelled_domain_error_cannot_ack() {
    let original = host::install(journey::heavy()).unwrap();
    let _old = host::produce(&original);
    let target = journey::successor(host::capture(&original));
    let _new = successor::host::produce(&target);
    let mut funded = successor::live_native::open(&target);
    publish_target(&target, 180311);
    successor::projection::reset();
    successor::output::reset_decodes();
    let (outcome, report) =
        successor::live_native::next(&target, &mut funded, &request_scope(), 8_000_000);
    assert!(matches!(
        outcome,
        WorthQueryApplicationLiveOutcome::Delivered(_)
    ));
    assert_eq!(successor::output::ignored_refusals(), 0);
    assert!(matches!(
        funded.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
    let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Raw(wire),
    ) = successor::output::DerivedBinding::encode(&successor::output::calculate(&journey::heavy()))
        .unwrap()
    else {
        panic!("canonical value is a string carrier")
    };
    let narrow = report
        .charged_work()
        .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
        .unwrap();
    let mut limited = successor::live_native::open(&target);
    publish_target(&target, 180312);
    successor::projection::reset();
    successor::output::reset_decodes();
    let (denied, _) = successor::live_native::next(&target, &mut limited, &request_scope(), narrow);
    assert!(matches!(
        denied,
        WorthQueryApplicationLiveOutcome::ProjectionDenied(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::Checkpoint(
                    worth_execution::MapKernelStop::WorkCeiling
                )
            )
        )
    ));
    assert_eq!(successor::projection::count(), 1);
    assert!(successor::output::ignored_refusals() > 0);
    assert!(matches!(
        successor::live_native::next(&target, &mut limited, &request_scope(), 8_000_000).0,
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(limited);
    for reject in [false, true] {
        let mut live = successor::live_native::open(&target);
        publish_target(&target, 180313 + u64::from(reject));
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(request_scope().deadline(), cancellation.token());
        let _guard = successor::projection::Guard::arm(cancellation, reject);
        let (denied, _) = successor::live_native::next(&target, &mut live, &scope, 8_000_000);
        assert_eq!(successor::projection::count(), 1);
        assert!(matches!(
            denied,
            WorthQueryApplicationLiveOutcome::Cancelled
                | WorthQueryApplicationLiveOutcome::ProjectionDenied(
                    WorthQueryApplicationProjectionDenialKind::ValueDecode(
                        WorthQueryValueDecodeDenial::Scope(
                            worth_execution::WorkCeilingDenial::Stopped(
                                worth_execution::MapKernelStop::Cancelled
                            )
                        )
                    )
                )
        ));
    }
    successor::projection::reset();
    let mut retry = successor::live_native::open(&target);
    publish_target(&target, 180315);
    assert!(matches!(
        successor::live_native::next(&target, &mut retry, &request_scope(), 8_000_000).0,
        WorthQueryApplicationLiveOutcome::Delivered(_)
    ));
    assert!(matches!(
        retry.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}
