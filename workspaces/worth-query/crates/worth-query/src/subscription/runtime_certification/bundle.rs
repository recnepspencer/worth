use super::coverage::CoverageResolutionPosture;
use super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
    QuerySubscriptionRuntimeCertificationErrorKind,
};
use super::identities::{
    coverage_receipt_identity, coverage_width_identity, hostile_coverage_identity,
    runtime_certification_bundle_identity, runtime_certification_counter_identity,
};
use super::scope::QuerySubscriptionRuntimeCertificationScope;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::subscription::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
    validation_usize_role_evidence_identity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionCertificationCoverageWidth {
    admitted_row_count: usize,
    hostile_row_count: usize,
    covered_variation_axis_count: usize,
    pub(in crate::subscription) width_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionCertificationCoverageWidth {
    fn new(
        admitted_row_count: usize,
        hostile_row_count: usize,
        covered_variation_axis_count: usize,
    ) -> Self {
        let width_identity = coverage_width_identity(
            admitted_row_count,
            hostile_row_count,
            covered_variation_axis_count,
        );
        Self {
            admitted_row_count,
            hostile_row_count,
            covered_variation_axis_count,
            width_identity,
        }
    }

    pub fn admitted_row_count(&self) -> usize {
        self.admitted_row_count
    }

    pub fn hostile_row_count(&self) -> usize {
        self.hostile_row_count
    }

    pub fn covered_variation_axis_count(&self) -> usize {
        self.covered_variation_axis_count
    }

    pub fn width_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.width_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificationCoverageReceipt {
    coverage_resolution_posture: CoverageResolutionPosture,
    family_coverage_index_lookup_count: usize,
    covered_row_width: SubscriptionCertificationCoverageWidth,
    uncovered_variation_width: usize,
    pub(in crate::subscription) receipt_identity: WorthQueryEvidenceIdentity,
}

impl CertificationCoverageReceipt {
    fn new(
        coverage_resolution_posture: CoverageResolutionPosture,
        family_coverage_index_lookup_count: usize,
        covered_row_width: SubscriptionCertificationCoverageWidth,
        uncovered_variation_width: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let receipt_identity = coverage_receipt_identity(
            coverage_resolution_posture,
            family_coverage_index_lookup_count,
            covered_row_width.width_identity(),
            uncovered_variation_width,
        )?;
        Ok(Self {
            coverage_resolution_posture,
            family_coverage_index_lookup_count,
            covered_row_width,
            uncovered_variation_width,
            receipt_identity,
        })
    }

    pub fn coverage_resolution_posture(&self) -> &CoverageResolutionPosture {
        &self.coverage_resolution_posture
    }

    pub fn family_coverage_index_lookup_count(&self) -> usize {
        self.family_coverage_index_lookup_count
    }

    pub fn covered_row_width(&self) -> &SubscriptionCertificationCoverageWidth {
        &self.covered_row_width
    }

    pub fn uncovered_variation_width(&self) -> usize {
        self.uncovered_variation_width
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionRuntimeCertificationBundle {
    pub(in crate::subscription) query_scope_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_family_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) bridge_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) signal_strategy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) support_report_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) bridge_parity_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) diagnostic_bundle_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) lifecycle_certification_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) hostile_coverage_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) family_coverage_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) runtime_certification_bundle_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) counter_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionRuntimeCertificationCounters,
}

impl QuerySubscriptionRuntimeCertificationBundle {
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

    pub fn diagnostic_bundle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostic_bundle_identity
    }

    pub fn lifecycle_certification_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lifecycle_certification_identity
    }

    pub fn hostile_coverage_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.hostile_coverage_identity
    }

    pub fn family_coverage_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.family_coverage_identity
    }

    pub fn runtime_certification_bundle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.runtime_certification_bundle_identity
    }

    pub fn counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionRuntimeCertificationCounters {
        &self.counters
    }
}

fn family_coverage_index_lookup_count(posture: &CoverageResolutionPosture) -> usize {
    match posture {
        CoverageResolutionPosture::IndexedCoverageSet => 1,
        CoverageResolutionPosture::PrecomputedCoverageMatrix
        | CoverageResolutionPosture::MatrixScanDebtExplicit
        | CoverageResolutionPosture::MatrixScanDenied => 0,
    }
}

pub fn certify_query_subscription_runtime_family(
    scope: QuerySubscriptionRuntimeCertificationScope,
) -> Result<
    (
        QuerySubscriptionRuntimeCertificationBundle,
        CertificationCoverageReceipt,
    ),
    QuerySubscriptionRuntimeCertificationError,
