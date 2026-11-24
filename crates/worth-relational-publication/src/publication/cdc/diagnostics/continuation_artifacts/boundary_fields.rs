use crate::publication::cdc::data::SubscriberBoundaryAssessment;
use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use contracts::publication::cdc::data::subscriber_contract::SubscriberContractDeclaration;
use worth_relational_contracts::internal_port as contracts;

use super::diagnostic_terms::{continuation_classification_value, strata_slice_value};

pub(super) fn boundary_summary_fields(
    boundary: &SubscriberBoundaryAssessment,
) -> RelationalDiagnosticFields {
    boundary_summary_value(boundary).into()
}

pub(super) fn boundary_rejection_fields(
    boundary: &SubscriberBoundaryAssessment,
    subscriber_contract: &SubscriberContractDeclaration,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "boundary_fingerprint",
            RelationalDiagnosticValue::SchemaBoundaryFingerprint(boundary.boundary_fingerprint()),
        ),
        (
            "descriptor_continuation",
            continuation_classification_value(boundary.descriptor_continuation()),
        ),
        (
            "subscriber_outcome",
            continuation_classification_value(boundary.subscriber_outcome()),
        ),
        (
            "changed_strata",
            strata_slice_value(boundary.changed_strata()),
        ),
        (
            "contract_consumes_boundary",
            RelationalDiagnosticValue::Bool(boundary.contract_consumes_boundary()),
        ),
        (
            "accepted_continuation",
            RelationalDiagnosticValue::Bool(
                subscriber_contract
                    .accepted_continuation_classes
                    .contains(&boundary.subscriber_outcome()),
            ),
        ),
        (
            "accepted_upgrade",
            RelationalDiagnosticValue::Bool(
                subscriber_contract
                    .accepted_upgrade_classes
                    .contains(&boundary.subscriber_outcome()),
            ),
        ),
    ])
    .into()
}

fn boundary_summary_value(boundary: &SubscriberBoundaryAssessment) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "boundary_fingerprint",
            RelationalDiagnosticValue::SchemaBoundaryFingerprint(boundary.boundary_fingerprint()),
        ),
        (
            "descriptor_continuation",
            continuation_classification_value(boundary.descriptor_continuation()),
        ),
        (
            "subscriber_outcome",
            continuation_classification_value(boundary.subscriber_outcome()),
        ),
        (
            "changed_strata",
            strata_slice_value(boundary.changed_strata()),
        ),
        (
            "contract_consumes_boundary",
            RelationalDiagnosticValue::Bool(boundary.contract_consumes_boundary()),
        ),
    ])
}
