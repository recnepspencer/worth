use crate::facade::foundation::{
    execute_preflight_bundle, resolve_snapshot_basis, AspectFieldSelector,
    AuthoredResultShapeField, BasisAuthorityFamily, BasisPreflightError, BasisResolutionError,
    BasisResolutionMode, ExecutionBasisIntent, GuidedAuthoringPath, ResolvedSnapshotIdentity,
    RootEntityKey, SnapshotLineageClass,
};
use crate::facade::policy::{
    plan_validated_bundle, planning_request_context_for_direct, PlanningError,
};
use crate::facade::runtime::validate_canonical_bundle;

use super::super::planning_matrix::{
    PlanningCertificationBundle, PlanningCertificationRow, PlanningHostileExpectation,
    PlanningPerturbationClass, PlanningRejectionBundle, PlanningRejectionRow,
};
use super::super::profiles::CertificationProfile;
pub(super) fn to_bundle(
    profile: CertificationProfile,
    preflight: &crate::facade::foundation::ExecutionPreflightBundle,
) -> PlanningCertificationBundle {
    let envelope = execute_preflight_bundle(preflight).unwrap();
    PlanningCertificationBundle {
        profile,
        query_digest: preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        plan_digest: preflight.plan().query().plan_digest().as_str().to_string(),
        result_digest: envelope.report().result_digest().as_str().to_string(),
        basis_digest: preflight.basis().proof().digest().as_str().to_string(),
        counter_snapshot: envelope.counters().clone(),
    }
}

pub(super) fn to_rejection_bundle(
    profile: CertificationProfile,
    failure_class: &str,
    failure_digest: String,
) -> PlanningRejectionBundle {
    PlanningRejectionBundle {
        profile,
        failure_class: failure_class.to_string(),
        failure_digest,
    }
}

pub(super) fn rejection_row(
    row_name: &'static str,
    perturbation_class: PlanningPerturbationClass,
    control: &crate::facade::foundation::ExecutionPreflightBundle,
    hostile: Result<(), impl RejectionDigest>,
) -> PlanningRejectionRow {
    let hostile = hostile.unwrap_err();
    PlanningRejectionRow {
        row_name,
        perturbation_class,
        control_lane: to_bundle(CertificationProfile::DirectConstruction, control),
        hostile_lane: to_rejection_bundle(
            CertificationProfile::BindingVariation,
            hostile.failure_class_name(),
            hostile.failure_digest(),
        ),
        parity_lane: to_bundle(CertificationProfile::ReplayParity, control),
    }
}

pub trait RejectionDigest {
    fn failure_class_name(&self) -> &'static str;
    fn failure_digest(&self) -> String;
}

impl RejectionDigest for PlanningError {
    fn failure_class_name(&self) -> &'static str {
        match self {
            PlanningError::MissingBindingResolutionForIdentityBoundQuery => {
                "incomplete-planning-inputs"
            }
            PlanningError::UnsupportedBackendParityRequest => "unsupported-backend-route",
            PlanningError::UnsupportedFallbackShape => "unsupported-fallback-shape",
            PlanningError::UnsupportedOrderingFamily => "unsupported-ordering-family",
            PlanningError::UnsupportedCursorShape => "unstable-cursor-shape",
            PlanningError::UnsupportedTraversalBound => "unsupported-traversal-bound",
            PlanningError::UnsupportedAggregateFamily => "unsupported-aggregate-family",
            PlanningError::UnsupportedCollectionResultFamily => "unsupported-cdc-result-family",
            PlanningError::BindingResolutionFailed { .. } => "incomplete-planning-inputs",
            PlanningError::PlanningInvariantViolation { .. } => "internal-invariant-break",
        }
    }

    fn failure_digest(&self) -> String {
        match self {
            PlanningError::MissingBindingResolutionForIdentityBoundQuery => {
                "missing-binding-resolution".to_string()
            }
            PlanningError::UnsupportedBackendParityRequest => {
                "unsupported-backend-route".to_string()
            }
            PlanningError::UnsupportedFallbackShape => "unsupported-fallback-shape".to_string(),
            PlanningError::UnsupportedOrderingFamily => "unsupported-ordering-family".to_string(),
            PlanningError::UnsupportedCursorShape => "unstable-cursor-shape".to_string(),
            PlanningError::UnsupportedTraversalBound => "unsupported-traversal-bound".to_string(),
            PlanningError::UnsupportedAggregateFamily => "unsupported-aggregate-family".to_string(),
            PlanningError::UnsupportedCollectionResultFamily => {
                "unsupported-cdc-result-family".to_string()
            }
            PlanningError::BindingResolutionFailed { failure_digest } => failure_digest.clone(),
            PlanningError::PlanningInvariantViolation { message } => {
                format!("planning-invariant:{message}")
            }
        }
    }
}

impl RejectionDigest for BasisPreflightError {
    fn failure_class_name(&self) -> &'static str {
        match self {
            BasisPreflightError::BasisIntentMismatch => "basis-intent-mismatch",
            BasisPreflightError::PlannedRouteBasisMismatch => "planned-route-basis-mismatch",
        }
    }

    fn failure_digest(&self) -> String {
        match self {
            BasisPreflightError::BasisIntentMismatch => "basis-intent-mismatch".to_string(),
            BasisPreflightError::PlannedRouteBasisMismatch => {
                "planned-route-basis-mismatch".to_string()
            }
        }
    }
}

