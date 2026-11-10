use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::subscription::bridge_parity::QuerySubscriptionBridgeParityExplanation;
use crate::subscription::certification::SubscriptionLifecycleCertificationBundle;
use crate::subscription::diagnostic::{
    QuerySubscriptionAdmittedDiagnosticBundle, QuerySubscriptionDeniedDiagnosticBundle,
    QuerySubscriptionDiagnosticFailure,
};
use crate::subscription::evidence_identities::typed_identity_drift;
use crate::subscription::family::QuerySubscriptionFamily;
use crate::subscription::support::QuerySubscriptionSupportReport;
use crate::subscription::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
};

use super::super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
    QuerySubscriptionRuntimeCertificationErrorKind,
};
use super::super::identities::coverage_row_identity;
use super::validation::{validate_hostile_diagnostic_alignment, validate_row_alignment};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageResolutionPosture {
    IndexedCoverageSet,
    PrecomputedCoverageMatrix,
    MatrixScanDebtExplicit,
    MatrixScanDenied,
}

impl CoverageResolutionPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndexedCoverageSet => "indexed_coverage_set",
            Self::PrecomputedCoverageMatrix => "precomputed_coverage_matrix",
            Self::MatrixScanDebtExplicit => "matrix_scan_debt_explicit",
            Self::MatrixScanDenied => "matrix_scan_denied",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QuerySubscriptionLifecycleCoverageClass {
    ActiveLifecycle,
    Continuation,
    PreviewIsolation,
    LifecycleCloseout,
}

impl QuerySubscriptionLifecycleCoverageClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveLifecycle => "active_lifecycle",
            Self::Continuation => "continuation",
            Self::PreviewIsolation => "preview_isolation",
            Self::LifecycleCloseout => "lifecycle_closeout",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionFamilyCoverageRowClass {
    Admitted,
    HostileDenied,
}

impl QuerySubscriptionFamilyCoverageRowClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::HostileDenied => "hostile_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionFamilyCoverageRow {
    family: QuerySubscriptionFamily,
    row_class: QuerySubscriptionFamilyCoverageRowClass,
    pub(in crate::subscription) query_scope_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_family_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) bridge_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) signal_strategy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) support_report_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) bridge_parity_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) lifecycle_certification_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) diagnostic_bundle_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) failure_identity: Option<WorthQueryEvidenceIdentity>,
    pub(in crate::subscription) basis_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) policy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) tenant_basis_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) relationship_proof_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) view_shape_identity: WorthQueryEvidenceIdentity,
    lifecycle_class: QuerySubscriptionLifecycleCoverageClass,
    pub(in crate::subscription) row_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionFamilyCoverageRow {
    pub fn admitted(
        family: &QuerySubscriptionFamily,
        support: &QuerySubscriptionSupportReport,
        parity: &QuerySubscriptionBridgeParityExplanation,
        lifecycle: &SubscriptionLifecycleCertificationBundle,
        diagnostic: &QuerySubscriptionAdmittedDiagnosticBundle,
        lifecycle_class: QuerySubscriptionLifecycleCoverageClass,
    ) -> Result<Self, QuerySubscriptionRuntimeCertificationError> {
        validate_row_alignment(family, support, parity, lifecycle)?;
        if typed_identity_drift(
            diagnostic.support_report_identity(),
            support.report_identity(),
        ) || typed_identity_drift(
            diagnostic.lifecycle_certification_identity(),
            lifecycle.certification_bundle_identity(),
        ) {
            return Err(QuerySubscriptionRuntimeCertificationError::new(
                QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch,
                "admitted family coverage rows require diagnostic bundle digests aligned with support and lifecycle evidence",
                Ok(vec![
                    validation_role_evidence_identity(
                        "diagnostic_support",
                        diagnostic.support_report_identity(),
                    ),
                    validation_role_evidence_identity("support", support.report_identity()),
                    validation_role_evidence_identity(
                        "diagnostic_lifecycle",
                        diagnostic.lifecycle_certification_identity(),
                    ),
                    validation_role_evidence_identity(
                        "lifecycle",
                        lifecycle.certification_bundle_identity(),
                    ),
                ]),
                QuerySubscriptionRuntimeCertificationCounters::default(),
            ));
        }

        Self::new(
            family,
            QuerySubscriptionFamilyCoverageRowClass::Admitted,
            lifecycle,
            support.report_identity(),
            parity.explanation_identity(),
            diagnostic.bundle_identity(),
            None,
            lifecycle_class,
        )
        .map_err(|cause| {
            QuerySubscriptionRuntimeCertificationError::identity_refused(
                cause,
                QuerySubscriptionRuntimeCertificationCounters::default(),
            )
        })
    }

    pub fn hostile(
        family: &QuerySubscriptionFamily,
        support: &QuerySubscriptionSupportReport,
        parity: &QuerySubscriptionBridgeParityExplanation,
        lifecycle: &SubscriptionLifecycleCertificationBundle,
        diagnostic: &QuerySubscriptionDeniedDiagnosticBundle,
        failure: &QuerySubscriptionDiagnosticFailure,
        lifecycle_class: QuerySubscriptionLifecycleCoverageClass,
    ) -> Result<Self, QuerySubscriptionRuntimeCertificationError> {
        validate_row_alignment(family, support, parity, lifecycle)?;
        validate_hostile_diagnostic_alignment(diagnostic, support, lifecycle)?;
        if diagnostic.semantic_labels().query_family_label() != family.as_str() {
            return Err(QuerySubscriptionRuntimeCertificationError::new(
                QuerySubscriptionRuntimeCertificationErrorKind::CoverageFamilyMismatch,
                "hostile family coverage rows require denied diagnostic bundles for the same query subscription family",
                (|| { Ok(vec![
                    validation_shape_role_evidence_identity(
                        "diagnostic_family",
                        diagnostic.semantic_labels().query_family_label(),
                    )?,
                    validation_shape_role_evidence_identity(
                        "expected_family",
                        family.as_str(),
                    )?,
                ]) })(),
                QuerySubscriptionRuntimeCertificationCounters::default(),
            ));
        }

        Self::new(
            family,
            QuerySubscriptionFamilyCoverageRowClass::HostileDenied,
            lifecycle,
            support.report_identity(),
            parity.explanation_identity(),
            diagnostic.bundle_identity(),
            Some(failure.failure_identity()),
            lifecycle_class,
        )
        .map_err(|cause| {
            QuerySubscriptionRuntimeCertificationError::identity_refused(
                cause,
                QuerySubscriptionRuntimeCertificationCounters::default(),
            )
        })
    }

    fn new(
        family: &QuerySubscriptionFamily,
        row_class: QuerySubscriptionFamilyCoverageRowClass,
        lifecycle: &SubscriptionLifecycleCertificationBundle,
        support_report_identity: &WorthQueryEvidenceIdentity,
        bridge_parity_identity: &WorthQueryEvidenceIdentity,
        diagnostic_bundle_identity: &WorthQueryEvidenceIdentity,
        failure_identity: Option<&WorthQueryEvidenceIdentity>,
        lifecycle_class: QuerySubscriptionLifecycleCoverageClass,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity = coverage_row_identity(
            family.as_str(),
            row_class.as_str(),
            lifecycle.query_scope_identity(),
            lifecycle.subscription_family_identity(),
            lifecycle.subscription_declaration_identity(),
            lifecycle.bridge_declaration_identity(),
            lifecycle.signal_strategy_identity(),
            lifecycle.basis_posture_identity(),
            lifecycle.policy_identity(),
            lifecycle.tenant_basis_identity(),
            lifecycle.relationship_proof_identity(),
            lifecycle.view_shape_identity(),
            support_report_identity,
            bridge_parity_identity,
            lifecycle.certification_bundle_identity(),
            diagnostic_bundle_identity,
            lifecycle_class.as_str(),
            failure_identity,
        )?;
        Ok(Self {
            family: family.clone(),
            row_class,
            query_scope_identity: lifecycle.query_scope_identity().clone(),
            subscription_family_identity: lifecycle.subscription_family_identity().clone(),
            subscription_declaration_identity: lifecycle
                .subscription_declaration_identity()
                .clone(),
            bridge_declaration_identity: lifecycle.bridge_declaration_identity().clone(),
            signal_strategy_identity: lifecycle.signal_strategy_identity().clone(),
            support_report_identity: support_report_identity.clone(),
            bridge_parity_identity: bridge_parity_identity.clone(),
            lifecycle_certification_identity: lifecycle.certification_bundle_identity().clone(),
            diagnostic_bundle_identity: diagnostic_bundle_identity.clone(),
            failure_identity: failure_identity.cloned(),
            basis_identity: lifecycle.basis_posture_identity().clone(),
            policy_identity: lifecycle.policy_identity().clone(),
            tenant_basis_identity: lifecycle.tenant_basis_identity().clone(),
            relationship_proof_identity: lifecycle.relationship_proof_identity().clone(),
            view_shape_identity: lifecycle.view_shape_identity().clone(),
            lifecycle_class,
            row_identity,
        })
    }

    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn row_class(&self) -> &QuerySubscriptionFamilyCoverageRowClass {
        &self.row_class
    }

    pub fn query_scope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_scope_identity
    }

    pub fn subscription_family_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.subscription_family_identity
    }

    pub fn subscription_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.subscription_declaration_identity
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn signal_strategy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.signal_strategy_identity
    }

    pub fn support_report_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_report_identity
    }

    pub fn bridge_parity_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_parity_identity
    }

    pub fn lifecycle_certification_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lifecycle_certification_identity
    }

    pub fn diagnostic_bundle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostic_bundle_identity
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn policy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_identity
    }

    pub fn tenant_basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.tenant_basis_identity
    }

    pub fn relationship_proof_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relationship_proof_identity
    }

    pub fn view_shape_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.view_shape_identity
    }

    pub fn failure_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.failure_identity.as_ref()
    }

    pub fn lifecycle_class(&self) -> &QuerySubscriptionLifecycleCoverageClass {
        &self.lifecycle_class
    }

    pub fn row_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_identity
    }
}
