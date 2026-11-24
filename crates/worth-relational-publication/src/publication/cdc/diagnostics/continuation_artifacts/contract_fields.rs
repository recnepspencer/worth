use crate::publication::cdc::data::SubscriberContinuationAssessment;
use crate::publication::cdc::data::SubscriberStreamFailureClass;
use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use contracts::publication::cdc::data::subscriber_contract::SubscriberContractDeclaration;
use worth_relational_contracts::internal_port as contracts;

use super::diagnostic_terms::{
    continuation_class_set_value, continuation_classification_value, failure_class_value,
    strata_set_value,
};

pub(super) fn continuation_summary_fields(
    assessment: &SubscriberContinuationAssessment,
    contract_id: &str,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "subscriber_contract_id",
            RelationalDiagnosticValue::string(contract_id),
        ),
        (
            "continuation_outcome",
            continuation_classification_value(assessment.continuation_outcome()),
        ),
        (
            "crossed_boundary_count",
            RelationalDiagnosticValue::unsigned(assessment.crossed_boundaries().len()),
        ),
        (
            "normalized_boundary_count",
            normalized_boundary_count_value(assessment),
        ),
        (
            "contract_upgrade_applied",
            RelationalDiagnosticValue::Bool(assessment.contract_upgrade_applied()),
        ),
    ])
    .into()
}

pub(super) fn continuation_upgrade_fields(
    assessment: &SubscriberContinuationAssessment,
    contract_id: &str,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "subscriber_contract_id",
            RelationalDiagnosticValue::string(contract_id),
        ),
        (
            "continuation_outcome",
            continuation_classification_value(assessment.continuation_outcome()),
        ),
        (
            "normalized_boundary_count",
            normalized_boundary_count_value(assessment),
        ),
    ])
    .into()
}

pub(super) fn continuation_renegotiation_fields(
    assessment: &SubscriberContinuationAssessment,
    contract_id: &str,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "subscriber_contract_id",
            RelationalDiagnosticValue::string(contract_id),
        ),
        (
            "normalized_boundary_count",
            normalized_boundary_count_value(assessment),
        ),
    ])
    .into()
}

pub(super) fn continuation_rejection_fields(
    failure_class: SubscriberStreamFailureClass,
    detail: &str,
    subscriber_contract: &SubscriberContractDeclaration,
    normalized_boundary_count_at_failure: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        ("failure_class", failure_class_value(failure_class)),
        ("detail", RelationalDiagnosticValue::string(detail)),
        (
            "subscriber_contract_id",
            RelationalDiagnosticValue::string(&subscriber_contract.contract_id),
        ),
        (
            "accepted_continuation_classes",
            continuation_class_set_value(&subscriber_contract.accepted_continuation_classes),
        ),
        (
            "accepted_upgrade_classes",
            continuation_class_set_value(&subscriber_contract.accepted_upgrade_classes),
        ),
        (
            "consumable_strata",
            strata_set_value(&subscriber_contract.consumable_strata),
        ),
        (
            "normalized_boundary_count_at_failure",
            RelationalDiagnosticValue::unsigned(normalized_boundary_count_at_failure),
        ),
    ])
    .into()
}

pub(super) fn continuation_rejection_upgrade_fields(
    subscriber_contract: &SubscriberContractDeclaration,
    normalized_boundary_count_at_failure: usize,
) -> RelationalDiagnosticFields {
    rejection_contract_boundary_count_fields(
        subscriber_contract,
        normalized_boundary_count_at_failure,
    )
}

pub(super) fn continuation_rejection_renegotiation_fields(
    subscriber_contract: &SubscriberContractDeclaration,
    normalized_boundary_count_at_failure: usize,
) -> RelationalDiagnosticFields {
    rejection_contract_boundary_count_fields(
        subscriber_contract,
        normalized_boundary_count_at_failure,
    )
}

fn normalized_boundary_count_value(
    assessment: &SubscriberContinuationAssessment,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::unsigned(
        assessment
            .normalized_continuation_proof()
            .normalized_boundary_count(),
    )
}

fn rejection_contract_boundary_count_fields(
    subscriber_contract: &SubscriberContractDeclaration,
    normalized_boundary_count_at_failure: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "subscriber_contract_id",
            RelationalDiagnosticValue::string(&subscriber_contract.contract_id),
        ),
        (
            "normalized_boundary_count_at_failure",
            RelationalDiagnosticValue::unsigned(normalized_boundary_count_at_failure),
        ),
    ])
    .into()
}
