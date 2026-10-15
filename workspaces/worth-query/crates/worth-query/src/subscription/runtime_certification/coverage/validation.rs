use crate::subscription::bridge_parity::QuerySubscriptionBridgeParityExplanation;
use crate::subscription::certification::SubscriptionLifecycleCertificationBundle;
use crate::subscription::diagnostic::{
    QuerySubscriptionDeniedDiagnosticBundle, QuerySubscriptionDiagnosticStage,
};
use crate::subscription::evidence_identities::typed_identity_drift;
use crate::subscription::family::QuerySubscriptionFamily;
use crate::subscription::support::{
    QuerySubscriptionSupportClass, QuerySubscriptionSupportPosture, QuerySubscriptionSupportReport,
};
use crate::subscription::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
};

use super::super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
    QuerySubscriptionRuntimeCertificationErrorKind,
};

pub(super) fn validate_hostile_diagnostic_alignment(
    diagnostic: &QuerySubscriptionDeniedDiagnosticBundle,
    support: &QuerySubscriptionSupportReport,
    lifecycle: &SubscriptionLifecycleCertificationBundle,
) -> Result<(), QuerySubscriptionRuntimeCertificationError> {
    if let Some(support_report_identity) = diagnostic.support_report_identity()
        && typed_identity_drift(support_report_identity, support.report_identity())
    {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
                QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
                "hostile family coverage rows require denied diagnostic bundles to preserve support-report identity when support evidence is present",
                &[
                    validation_role_evidence_identity("diagnostic_support", support_report_identity),
                    validation_role_evidence_identity("support", support.report_identity()),
                    validation_role_evidence_identity("diagnostic", diagnostic.bundle_identity()),
                ],
                QuerySubscriptionRuntimeCertificationCounters::default(),
            ));
    }

    validate_trace_stage_source(
        diagnostic,
        QuerySubscriptionDiagnosticStage::Declaration,
        lifecycle.subscription_declaration_identity(),
        "hostile family coverage rows require denied diagnostic traces to preserve declaration identity",
    )?;
    validate_trace_stage_source(
        diagnostic,
        QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
        lifecycle.bridge_declaration_identity(),
        "hostile family coverage rows require denied diagnostic traces to preserve bridge declaration identity",
    )?;
    validate_trace_stage_source(
        diagnostic,
        QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
        lifecycle.admission_identity(),
        "hostile family coverage rows require denied diagnostic traces to preserve runtime admission identity",
    )?;

    Ok(())
}

fn validate_trace_stage_source(
    diagnostic: &QuerySubscriptionDeniedDiagnosticBundle,
    stage: QuerySubscriptionDiagnosticStage,
    expected_source_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
    message: &'static str,
) -> Result<(), QuerySubscriptionRuntimeCertificationError> {
    let Some(stage_trace) = diagnostic
        .trace()
        .stage_traces()
        .iter()
        .find(|stage_trace| stage_trace.stage() == &stage)
    else {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
            message,
            &[
                validation_role_evidence_identity("diagnostic", diagnostic.bundle_identity()),
                validation_shape_role_evidence_identity("missing_stage", stage.as_str()),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    };

    if typed_identity_drift(stage_trace.source_identity(), expected_source_identity) {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
            message,
            &[
                validation_role_evidence_identity("diagnostic", diagnostic.bundle_identity()),
                validation_shape_role_evidence_identity("stage", stage.as_str()),
                validation_role_evidence_identity("trace_source", stage_trace.source_identity()),
                validation_role_evidence_identity("expected_source", expected_source_identity),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    Ok(())
}

pub(super) fn validate_row_alignment(
    family: &QuerySubscriptionFamily,
    support: &QuerySubscriptionSupportReport,
    parity: &QuerySubscriptionBridgeParityExplanation,
    lifecycle: &SubscriptionLifecycleCertificationBundle,
) -> Result<(), QuerySubscriptionRuntimeCertificationError> {
    if !matches!(
        support.support_subject().support_class(),
        QuerySubscriptionSupportClass::ActiveLifecycle
            | QuerySubscriptionSupportClass::Continuation
            | QuerySubscriptionSupportClass::PreviewCloseout
    ) {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CertificationSupportClassDenied,
            "runtime family certification requires support reports from runtime-backed lifecycle, continuation, or preview-closeout phases",
            &[
                validation_shape_role_evidence_identity(
                    "support_class",
                    support.support_subject().support_class().as_str(),
                ),
                validation_role_evidence_identity("support_report", support.report_identity()),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if support.support_posture() != &QuerySubscriptionSupportPosture::RuntimeBackedCertified {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CertificationSupportPostureDenied,
            "runtime family certification requires support reports whose posture is runtime-backed certified",
            &[
                validation_shape_role_evidence_identity(
                    "support_posture",
                    support.support_posture().as_str(),
                ),
                validation_role_evidence_identity("support_report", support.report_identity()),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if support.support_subject().family() != family
        || parity.query_family_label() != family.as_str()
    {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CoverageFamilyMismatch,
            "runtime family coverage rows require support and bridge parity artifacts for the same query subscription family",
            &[
                validation_shape_role_evidence_identity("expected_family", family.as_str()),
                validation_shape_role_evidence_identity(
                    "support_family",
                    support.support_subject().family().as_str(),
                ),
                validation_shape_role_evidence_identity(
                    "parity_family",
                    parity.query_family_label(),
                ),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if typed_identity_drift(
        support.support_subject().declaration_identity(),
        lifecycle.subscription_declaration_identity(),
    ) || typed_identity_drift(
        parity.comparison().query_declaration_identity(),
        lifecycle.subscription_declaration_identity(),
    ) || typed_identity_drift(
        parity.comparison().bridge_declaration_identity(),
        lifecycle.bridge_declaration_identity(),
    ) {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
            "runtime family coverage rows require support, parity, and lifecycle certification to preserve canonical declaration and bridge identity",
            &[
                validation_role_evidence_identity(
                    "support_declaration",
                    support.support_subject().declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle_declaration",
                    lifecycle.subscription_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "parity_declaration",
                    parity.comparison().query_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "parity_bridge",
                    parity.comparison().bridge_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle_bridge",
                    lifecycle.bridge_declaration_identity(),
                ),
            ],
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    Ok(())
}
