use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::super::evidence_identities::support_counters_identity;
use super::super::super::evidence_projection::subscription_evidence_projection;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuerySubscriptionSupportCounters {
    support_report_request_count: u64,
    supported_family_count: u64,
    denied_family_count: u64,
    deferred_family_count: u64,
    uncertified_family_denial_count: u64,
    support_matrix_emission_count: u64,
    support_family_index_lookup_count: u64,
    support_matrix_scan_debt_count: u64,
}

impl QuerySubscriptionSupportCounters {
    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        support_counters_identity(
            self.support_report_request_count,
            self.supported_family_count,
            self.denied_family_count,
            self.deferred_family_count,
            self.uncertified_family_denial_count,
            self.support_matrix_emission_count,
            self.support_family_index_lookup_count,
            self.support_matrix_scan_debt_count,
        )
    }

    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        let identity = self.evidence_identity();
        subscription_evidence_projection(&identity)
    }

    pub fn support_report_request_count(&self) -> u64 {
        self.support_report_request_count
    }

    pub fn supported_family_count(&self) -> u64 {
        self.supported_family_count
    }

    pub fn denied_family_count(&self) -> u64 {
        self.denied_family_count
    }

    pub fn deferred_family_count(&self) -> u64 {
        self.deferred_family_count
    }

    pub fn uncertified_family_denial_count(&self) -> u64 {
        self.uncertified_family_denial_count
    }

    pub fn support_matrix_emission_count(&self) -> u64 {
        self.support_matrix_emission_count
    }

    pub fn support_family_index_lookup_count(&self) -> u64 {
        self.support_family_index_lookup_count
    }

    pub fn support_matrix_scan_debt_count(&self) -> u64 {
        self.support_matrix_scan_debt_count
    }
}

pub(super) fn counters_for_posture(
    posture: &super::super::subject::QuerySubscriptionSupportPosture,
) -> QuerySubscriptionSupportCounters {
    let mut counters = QuerySubscriptionSupportCounters {
        support_report_request_count: 1,
        support_matrix_emission_count: 1,
        support_family_index_lookup_count: 1,
        ..Default::default()
    };
    match posture {
        super::super::subject::QuerySubscriptionSupportPosture::RuntimeBackedCertified => {
            counters.supported_family_count = 1;
        }
        super::super::subject::QuerySubscriptionSupportPosture::RuntimeBackedDenied => {
            counters.denied_family_count = 1;
        }
        super::super::subject::QuerySubscriptionSupportPosture::RuntimeBackedDeferred => {
            counters.deferred_family_count = 1;
        }
        super::super::subject::QuerySubscriptionSupportPosture::UncertifiedDenied => {
            counters.uncertified_family_denial_count = 1;
            counters.denied_family_count = 1;
        }
    }
    counters
}
