use crate::identity::hash_parts;

use super::super::fixtures::{
    certified_admitted_intent_fixture, certified_advisory_intent_fixture,
    certified_routing_intent_fixture, certified_violation_intent_fixture,
};
use super::super::fixtures::{
    certified_deferred_intent_fixture, certified_unsupported_intent_fixture,
};
use crate::intent_admission::worth_query_intent_admission_coverage_inventory;
use crate::intent_admission::worth_query_intent_admission_support_matrix;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionSupportTraceabilityRow {
    lane: &'static str,
    family: String,
    entrypoint: String,
    support_detail: String,
    target_binding_digest: Option<String>,
    request_digest: Option<String>,
    eligibility_digest: Option<String>,
    decision_digest: Option<String>,
    row_digest: String,
}

impl WorthQueryIntentAdmissionSupportTraceabilityRow {
    pub(crate) fn new(
        lane: &'static str,
        family: impl Into<String>,
        entrypoint: impl Into<String>,
        support_detail: impl Into<String>,
    ) -> Self {
        Self::new_domain_scoped(
            lane,
            family,
            entrypoint,
            support_detail,
            None,
            None,
            None,
            None,
        )
    }

    pub(crate) fn new_domain_scoped(
        lane: &'static str,
        family: impl Into<String>,
        entrypoint: impl Into<String>,
        support_detail: impl Into<String>,
        target_binding_digest: Option<String>,
        request_digest: Option<String>,
        eligibility_digest: Option<String>,
        decision_digest: Option<String>,
    ) -> Self {
        let family = family.into();
        let entrypoint = entrypoint.into();
        let support_detail = support_detail.into();
        let target_binding_digest_part = target_binding_digest.as_deref().map_or_else(
            || "target_binding:none".to_string(),
            |digest| format!("target_binding:{digest}"),
        );
        let request_digest_part = request_digest.as_deref().map_or_else(
            || "request:none".to_string(),
            |digest| format!("request:{digest}"),
        );
        let eligibility_digest_part = eligibility_digest.as_deref().map_or_else(
            || "eligibility:none".to_string(),
            |digest| format!("eligibility:{digest}"),
        );
        let decision_digest_part = decision_digest.as_deref().map_or_else(
            || "decision:none".to_string(),
            |digest| format!("decision:{digest}"),
        );
        Self {
            lane,
            family: family.clone(),
            entrypoint: entrypoint.clone(),
            support_detail: support_detail.clone(),
            target_binding_digest,
            request_digest,
            eligibility_digest,
            decision_digest,
            row_digest: hash_parts(&[
                "worth_query_intent_admission_support_traceability_row_v1".to_string(),
                format!("lane:{lane}"),
                format!("family:{family}"),
                format!("entrypoint:{entrypoint}"),
                support_detail,
                target_binding_digest_part,
                request_digest_part,
                eligibility_digest_part,
                decision_digest_part,
            ]),
        }
    }

    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn entrypoint(&self) -> &str {
        &self.entrypoint
    }

    pub fn support_detail(&self) -> &str {
        &self.support_detail
    }

    pub fn target_binding_digest(&self) -> Option<&str> {
        self.target_binding_digest.as_deref()
    }

    pub fn request_digest(&self) -> Option<&str> {
        self.request_digest.as_deref()
    }

    pub fn eligibility_digest(&self) -> Option<&str> {
        self.eligibility_digest.as_deref()
    }

