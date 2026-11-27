//! Inspection-phase stops classify by their exact resource and owner cause:
//! a cancelled native read or an elapsed kernel deadline is interrupted, an
//! expired authentication asks for a renewed session, and only policy,
//! scope and disclosure refusals read as withheld evidence.
use worth_execution::{ExecutionRequestArrayDenial, MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_inspection::{
        DiagnosticComparisonDenial, WorthQueryApplicationInspectionOwnerRefusal as Owner,
        WorthQueryApplicationInspectionResourceRefusal as Resource,
        WorthQueryApplicationInspectionStage,
    },
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryOperationAuthorizationRefusalKind as Policy,
    },
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;

use crate::audit::{WorthQueryReferencedByDenial, WorthQuerySourceProvenanceDenial};

use super::super::inspection_owner::{owner, EVIDENCE_WITHHELD};
use super::super::operator_safety::assert_renders;
use super::super::summary::WorthQueryRefusalCategory as Category;
use super::{resource, FIELD_DETAIL_UNSUPPORTED};

#[test]
fn inspection_resource_stops_keep_their_interruption_capacity_and_storage_category() {
    let cancelled = Native::Cancelled;
    let inconsistent = Native::ContentInconsistency;
    let ceiling = WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling);
    let array =
        ExecutionRequestArrayDenial::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled));
    let interrupted = WorthQueryAdvancementDenial::Interrupted(
        WorthQueryManagedComputationInterruption::DeadlineExceeded,
    );
    for (stop, category) in [
        (Resource::Native(&cancelled), Category::Interrupted),
        (Resource::Native(&inconsistent), Category::Storage),
        (
            Resource::Kernel(&MapKernelStop::DeadlineElapsed),
            Category::Interrupted,
        ),
        (
            Resource::Kernel(&MapKernelStop::AccountingUnavailable),
            Category::Internal,
        ),
        (Resource::Request(&ceiling), Category::Capacity),
        (Resource::Array(&array), Category::Interrupted),
        (Resource::Advancement(&interrupted), Category::Interrupted),
    ] {
        assert_eq!(resource(stop).category(), category, "{stop:?}");
    }
}

#[test]
fn an_expired_authentication_asks_for_a_new_session_and_policy_stays_withheld() {
    let expired = Authorization::Refused(Policy::ExpiredAuthentication);
    let summary = owner(Owner::CurrentAuthorization(&expired));
    assert_eq!(summary.category(), Category::SessionExpired);
    assert!(summary.retryable());
    // A missing grant and an unknown scope both read as withheld evidence:
    // the refusal never says whether the target exists.
    for refused in [Policy::CapabilityGrantMissing, Policy::StaleScope] {
        let kind = Authorization::Refused(refused);
        let summary = owner(Owner::CurrentAuthorization(&kind));
        assert_eq!(summary, EVIDENCE_WITHHELD, "{refused:?}");
        assert!(!summary.retryable());
    }
    let cancelled = WorthQueryAdvancementDenial::Interrupted(
        WorthQueryManagedComputationInterruption::Cancelled,
    );
    assert_eq!(
        owner(Owner::Advancement(&cancelled)).category(),
        Category::Interrupted
    );
}

#[test]
fn unsupported_field_shapes_and_internal_faults_are_not_permission_refusals() {
    for shape in [
        Owner::DiagnosticContractUnsupportedShape,
        Owner::DiagnosticAdmission(&DiagnosticComparisonDenial::UnsupportedShape),
    ] {
        let summary = owner(shape);
        assert_eq!(summary, FIELD_DETAIL_UNSUPPORTED);
        assert_eq!(summary.category(), Category::Unsupported);
        assert!(!summary.retryable());
    }
    let stage = WorthQueryApplicationInspectionStage::SourceMaterialization;
    assert_eq!(
        owner(Owner::KernelPanic(stage)).category(),
        Category::Internal
    );
    assert_eq!(
        owner(Owner::StorageAccountingOverflow(stage)).category(),
        Category::Internal
    );
    assert_eq!(
        owner(Owner::KernelResultCapacityExceeded(stage)).category(),
        Category::Capacity
    );
    assert_eq!(owner(Owner::FacetDenied), EVIDENCE_WITHHELD);
}

#[test]
fn provenance_and_referenced_by_refusals_render_safely() {
    assert_renders(
        &WorthQuerySourceProvenanceDenial::ForeignBranch,
        Category::NotFound,
    );
    assert_renders(
        &WorthQueryReferencedByDenial::NotAnIncomingReference {
            query: "ForwardQuery".to_owned(),
        },
        Category::Internal,
    );
}
