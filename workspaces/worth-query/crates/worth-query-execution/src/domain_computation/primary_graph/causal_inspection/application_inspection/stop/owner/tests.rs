//! Literal owner causes and pointer identity detect lossy public projections.
use super::*;
use crate::domain_computation::{
    authorization::WorthQueryOperationAuthorizationRefusalKind as Authorization,
    primary_graph::{
        WorthQueryApplicationQueryAdmissionDenialKind as Query,
        WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Entity,
    },
};

fn denial(cause: InspectionAdmissionDenial) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(InspectionCause::Admission(cause), None)
}

#[test]
fn inspection_preserves_query_authorization_cause() {
    let query = WorthQueryApplicationQueryAdmissionDenial::new(Query::StaleScope, "query scope");
    let refusal = denial(InspectionAdmissionDenial::Authorization(query.clone()));
    let Some(Refusal::Authorization(actual)) = refusal.owner_refusal() else {
        panic!("query admission cause must survive inspection");
    };
    assert_eq!(actual, &query);
    let InspectionCause::Admission(InspectionAdmissionDenial::Authorization(original)) =
        &refusal.cause
    else {
        unreachable!()
    };
    assert!(std::ptr::eq(actual, original));
}

#[test]
fn inspection_preserves_current_authorization_cause() {
    let current =
        WorthQueryOperationAuthorizationDenialKind::Refused(Authorization::StaleAuthorization);
    let refusal = denial(InspectionAdmissionDenial::CurrentAuthorization(current));
    assert!(
        matches!(refusal.owner_refusal(), Some(Refusal::CurrentAuthorization(actual)) if *actual == current)
    );
}

#[test]
fn inspection_preserves_capability_cause() {
    let capability = WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(Authorization::PermissionDenied),
        "inspection capability",
    );
    let refusal = denial(InspectionAdmissionDenial::Capability(capability.clone()));
    let Some(Refusal::Capability(actual)) = refusal.owner_refusal() else {
        panic!("full capability cause must survive inspection");
    };
    assert_eq!(actual, &capability);
    let InspectionCause::Admission(InspectionAdmissionDenial::Capability(original)) =
        &refusal.cause
    else {
        unreachable!()
    };
    assert!(std::ptr::eq(actual, original));
}

#[test]
fn inspection_preserves_scope_cause() {
    let scope = WorthQueryEntityResolutionDenial::new(
        Resolution::Refused(Entity::AmbiguousEntity),
        "scope identity",
    );
    let refusal = denial(InspectionAdmissionDenial::Scope(scope.clone()));
    assert!(matches!(refusal.owner_refusal(), Some(Refusal::Scope(actual)) if *actual == scope));
}

#[test]
fn inspection_preserves_security_cause() {
    let security = crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
        crate::basis::WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch,
    );
    let refusal = denial(InspectionAdmissionDenial::Security(security));
    assert!(
        matches!(refusal.owner_refusal(), Some(Refusal::Security(actual)) if *actual == security)
    );
}

#[test]
fn inspection_preserves_governance_cause() {
    governance(false);
}

#[test]
fn inspection_preserves_diagnostic_preparation_governance_cause() {
    governance(true);
}

fn governance(diagnostic: bool) {
    for cause in [
        WorthQueryApplicationQueryGovernanceDenialKind::Required,
        WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch,
        WorthQueryApplicationQueryGovernanceDenialKind::InternalComputationDenied,
    ] {
        let original = if diagnostic {
            InspectionAdmissionDenial::DiagnosticPreparation(CaptureStop::Kernel(
                ScopeStop::Failure(MapKernelFailure::Domain(
                    DiagnosticPreparationDenial::Governance(cause),
                )),
            ))
        } else {
            InspectionAdmissionDenial::Governance(cause)
        };
        let refusal = denial(original);
        assert!(
            matches!(refusal.owner_refusal(), Some(Refusal::Governance(actual)) if *actual == cause)
        );
        assert!(refusal.resource_refusal().is_none());
        assert_eq!(
            refusal.kind(),
            WorthQueryApplicationInspectionStopKind::Admission
        );
    }
}

#[test]
fn unrelated_stops_do_not_project_owner_causes() {
    for original in [
        InspectionAdmissionDenial::ForeignIssuedAccess,
        InspectionAdmissionDenial::DiagnosticPreparation(CaptureStop::Kernel(ScopeStop::Failure(
            MapKernelFailure::Panic,
        ))),
    ] {
        assert!(denial(original).owner_refusal().is_none());
    }
}

#[test]
fn inspection_preserves_outer_query_cause() {
    let query = WorthQueryApplicationQueryAdmissionDenial::new(Query::ForeignScope, "outer query");
    let refusal =
        WorthQueryApplicationInspectionDenial::new(InspectionCause::Query(query.clone()), None);
    assert!(
        matches!(refusal.owner_refusal(), Some(Refusal::Authorization(actual)) if *actual == query)
    );
}

#[test]
fn inspection_preserves_outer_scope_cause() {
    let scope = WorthQueryEntityResolutionDenial::new(
        Resolution::Refused(Entity::CorruptIdentityIndex),
        "outer scope",
    );
    let refusal =
        WorthQueryApplicationInspectionDenial::new(InspectionCause::Scope(scope.clone()), None);
    assert!(matches!(refusal.owner_refusal(), Some(Refusal::Scope(actual)) if *actual == scope));
}

#[test]
fn inspection_preserves_outer_selection_cause() {
    let selection = crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
        crate::basis::WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner,
    );
    let refusal =
        WorthQueryApplicationInspectionDenial::new(InspectionCause::Selection(selection), None);
    assert!(
        matches!(refusal.owner_refusal(), Some(Refusal::Security(actual)) if *actual == selection)
    );
}
