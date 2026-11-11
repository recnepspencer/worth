use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::bridge_parity::QuerySubscriptionBridgeParityExplanation;
use super::super::certification::SubscriptionLifecycleCertificationBundle;
use super::super::diagnostic::QuerySubscriptionAdmittedDiagnosticBundle;
use super::super::evidence_identities::typed_identity_drift;
use super::super::family::QuerySubscriptionFamily;
use super::super::support::QuerySubscriptionSupportReport;
use super::super::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
};
use super::coverage::CertifiedFamilyCoverageHandle;
use super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
    QuerySubscriptionRuntimeCertificationErrorKind,
};
use super::identities::runtime_certification_counter_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionRuntimeCertificationScope {
    family: QuerySubscriptionFamily,
    support_report: QuerySubscriptionSupportReport,
    bridge_parity: QuerySubscriptionBridgeParityExplanation,
    admitted_diagnostic_bundle: QuerySubscriptionAdmittedDiagnosticBundle,
    lifecycle_certification: SubscriptionLifecycleCertificationBundle,
    coverage_handle: CertifiedFamilyCoverageHandle,
    scope_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionRuntimeCertificationCounters,
}

impl QuerySubscriptionRuntimeCertificationScope {
    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn support_report(&self) -> &QuerySubscriptionSupportReport {
        &self.support_report
    }

    pub fn bridge_parity(&self) -> &QuerySubscriptionBridgeParityExplanation {
        &self.bridge_parity
    }

    pub fn admitted_diagnostic_bundle(&self) -> &QuerySubscriptionAdmittedDiagnosticBundle {
        &self.admitted_diagnostic_bundle
    }

    pub fn lifecycle_certification(&self) -> &SubscriptionLifecycleCertificationBundle {
        &self.lifecycle_certification
    }

    pub fn coverage_handle(&self) -> &CertifiedFamilyCoverageHandle {
        &self.coverage_handle
    }

    pub(crate) fn scope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scope_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionRuntimeCertificationCounters {
        &self.counters
    }
}

