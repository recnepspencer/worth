use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeSet;

use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::subscription::family::QuerySubscriptionFamily;
use crate::subscription::validation_evidence::{
    validation_role_evidence_identity, validation_shape_role_evidence_identity,
};

use super::super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
    QuerySubscriptionRuntimeCertificationErrorKind,
};
use super::super::identities::{
    certified_family_coverage_handle_identity, coverage_matrix_identity,
};
use super::row::{
    CoverageResolutionPosture, QuerySubscriptionFamilyCoverageRow,
    QuerySubscriptionFamilyCoverageRowClass,
};
use super::variations::{
    QuerySubscriptionBasisVariationSet, QuerySubscriptionLifecycleClassVariationSet,
    QuerySubscriptionPolicyVariationSet, QuerySubscriptionRelationshipProofVariationSet,
    QuerySubscriptionTenantVariationSet, QuerySubscriptionViewShapeVariationSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionFamilyCoverageMatrix {
    rows: Vec<QuerySubscriptionFamilyCoverageRow>,
    family_coverage_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionFamilyCoverageMatrix {
    pub fn rows(&self) -> &[QuerySubscriptionFamilyCoverageRow] {
        &self.rows
    }
    pub fn family_coverage_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.family_coverage_identity
    }
}

pub fn build_query_subscription_family_coverage_matrix(
    rows: Vec<QuerySubscriptionFamilyCoverageRow>,
) -> Result<QuerySubscriptionFamilyCoverageMatrix, IdentityDerivationDenial> {
    let family_coverage_identity =
        coverage_matrix_identity(rows.iter().map(|row| row.row_identity()))?;
    Ok(QuerySubscriptionFamilyCoverageMatrix {
        rows,
        family_coverage_identity,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertifiedFamilyCoverageHandle {
    family: QuerySubscriptionFamily,
    coverage_resolution_posture: CoverageResolutionPosture,
    admitted_rows: Vec<QuerySubscriptionFamilyCoverageRow>,
    hostile_rows: Vec<QuerySubscriptionFamilyCoverageRow>,
    basis_variations: QuerySubscriptionBasisVariationSet,
    policy_variations: QuerySubscriptionPolicyVariationSet,
    tenant_variations: QuerySubscriptionTenantVariationSet,
    relationship_proof_variations: QuerySubscriptionRelationshipProofVariationSet,
    view_shape_variations: QuerySubscriptionViewShapeVariationSet,
    lifecycle_class_variations: QuerySubscriptionLifecycleClassVariationSet,
    family_coverage_identity: WorthQueryEvidenceIdentity,
}

impl CertifiedFamilyCoverageHandle {
    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn coverage_resolution_posture(&self) -> &CoverageResolutionPosture {
        &self.coverage_resolution_posture
    }

    pub fn admitted_rows(&self) -> &[QuerySubscriptionFamilyCoverageRow] {
        &self.admitted_rows
    }

    pub fn hostile_rows(&self) -> &[QuerySubscriptionFamilyCoverageRow] {
        &self.hostile_rows
    }

    pub fn basis_variations(&self) -> &QuerySubscriptionBasisVariationSet {
        &self.basis_variations
    }

    pub fn policy_variations(&self) -> &QuerySubscriptionPolicyVariationSet {
        &self.policy_variations
    }

    pub fn tenant_variations(&self) -> &QuerySubscriptionTenantVariationSet {
        &self.tenant_variations
    }

    pub fn relationship_proof_variations(&self) -> &QuerySubscriptionRelationshipProofVariationSet {
        &self.relationship_proof_variations
    }

    pub fn view_shape_variations(&self) -> &QuerySubscriptionViewShapeVariationSet {
        &self.view_shape_variations
    }

    pub fn lifecycle_class_variations(&self) -> &QuerySubscriptionLifecycleClassVariationSet {
        &self.lifecycle_class_variations
    }
    pub fn family_coverage_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.family_coverage_identity
    }
}

pub fn build_certified_family_coverage_handle(
    matrix: &QuerySubscriptionFamilyCoverageMatrix,
    family: &QuerySubscriptionFamily,
    posture: CoverageResolutionPosture,
) -> Result<CertifiedFamilyCoverageHandle, QuerySubscriptionRuntimeCertificationError> {
    if posture == CoverageResolutionPosture::MatrixScanDenied {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CoverageResolutionDenied,
            "runtime family coverage handle construction may not proceed from a denied matrix-scan posture",
            (|| { Ok(vec![validation_shape_role_evidence_identity("family", family.as_str())?]) })(),
            QuerySubscriptionRuntimeCertificationCounters::uncovered_family(false),
        ));
    }

    let grouped_rows = matrix
        .rows()
        .iter()
        .filter(|row| row.family() == family)
        .cloned()
        .collect::<Vec<_>>();
    if grouped_rows.is_empty() {
        return Err(QuerySubscriptionRuntimeCertificationError::new(
            QuerySubscriptionRuntimeCertificationErrorKind::CoverageFamilyMissing,
            "runtime family coverage handle construction requires at least one family-scoped coverage row",
            (|| { Ok(vec![
                validation_shape_role_evidence_identity("family", family.as_str())?,
                validation_role_evidence_identity("matrix", matrix.family_coverage_identity()),
            ]) })(),
            QuerySubscriptionRuntimeCertificationCounters::uncovered_family(
                posture == CoverageResolutionPosture::MatrixScanDebtExplicit,
            ),
        ));
    }

    let mut admitted_rows = Vec::new();
    let mut hostile_rows = Vec::new();
    let mut basis_identities = Vec::new();
    let mut policy_identities = Vec::new();
    let mut tenant_identities = Vec::new();
    let mut relationship_identities = Vec::new();
    let mut view_shape_identities = Vec::new();
    let mut lifecycle_classes = BTreeSet::new();

    for row in grouped_rows {
        basis_identities.push(row.basis_identity().clone());
        policy_identities.push(row.policy_identity().clone());
        tenant_identities.push(row.tenant_basis_identity().clone());
        relationship_identities.push(row.relationship_proof_identity().clone());
        view_shape_identities.push(row.view_shape_identity().clone());
        lifecycle_classes.insert(*row.lifecycle_class());
        match row.row_class() {
            QuerySubscriptionFamilyCoverageRowClass::Admitted => admitted_rows.push(row),
            QuerySubscriptionFamilyCoverageRowClass::HostileDenied => hostile_rows.push(row),
        }
    }

    let basis_variations =
        QuerySubscriptionBasisVariationSet::from_identities(basis_identities.iter());
    let policy_variations =
        QuerySubscriptionPolicyVariationSet::from_identities(policy_identities.iter());
    let tenant_variations =
        QuerySubscriptionTenantVariationSet::from_identities(tenant_identities.iter());
    let relationship_proof_variations =
        QuerySubscriptionRelationshipProofVariationSet::from_identities(
            relationship_identities.iter(),
        );
    let view_shape_variations =
        QuerySubscriptionViewShapeVariationSet::from_identities(view_shape_identities.iter());
    let lifecycle_class_variations =
        QuerySubscriptionLifecycleClassVariationSet::from_set(lifecycle_classes)?;

    let family_coverage_identity = certified_family_coverage_handle_identity(
        family.as_str(),
        posture.as_str(),
        matrix.family_coverage_identity(),
        basis_variations.variation_identity(),
        policy_variations.variation_identity(),
        tenant_variations.variation_identity(),
        relationship_proof_variations.variation_identity(),
        view_shape_variations.variation_identity(),
        lifecycle_class_variations?.variation_identity(),
        admitted_rows.len(),
        hostile_rows.len(),
    )?;

    Ok(CertifiedFamilyCoverageHandle {
        family: family.clone(),
        coverage_resolution_posture: posture,
        admitted_rows,
        hostile_rows,
        basis_variations,
        policy_variations,
        tenant_variations,
        relationship_proof_variations,
        view_shape_variations,
        lifecycle_class_variations,
        family_coverage_identity,
    })
}
