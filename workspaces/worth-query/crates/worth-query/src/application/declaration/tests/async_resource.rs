use super::super::{
    WorthQueryAsyncDeclarationSupport, WorthQueryAsyncFailurePosture,
    WorthQueryAsyncLoadingPosture, WorthQueryAsyncRequestIdentityValue,
    WorthQueryAsyncSourceFamily, WorthQueryTemporalDeclarationClause, WorthQueryTemporalDuration,
};
use super::async_support::{
    resource_request, AsyncResourceReadDeclaration, AsyncResourceReadFamily,
    AsyncUnsupportedSplitEdgeDeclaration, DeferredAsyncResourceReadDeclaration,
    DeferredAsyncResourceReadFamily,
};
use super::support::{admitted_handle, GeometryOperatingContext};
use crate::application::WorthQueryDeclarationFamilyMarker;

#[test]
fn equivalent_async_authoring_forms_share_canonical_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let left = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("edge", "edge:42"), ("material", "mat:blue")],
            )],
        ))
        .expect("left async declaration should canonicalize");
    let right = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("material", "mat:blue"), ("edge", "edge:42")],
            )],
        ))
        .expect("right async declaration should canonicalize");

    assert_eq!(
        left.declaration_family_key(),
        AsyncResourceReadFamily::semantic_family_key()
    );
    assert_eq!(left.declaration_digest(), right.declaration_digest());
}

#[test]
fn async_source_family_changes_mutate_declaration_identity_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let bridge = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::FailClosed,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("bridge declaration should canonicalize");
    let host = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::HostResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::FailClosed,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("host declaration should canonicalize");

    assert_ne!(bridge.declaration_digest(), host.declaration_digest());
}

#[test]
fn async_request_identity_changes_mutate_declaration_identity_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let left = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::ExternalResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("left async declaration should canonicalize");
    let right = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::ExternalResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("edge", "edge:43")],
            )],
        ))
        .expect("right async declaration should canonicalize");

    assert_ne!(left.declaration_digest(), right.declaration_digest());
}

#[test]
fn async_loading_and_failure_posture_changes_mutate_declaration_identity_explicitly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let blocking = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::ExternalResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::FailClosed,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("blocking async declaration should canonicalize");
    let background = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::ExternalResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::FailClosed,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("background-refresh async declaration should canonicalize");
    let stale_retained = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::ExternalResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("stale-retained async declaration should canonicalize");

    assert_ne!(
        blocking.declaration_digest(),
        background.declaration_digest()
    );
    assert_ne!(
        blocking.declaration_digest(),
        stale_retained.declaration_digest()
    );
}

#[test]
fn async_clause_accessors_retain_normalized_request_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let declaration = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[
                    ("material", "mat:blue"),
                    ("edge", "edge:42"),
                    ("edge", "edge:42"),
                ],
            )],
        ))
        .expect("async declaration should canonicalize");

    let [clause] = declaration.async_resource_clauses() else {
        panic!("expected one normalized async clause");
    };
    match clause {
        crate::application::WorthQueryAsyncDeclarationClause::ResourceRequest {
            source_family,
            loading_posture,
            failure_posture,
            request_identity,
        } => {
            assert_eq!(*source_family, WorthQueryAsyncSourceFamily::BridgeResource);
            assert_eq!(
                *loading_posture,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh
            );
            assert_eq!(
                *failure_posture,
                WorthQueryAsyncFailurePosture::RetainStaleValue
            );
            assert_eq!(request_identity.len(), 2);
            assert_eq!(request_identity[0].key(), "edge");
            assert_eq!(
                request_identity[0].value(),
                &WorthQueryAsyncRequestIdentityValue::Text("edge:42".to_owned())
            );
            assert_eq!(request_identity[1].key(), "material");
            assert_eq!(
                request_identity[1].value(),
                &WorthQueryAsyncRequestIdentityValue::Text("mat:blue".to_owned())
            );
        }
        crate::application::WorthQueryAsyncDeclarationClause::CompletionRequest { .. } => {
            panic!("expected resource request clause");
        }
    }
}

#[test]
fn async_and_temporal_clauses_compose_into_canonical_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let plain_async = handle
        .declare(AsyncResourceReadDeclaration::new(
            "edge:42",
            vec![resource_request(
                WorthQueryAsyncSourceFamily::BridgeResource,
                WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                WorthQueryAsyncFailurePosture::RetainStaleValue,
                &[("edge", "edge:42")],
            )],
        ))
        .expect("plain async declaration should canonicalize");
    let temporal_async = handle
        .declare(
            AsyncResourceReadDeclaration::new(
                "edge:42",
                vec![resource_request(
                    WorthQueryAsyncSourceFamily::BridgeResource,
                    WorthQueryAsyncLoadingPosture::BackgroundRefresh,
                    WorthQueryAsyncFailurePosture::RetainStaleValue,
                    &[("edge", "edge:42")],
                )],
            )
            .with_temporal(vec![WorthQueryTemporalDeclarationClause::stale_after(
                WorthQueryTemporalDuration::seconds(30),
            )]),
        )
        .expect("temporal async declaration should canonicalize");

    assert_ne!(
        plain_async.declaration_digest(),
        temporal_async.declaration_digest()
    );
}

#[test]
fn async_clauses_fail_closed_without_family_opt_in() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());

    match handle.declare_checked(AsyncUnsupportedSplitEdgeDeclaration::new(
        "edge:42",
        vec![resource_request(
            WorthQueryAsyncSourceFamily::BridgeResource,
            WorthQueryAsyncLoadingPosture::Blocking,
            WorthQueryAsyncFailurePosture::FailClosed,
            &[("edge", "edge:42")],
        )],
    )) {
        crate::application::WorthQueryDeclaredFamilyChecked::AsyncUnsupported(denial) => {
            assert_eq!(
                denial.async_support(),
                WorthQueryAsyncDeclarationSupport::Unsupported
            );
            assert_eq!(
                denial.support_report().declaration_family_key(),
                "split-edge"
            );
        }
        other => panic!(
            "expected async unsupported denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn async_declaration_families_can_fail_closed_as_deferred_debt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());

    match handle.declare_checked(DeferredAsyncResourceReadDeclaration::new(
        "edge:42",
        vec![resource_request(
            WorthQueryAsyncSourceFamily::BridgeResource,
            WorthQueryAsyncLoadingPosture::BackgroundRefresh,
            WorthQueryAsyncFailurePosture::RetainStaleValue,
            &[("edge", "edge:42")],
        )],
    )) {
        crate::application::WorthQueryDeclaredFamilyChecked::AsyncDeferred(denial) => {
            assert_eq!(
                denial.async_support(),
                WorthQueryAsyncDeclarationSupport::DeferredDebt
            );
            assert_eq!(
                denial.support_report().declaration_family_key(),
                DeferredAsyncResourceReadFamily::semantic_family_key()
            );
        }
        other => panic!(
            "expected async deferred denial, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}