impl RejectionDigest for BasisResolutionError {
    fn failure_class_name(&self) -> &'static str {
        match self {
            BasisResolutionError::EvidenceIdentityDerivationDenied(_) => {
                "evidence-identity-derivation-denied"
            }
            BasisResolutionError::UnsupportedBasisKind => "unsupported-basis-kind",
            BasisResolutionError::ResolutionIdentityMismatch => "snapshot-basis-resolution-failure",
        }
    }

    fn failure_digest(&self) -> String {
        match self {
            BasisResolutionError::EvidenceIdentityDerivationDenied(cause) => {
                format!("evidence-identity-derivation-denied:{cause:?}")
            }
            BasisResolutionError::UnsupportedBasisKind => "unsupported-basis-kind".to_string(),
            BasisResolutionError::ResolutionIdentityMismatch => {
                "snapshot-basis-resolution-failure".to_string()
            }
        }
    }
}

pub(super) fn binding_conflict_hostile() -> Result<(), PlanningError> {
    let query =
        crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap())
            .build()
            .unwrap();
    let shape = crate::authoring::RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let bindings = crate::facade::foundation::QueryBindingDescriptor::new().with_identity(
        crate::facade::foundation::IdentityBindingDescriptor::new(
            crate::facade::foundation::QueryBindingSlot::new("root").unwrap(),
            crate::facade::foundation::QueryBindingSubject::RootEntity,
        ),
    );
    let request = GuidedAuthoringPath::pair_detail_with_bindings(query, shape, bindings).unwrap();
    let canonical = crate::facade::foundation::canonicalize_request(request).unwrap();
    let validated = validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap();
    crate::facade::policy::planning_request_context_for_bound(
        &validated,
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            false,
        ),
        crate::facade::foundation::BoundBindings::new(vec![
            crate::facade::foundation::BoundBinding::new(
                crate::facade::foundation::QueryBindingSlot::new("root").unwrap(),
                crate::facade::foundation::QueryBindingSubject::TraversalRoot,
                "user-1",
            ),
        ]),
        Vec::new(),
    )
    .map(|_| ())
}

pub(super) fn unsupported_backend_route_hostile() -> Result<(), PlanningError> {
    let query =
        crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap())
            .build()
            .unwrap();
    let shape = crate::authoring::RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let request = GuidedAuthoringPath::pair_detail(query, shape).unwrap();
    let canonical = crate::facade::foundation::canonicalize_request(request).unwrap();
    let validated = validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap();
    let request = planning_request_context_for_direct(
        &validated,
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Store,
            SnapshotLineageClass::CurrentHead,
            false,
        ),
    )
    .unwrap();
    plan_validated_bundle(&validated, request).map(|_| ())
}

pub(super) fn unsupported_fallback_shape_hostile() -> Result<(), PlanningError> {
    let query =
        crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap())
            .build()
            .unwrap();
    let shape = crate::authoring::RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let request = GuidedAuthoringPath::pair_detail(query, shape).unwrap();
    let canonical = crate::facade::foundation::canonicalize_request(request).unwrap();
    let validated = validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap();
    let request = planning_request_context_for_direct(
        &validated,
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            true,
        ),
    )
    .unwrap();
    plan_validated_bundle(&validated, request).map(|_| ())
}

pub(super) fn snapshot_basis_resolution_failure_hostile() -> Result<(), BasisResolutionError> {
    let query =
        crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap())
            .build()
            .unwrap();
    let shape = crate::authoring::RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let request = GuidedAuthoringPath::pair_detail(query, shape).unwrap();
    let canonical = crate::facade::foundation::canonicalize_request(request).unwrap();
    let validated = validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap();
    resolve_snapshot_basis(
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            false,
        ),
        ResolvedSnapshotIdentity::new(
            BasisAuthorityFamily::Store,
            Some("workspace-main".to_string()),
            crate::memory_workspace::admit_external_snapshot_label("snapshot-2")?
                .evidence_identity(),
            validated.query().schema_basis().clone(),
            SnapshotLineageClass::CurrentHead,
        ),
        BasisResolutionMode::StoreDirect,
    )
    .map(|_| ())
}

pub(super) fn canonical_row(
    row_name: &'static str,
    perturbation_class: PlanningPerturbationClass,
    hostile_expectation: PlanningHostileExpectation,
    control: crate::facade::foundation::ExecutionPreflightBundle,
    hostile: crate::facade::foundation::ExecutionPreflightBundle,
    parity: crate::facade::foundation::ExecutionPreflightBundle,
) -> PlanningCertificationRow {
    PlanningCertificationRow {
        row_name,
        perturbation_class,
        hostile_expectation,
        parity_anchor: crate::harness::certification::ParityAnchor::Control,
        control_lane: to_bundle(CertificationProfile::DirectConstruction, &control),
        hostile_lane: to_bundle(CertificationProfile::BindingVariation, &hostile),
        parity_lane: to_bundle(CertificationProfile::ReplayParity, &parity),
    }
}
