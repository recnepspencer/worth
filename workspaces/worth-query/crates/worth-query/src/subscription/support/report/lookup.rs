use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::super::evidence_identities::support_lookup_receipt_identity;
use super::super::super::evidence_projection::subscription_evidence_projection;
use super::super::super::family::QuerySubscriptionFamily;
use super::super::subject::QuerySubscriptionSupportClass;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportResolutionPosture {
    IndexedFamilyLookup,
    PrecomputedFamilyMatrix,
    LinearScanDebtExplicit,
    LinearScanDenied,
}

impl SupportResolutionPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndexedFamilyLookup => "indexed_family_lookup",
            Self::PrecomputedFamilyMatrix => "precomputed_family_matrix",
            Self::LinearScanDebtExplicit => "linear_scan_debt_explicit",
            Self::LinearScanDenied => "linear_scan_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportLookupReceipt {
    family: QuerySubscriptionFamily,
    support_class: QuerySubscriptionSupportClass,
    resolution_posture: SupportResolutionPosture,
    consumed_lookup_width: usize,
    remaining_lookup_width: usize,
    lookup_receipt_identity: WorthQueryEvidenceIdentity,
}

impl SupportLookupReceipt {
    pub(super) fn new(
        family: &QuerySubscriptionFamily,
        support_class: QuerySubscriptionSupportClass,
        resolution_posture: SupportResolutionPosture,
        consumed_lookup_width: usize,
        remaining_lookup_width: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let lookup_receipt_identity = support_lookup_receipt_identity(
            family,
            support_class.as_str(),
            resolution_posture.as_str(),
            consumed_lookup_width,
            remaining_lookup_width,
        )?;
        Ok(Self {
            family: family.clone(),
            support_class,
            resolution_posture,
            consumed_lookup_width,
            remaining_lookup_width,
            lookup_receipt_identity,
        })
    }

    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn support_class(&self) -> &QuerySubscriptionSupportClass {
        &self.support_class
    }

    pub fn resolution_posture(&self) -> &SupportResolutionPosture {
        &self.resolution_posture
    }

    pub fn consumed_lookup_width(&self) -> usize {
        self.consumed_lookup_width
    }

    pub fn remaining_lookup_width(&self) -> usize {
        self.remaining_lookup_width
    }

    pub fn lookup_receipt_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.lookup_receipt_identity)
    }

    pub fn lookup_receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lookup_receipt_identity
    }
}
