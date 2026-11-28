//! Nested request stops inside facade refusals keep their own meaning: a
//! cancellation or deadline reads as interrupted, a storage fault as storage,
//! never as a permission or capacity problem.
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryProviderSessionControlStopKind,
    WorthQueryValueDecodeDenial,
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestQueryDenial,
    WorthQueryRefusalCategory as Category, WorthQueryTypedLiveRefreshDenial,
    WorthQueryUserFacingRefusal,
};
use worth_relational::facade::content::RelationalContentDenial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
fn assert_category(refusal: &dyn WorthQueryUserFacingRefusal, category: Category) {
    let message = refusal.user_message();
    assert_eq!(refusal.refusal_category(), category, "{message}");
    assert!(
        !message.contains("::") && !message.contains('('),
        "{message}"
    );
}

#[test]
fn nested_cancellation_deadline_and_storage_stops_keep_their_category() {
    let resolution = |cause| {
        WorthQueryApplicationRequestQueryDenial::ScopeResolution(
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(cause),
                "Task",
            ),
        )
    };
    assert_category(
        &resolution(RelationalExecutionDenialCause::Cancelled),
        Category::Interrupted,
    );
    let corrupt = resolution(RelationalExecutionDenialCause::ContentInconsistency);
    assert_category(&corrupt, Category::Storage);
    assert!(!corrupt.retryable());
    assert_category(
        &resolution(RelationalExecutionDenialCause::WorkerFailed),
        Category::Internal,
    );

    let stopped_authorization = WorthQueryApplicationRequestQueryDenial::Admission(
        WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                    WorthQueryProviderSessionControlStopKind::Cancelled,
                )),
            ),
            "ReadTaskDetails",
        ),
    );
    assert_category(&stopped_authorization, Category::Interrupted);
    assert!(stopped_authorization.retryable());

    let live_authorization = WorthQueryTypedLiveRefreshDenial::AuthorizationDenied(Box::new(
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                WorthQueryProviderSessionControlStopKind::TimedOut,
            )),
            "ReadTaskDetails",
        ),
    ));
    assert_category(&live_authorization, Category::Interrupted);
    let decode_stop = WorthQueryTypedLiveRefreshDenial::ProjectionDenied(
        WorthQueryApplicationProjectionDenialKind::ValueDecode(
            WorthQueryValueDecodeDenial::Content(RelationalContentDenial::Execution(
                RelationalExecutionDenialCause::Cancelled,
            )),
        ),
    );
    assert_category(&decode_stop, Category::Interrupted);
}