> {
    let coverage_handle = scope.coverage_handle();
    let covered_row_width = SubscriptionCertificationCoverageWidth::new(
        coverage_handle.admitted_rows().len(),
        coverage_handle.hostile_rows().len(),
        6,
    );
    let uncovered_variation_width = usize::from(coverage_handle.admitted_rows().is_empty())
        + usize::from(coverage_handle.hostile_rows().is_empty())
        + usize::from(coverage_handle.basis_variations().digests().is_empty())
        + usize::from(coverage_handle.policy_variations().digests().is_empty())
        + usize::from(coverage_handle.tenant_variations().digests().is_empty())
        + usize::from(
            coverage_handle
                .relationship_proof_variations()
                .digests()
                .is_empty(),
        )
        + usize::from(coverage_handle.view_shape_variations().digests().is_empty())
        + usize::from(
            coverage_handle
                .lifecycle_class_variations()
                .classes()
                .is_empty(),
        );

    if coverage_handle.hostile_rows().is_empty() {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::MissingHostileCoverage,
            "runtime family certification requires at least one hostile family coverage row for every supported family",
            &[
                validation_shape_role_evidence_identity("family", scope.family().as_str()),
                validation_role_evidence_identity(
                    "coverage",
                    coverage_handle.family_coverage_identity(),
                ),
            ],
            QuerySubscriptionRuntimeCertificationCounters::uncovered_family(false),
        ));
    }

    if uncovered_variation_width > 0 {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::UncoveredFamily,
            "runtime family certification requires admitted coverage plus non-empty basis, policy, tenant, relationship-proof, view-shape, and lifecycle variation sets",
            &[
                validation_shape_role_evidence_identity("family", scope.family().as_str()),
                validation_role_evidence_identity(
                    "coverage",
                    coverage_handle.family_coverage_identity(),
                ),
                validation_usize_role_evidence_identity(
                    "uncovered_variation_width",
                    uncovered_variation_width,
                ),
            ],
            QuerySubscriptionRuntimeCertificationCounters::uncovered_family(false),
        ));
    }

    let hostile_coverage_identity = hostile_coverage_identity(coverage_handle.hostile_rows());
    let coverage_resolution_posture = *coverage_handle.coverage_resolution_posture();
    let receipt = CertificationCoverageReceipt::new(
        coverage_resolution_posture,
        family_coverage_index_lookup_count(&coverage_resolution_posture),
        covered_row_width,
        uncovered_variation_width,
    );
    let counters = QuerySubscriptionRuntimeCertificationCounters::certified(
        coverage_handle.hostile_rows().len(),
        coverage_resolution_posture,
    );
    let counter_identity = runtime_certification_counter_identity(&counters);
    let runtime_certification_bundle_identity = runtime_certification_bundle_identity(
        scope.scope_identity(),
        scope.support_report().report_identity(),
        scope.bridge_parity().explanation_identity(),
        scope.admitted_diagnostic_bundle().bundle_identity(),
        scope
            .lifecycle_certification()
            .certification_bundle_identity(),
        coverage_handle.family_coverage_identity(),
        &hostile_coverage_identity,
        receipt.receipt_identity(),
        &counter_identity,
    );

    Ok((
        QuerySubscriptionRuntimeCertificationBundle {
            query_scope_identity: scope
                .lifecycle_certification()
                .query_scope_identity()
                .clone(),
            subscription_family_identity: scope
                .lifecycle_certification()
                .subscription_family_identity()
                .clone(),
            subscription_declaration_identity: scope
                .lifecycle_certification()
                .subscription_declaration_identity()
                .clone(),
            bridge_declaration_identity: scope
                .lifecycle_certification()
                .bridge_declaration_identity()
                .clone(),
            signal_strategy_identity: scope
                .lifecycle_certification()
                .signal_strategy_identity()
                .clone(),
            support_report_identity: scope.support_report().report_identity().clone(),
            bridge_parity_identity: scope.bridge_parity().explanation_identity().clone(),
            diagnostic_bundle_identity: scope
                .admitted_diagnostic_bundle()
                .bundle_identity()
                .clone(),
            lifecycle_certification_identity: scope
                .lifecycle_certification()
                .certification_bundle_identity()
                .clone(),
            hostile_coverage_identity,
            family_coverage_identity: coverage_handle.family_coverage_identity().clone(),
            runtime_certification_bundle_identity,
            counter_identity,
            counters,
        },
        receipt,
    ))
}
