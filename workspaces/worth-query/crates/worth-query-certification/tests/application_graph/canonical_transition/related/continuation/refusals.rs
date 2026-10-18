//! Refusal and cancellation veto actual changed-codec continuation handoff.
use super::{super::*, native};
use crate::document_retention_model::operator_identity::request_scope;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    declaration::application_schema::ApplicationScalarValueBinding,
    primary_graph::*,
};
#[test]
fn changed_codec_continuation_swallowed_refusal_and_original_cancellation_never_return_successors()
{
    let original = host::install(journey::heavy()).unwrap();
    let _settled = host::produce(&original);
    // All tokens are issued before adoption by real first pages, never copied.
    let token = journey::page(&original).into_parts().1.unwrap();
    let narrow_token = journey::page(&original).into_parts().1.unwrap();
    let retry_token = journey::page(&original).into_parts().1.unwrap();
    let cancellation_tokens = [
        journey::page(&original).into_parts().1.unwrap(),
        journey::page(&original).into_parts().1.unwrap(),
    ];
    let final_retry = journey::page(&original).into_parts().1.unwrap();
    let checkpoint = host::capture(&original);
    let target = journey::successor(checkpoint);
    assert_eq!(successor::output::decodes(), 0);
    let _target_settled = successor::host::produce(&target);
    projection::reset();
    output::reset_decodes();
    let (funded, report) = native::resume(&original, token, &request_scope(), 8_000_000);
    assert!(funded.unwrap().continuation().is_none());
    assert_eq!(output::ignored_refusals(), 0);
    let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Raw(wire),
    ) = output::DerivedBinding::encode(&output::calculate(&journey::heavy())).unwrap()
    else {
        panic!("canonical carrier is a string")
    };
    let narrow = report
        .charged_work()
        .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
        .unwrap();
    projection::reset();
    output::reset_decodes();
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(request_scope().deadline(), cancellation.token());
        let _guard = projection::Guard::arm(cancellation, true);
        let (denied, _) = native::resume(&original, narrow_token, &scope, narrow);
        let denied = denied
            .err()
            .expect("swallowed refusal cannot return rows or a token");
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationContinuationDenialKind::Projection(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Checkpoint(
                        worth_execution::MapKernelStop::WorkCeiling
                    )
                )
            )
        );
        assert_eq!(projection::count(), 1);
        assert!(output::ignored_refusals() > 0);
    }
    projection::reset();
    output::reset_decodes();
    let (retried, _) = native::resume(&original, retry_token, &request_scope(), 8_000_000);
    assert!(retried.unwrap().continuation().is_none());
    assert_eq!(output::ignored_refusals(), 0);
    for (reject, token) in [false, true].into_iter().zip(cancellation_tokens) {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(request_scope().deadline(), cancellation.token());
        let _guard = projection::Guard::arm(cancellation, reject);
        let (denied, _) = native::resume(&original, token, &scope, 8_000_000);
        let denied = denied
            .err()
            .expect("late original cancellation overrides success AND domain rejection");
        assert_eq!(projection::count(), 1);
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationContinuationDenialKind::Projection(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            )
        );
    }
    projection::reset();
    assert!(
        native::resume(&original, final_retry, &request_scope(), 8_000_000)
            .0
            .is_ok()
    );
}