    pub fn decision_digest(&self) -> Option<&str> {
        self.decision_digest.as_deref()
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionSupportTraceabilityReport {
    rows: Vec<WorthQueryIntentAdmissionSupportTraceabilityRow>,
    decision_support_traceability_digest: String,
}

impl WorthQueryIntentAdmissionSupportTraceabilityReport {
    pub(crate) fn from_rows(rows: Vec<WorthQueryIntentAdmissionSupportTraceabilityRow>) -> Self {
        let decision_support_traceability_digest = hash_parts(
            &rows
                .iter()
                .map(|row| row.row_digest().to_string())
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            decision_support_traceability_digest,
        }
    }

    pub fn rows(&self) -> &[WorthQueryIntentAdmissionSupportTraceabilityRow] {
        &self.rows
    }

    pub fn decision_support_traceability_digest(&self) -> &str {
        &self.decision_support_traceability_digest
    }
}

pub fn worth_query_intent_admission_support_traceability_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionSupportTraceabilityReport {
    let support = worth_query_intent_admission_support_matrix();
    let admitted = certified_admitted_intent_fixture(execution);
    let advisory = certified_advisory_intent_fixture();
    let violation = certified_violation_intent_fixture();
    let routing = certified_routing_intent_fixture(execution);
    let deferred = certified_deferred_intent_fixture();
    let unsupported = certified_unsupported_intent_fixture();
    let rows = vec![
        traceability_row(
            "admitted",
            admitted.request.family().as_str(),
            admitted.request.entrypoint().as_str(),
            support_row_detail(
                &support,
                admitted.request.family(),
                admitted.request.entrypoint(),
            ),
        ),
        traceability_row(
            "advisory",
            advisory.request.family().as_str(),
            advisory.request.entrypoint().as_str(),
            support_row_detail(
                &support,
                advisory.request.family(),
                advisory.request.entrypoint(),
            ),
        ),
        traceability_row(
            "violation",
            violation.request.family().as_str(),
            violation.request.entrypoint().as_str(),
            support_row_detail(
                &support,
                violation.request.family(),
                violation.request.entrypoint(),
            ),
        ),
        traceability_row(
            "routing_admitted",
            routing.request.family().as_str(),
            routing.request.entrypoint().as_str(),
            support_row_detail(
                &support,
                routing.request.family(),
                routing.request.entrypoint(),
            ),
        ),
        traceability_row(
            "deferred",
            deferred.request.family().as_str(),
            deferred.request.entrypoint().as_str(),
            support_row_detail(
                &support,
                deferred.request.family(),
                deferred.request.entrypoint(),
            ),
        ),
        traceability_row(
            "unsupported",
            unsupported.request.family().as_str(),
            unsupported.request.entrypoint().as_str(),
            unsupported_coverage_detail(
                unsupported.request.family(),
                unsupported.request.entrypoint(),
            ),
        ),
    ];
    WorthQueryIntentAdmissionSupportTraceabilityReport::from_rows(rows)
}

fn traceability_row(
    lane: &'static str,
    family: &str,
    entrypoint: &str,
    support_detail: String,
) -> WorthQueryIntentAdmissionSupportTraceabilityRow {
    WorthQueryIntentAdmissionSupportTraceabilityRow::new(lane, family, entrypoint, support_detail)
}

fn support_row_detail(
    matrix: &crate::intent_admission::WorthQueryIntentAdmissionSupportMatrix,
    family: crate::intent_admission::WorthQueryIntentAdmissionFamily,
    entrypoint: crate::intent_admission::WorthQueryIntentAdmissionCoveredEntrypoint,
) -> String {
    let row = matrix
        .rows()
        .iter()
        .find(|row| row.family() == family && row.entrypoint() == entrypoint)
        .expect("support row should exist for certified lane");
    format!(
        "support:{}:{}:{}",
        row.posture().as_str(),
        row.execution_boundary().as_str(),
        row.detail().as_str()
    )
}

fn unsupported_coverage_detail(
    family: crate::intent_admission::WorthQueryIntentAdmissionFamily,
    entrypoint: crate::intent_admission::WorthQueryIntentAdmissionCoveredEntrypoint,
) -> String {
    let row = worth_query_intent_admission_coverage_inventory()
        .rows()
        .iter()
        .find(|row| row.family() == family && row.entrypoint() == entrypoint)
        .expect("coverage row should exist for unsupported certification lane");
    format!(
        "unsupported:{}:{}:{}",
        row.execution_boundary().as_str(),
        row.advisory_decision_class().as_str(),
        row.violation_decision_class().as_str(),
    )
}
