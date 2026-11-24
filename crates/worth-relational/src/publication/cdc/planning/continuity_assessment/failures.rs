use worth_relational_publication::internal_port::publication::cdc::data::subscriber_failure::SubscriberStreamFailureOwnerPort as _;
use worth_relational_publication::internal_port::publication::cdc::data::subscriber_recovery_plan::continuation_assessment::SubscriberContinuationAssessmentOwnerPort as _;
use crate::publication::cdc::data::{
    NormalizedContinuationProof, SubscriberBoundaryAssessment, SubscriberContinuationAssessment,
    SubscriberContinuationSummary, SubscriberContractDeclaration, SubscriberStreamFailure,
    SubscriberStreamFailureClass,
};
use crate::publication::cdc::diagnostics::continuation_rejection_artifact;
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaBoundaryFingerprint, SchemaContinuationClassification,
};

use super::normalized_proof::normalized_boundary_count_at_failure;

pub(super) fn unsupported_continuation_failure(
    class: SubscriberStreamFailureClass,
    detail: impl Into<String>,
    subscriber_contract: &SubscriberContractDeclaration,
    prior_proof: &NormalizedContinuationProof,
    crossed_boundaries: &[SchemaBoundaryFingerprint],
    boundary_assessments: &[SubscriberBoundaryAssessment],
    descriptor_semantics_version: DescriptorSemanticsVersion,
) -> SubscriberStreamFailure {
    let detail = detail.into();
    let continuation_outcome = boundary_assessments
        .last()
        .map(|assessment| assessment.subscriber_outcome())
        .unwrap_or(SchemaContinuationClassification::ContinueUnchanged);
    let contract_upgrade_applied = boundary_assessments.iter().any(|assessment| {
        assessment.subscriber_outcome()
            == SchemaContinuationClassification::ContinueWithContractUpgrade
    });
    let normalized_boundary_count_at_failure =
        normalized_boundary_count_at_failure(prior_proof, crossed_boundaries);
    let assessment = SubscriberContinuationAssessment::new(
        crossed_boundaries.to_vec(),
        continuation_outcome,
        contract_upgrade_applied,
        NormalizedContinuationProof::default(),
        SubscriberContinuationSummary::new(
            subscriber_contract.contract_id.clone(),
            continuation_outcome,
            crossed_boundaries.len(),
            normalized_boundary_count_at_failure,
            descriptor_semantics_version,
            contract_upgrade_applied,
        ),
        boundary_assessments.to_vec(),
    );
    SubscriberStreamFailure::new(
        class,
        detail.clone(),
        None,
        vec![continuation_rejection_artifact(
            &assessment,
            class,
            &detail,
            subscriber_contract,
            normalized_boundary_count_at_failure,
        )],
    )
}