pub fn build_query_subscription_runtime_certification_scope(
    support_report: QuerySubscriptionSupportReport,
    bridge_parity: QuerySubscriptionBridgeParityExplanation,
    admitted_diagnostic_bundle: QuerySubscriptionAdmittedDiagnosticBundle,
    lifecycle_certification: SubscriptionLifecycleCertificationBundle,
    coverage_handle: CertifiedFamilyCoverageHandle,
) -> Result<QuerySubscriptionRuntimeCertificationScope, QuerySubscriptionRuntimeCertificationError>
{
    let family = support_report.support_subject().family().clone();
    if bridge_parity.query_family_label() != family.as_str()
        || admitted_diagnostic_bundle
            .semantic_labels()
            .query_family_label()
            != family.as_str()
        || coverage_handle.family() != &family
    {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::ScopeFamilyMismatch,
            "runtime certification scope requires support, parity, diagnostic, and coverage artifacts for the same query subscription family",
            (|| { Ok(vec![
                validation_shape_role_evidence_identity(
                    "support_family",
                    support_report.support_subject().family().as_str(),
                )?,
                validation_shape_role_evidence_identity(
                    "parity_family",
                    bridge_parity.query_family_label(),
                )?,
                validation_shape_role_evidence_identity(
                    "diagnostic_family",
                    admitted_diagnostic_bundle
                        .semantic_labels()
                        .query_family_label(),
                )?,
                validation_shape_role_evidence_identity(
                    "coverage_family",
                    coverage_handle.family().as_str(),
                )?,
            ]) })(),
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if support_report.support_posture()
        != &super::super::support::QuerySubscriptionSupportPosture::RuntimeBackedCertified
    {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CertificationSupportPostureDenied,
            "runtime certification scope requires runtime-backed certified support reports",
            (|| {
                Ok(vec![
                    validation_shape_role_evidence_identity(
                        "support_posture",
                        support_report.support_posture().as_str(),
                    )?,
                    validation_role_evidence_identity(
                        "support_report",
                        support_report.report_identity(),
                    ),
                ])
            })(),
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if typed_identity_drift(
        support_report.support_subject().declaration_identity(),
        lifecycle_certification.subscription_declaration_identity(),
    ) || typed_identity_drift(
        bridge_parity.witness().query_declaration_identity(),
        lifecycle_certification.subscription_declaration_identity(),
    ) || typed_identity_drift(
        bridge_parity.witness().bridge_declaration_identity(),
        lifecycle_certification.bridge_declaration_identity(),
    ) || typed_identity_drift(
        admitted_diagnostic_bundle.support_report_identity(),
        support_report.report_identity(),
    ) || typed_identity_drift(
        admitted_diagnostic_bundle.lifecycle_certification_identity(),
        lifecycle_certification.certification_bundle_identity(),
    ) {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
            "runtime certification scope requires aligned support, parity, diagnostic, and lifecycle digests",
            Ok(vec![
                validation_role_evidence_identity(
                    "support_declaration",
                    support_report.support_subject().declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle_declaration",
                    lifecycle_certification.subscription_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "parity_declaration",
                    bridge_parity.comparison().query_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "parity_bridge",
                    bridge_parity.comparison().bridge_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle_bridge",
                    lifecycle_certification.bridge_declaration_identity(),
                ),
                validation_role_evidence_identity(
                    "diagnostic_support",
                    admitted_diagnostic_bundle.support_report_identity(),
                ),
                validation_role_evidence_identity("support", support_report.report_identity()),
                validation_role_evidence_identity(
                    "diagnostic_lifecycle",
                    admitted_diagnostic_bundle.lifecycle_certification_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle",
                    lifecycle_certification.certification_bundle_identity(),
                ),
            ]),
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    if !coverage_handle.admitted_rows().iter().any(|row| {
        !typed_identity_drift(
            row.support_report_identity(),
            support_report.report_identity(),
        ) && row.bridge_parity_identity() == bridge_parity.explanation_identity()
            && !typed_identity_drift(
                row.lifecycle_certification_identity(),
                lifecycle_certification.certification_bundle_identity(),
            )
            && !typed_identity_drift(
                row.diagnostic_bundle_identity(),
                admitted_diagnostic_bundle.bundle_identity(),
            )
    }) {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CoverageScopeMissingAdmittedRow,
            "runtime certification scope requires the indexed family coverage handle to contain the scope's admitted support, parity, lifecycle, and diagnostic row",
            Ok(vec![
                validation_role_evidence_identity("support", support_report.report_identity()),
                validation_role_evidence_identity(
                    "parity",
                    bridge_parity.explanation_identity(),
                ),
                validation_role_evidence_identity(
                    "lifecycle",
                    lifecycle_certification.certification_bundle_identity(),
                ),
                validation_role_evidence_identity(
                    "diagnostic",
                    admitted_diagnostic_bundle.bundle_identity(),
                ),
                validation_role_evidence_identity(
                    "coverage",
                    coverage_handle.family_coverage_identity(),
                ),
            ]),
            QuerySubscriptionRuntimeCertificationCounters::default(),
        ));
    }

    let counters = QuerySubscriptionRuntimeCertificationCounters::scope_emitted();
    let counter_identity = runtime_certification_counter_identity(&counters);
    let scope_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_certification_scope_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("support_report"),
                support_report.report_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_parity"),
                bridge_parity.explanation_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("diagnostic_bundle"),
                admitted_diagnostic_bundle.bundle_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lifecycle_certification"),
                lifecycle_certification.certification_bundle_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("coverage_handle"),
                coverage_handle.family_coverage_identity(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), &counter_identity)
            .seal().expect("runtime certification scope grammar contains a closed family label and sealed identity tokens");

    Ok(QuerySubscriptionRuntimeCertificationScope {
        family,
        support_report,
        bridge_parity,
        admitted_diagnostic_bundle,
        lifecycle_certification,
        coverage_handle,
        scope_identity,
        counters,
    })
}
