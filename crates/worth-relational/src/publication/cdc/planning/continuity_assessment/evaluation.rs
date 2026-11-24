use worth_relational_publication::internal_port::publication::cdc::data::subscriber_recovery_plan::continuation_assessment::{SubscriberBoundaryAssessmentOwnerPort as _, SubscriberContinuationAssessmentOwnerPort as _};
use std::collections::BTreeSet;

use crate::history::data::PositionedCanonicalCommit;
use crate::publication::cdc::data::{
    NormalizedContinuationProof, SubscriberBoundaryAssessment, SubscriberContinuationAssessment,
    SubscriberContinuationSummary, SubscriberContractDeclaration, SubscriberStreamFailure,
    SubscriberStreamFailureClass,
};
use crate::runtime::RelationalRuntime;
use crate::schema::data::{DescriptorSemanticsVersion, SchemaContinuationClassification};

use super::disposition::strongest_outcome;
use super::failures::unsupported_continuation_failure;
use super::normalized_proof::compose_normalized_proof;

pub(crate) fn assess_subscriber_continuity(
    runtime: &RelationalRuntime,
    selected_envelopes: &[PositionedCanonicalCommit],
    subscriber_contract: &SubscriberContractDeclaration,
    prior_proof: &NormalizedContinuationProof,
    resume_basis_descriptor_semantics_version: DescriptorSemanticsVersion,
) -> Result<SubscriberContinuationAssessment, SubscriberStreamFailure> {
    let mut crossed_boundaries = Vec::new();
    let mut seen_boundaries = BTreeSet::new();
    let mut continuation_outcome = SchemaContinuationClassification::ContinueUnchanged;
    let mut contract_upgrade_applied = false;
    let mut boundary_assessments = Vec::new();

    for envelope in selected_envelopes {
        let Some(descriptor) = &envelope.schema_continuation_descriptor else {
            continue;
        };

        let fingerprint = descriptor.boundary_fingerprint;
        if seen_boundaries.insert(fingerprint) {
            crossed_boundaries.push(fingerprint);
        }

        let contract_consumes_boundary =
            subscriber_contract.consumes_any_strata(&descriptor.bridge.changed_strata);
        let subscriber_specific_outcome = if !contract_consumes_boundary {
            SchemaContinuationClassification::ContinueUnchanged
        } else {
            descriptor.bridge.continuation
        };
        boundary_assessments.push(SubscriberBoundaryAssessment::new(
            fingerprint,
            descriptor.bridge.continuation,
            subscriber_specific_outcome,
            descriptor.bridge.changed_strata.clone(),
            contract_consumes_boundary,
        ));

        match subscriber_specific_outcome {
            SchemaContinuationClassification::ContinueUnchanged => {
                if !subscriber_contract.accepts_continuation(subscriber_specific_outcome) {
                    return Err(unsupported_continuation_failure(
                        SubscriberStreamFailureClass::UnsupportedContinuation,
                        "subscriber contract does not accept unchanged continuation classification",
                        subscriber_contract,
                        prior_proof,
                        &crossed_boundaries,
                        &boundary_assessments,
                        assessed_descriptor_semantics_version(
                            selected_envelopes,
                            resume_basis_descriptor_semantics_version,
                        ),
                    ));
                }
            }
            SchemaContinuationClassification::ContinueWithTransparentBridge
            | SchemaContinuationClassification::ContinueWithVisibleBridge => {
                if !subscriber_contract.accepts_continuation(subscriber_specific_outcome) {
                    return Err(unsupported_continuation_failure(
                        SubscriberStreamFailureClass::UnsupportedContinuation,
                        format!(
                            "subscriber contract does not accept {:?}",
                            subscriber_specific_outcome
                        ),
                        subscriber_contract,
                        prior_proof,
                        &crossed_boundaries,
                        &boundary_assessments,
                        assessed_descriptor_semantics_version(
                            selected_envelopes,
                            resume_basis_descriptor_semantics_version,
                        ),
                    ));
                }
            }
            SchemaContinuationClassification::ContinueWithContractUpgrade => {
                if !subscriber_contract.accepts_upgrade(subscriber_specific_outcome) {
                    return Err(unsupported_continuation_failure(
                        SubscriberStreamFailureClass::ContractUpgradeUnsupported,
                        "subscriber contract does not accept contract upgrade continuation",
                        subscriber_contract,
                        prior_proof,
                        &crossed_boundaries,
                        &boundary_assessments,
                        assessed_descriptor_semantics_version(
                            selected_envelopes,
                            resume_basis_descriptor_semantics_version,
                        ),
                    ));
                }
                contract_upgrade_applied = true;
            }
            SchemaContinuationClassification::RequireRenegotiation => {
                return Err(unsupported_continuation_failure(
                    SubscriberStreamFailureClass::RenegotiationRequired,
                    "schema boundary requires subscriber renegotiation before continuation",
                    subscriber_contract,
                    prior_proof,
                    &crossed_boundaries,
                    &boundary_assessments,
                    assessed_descriptor_semantics_version(
                        selected_envelopes,
                        resume_basis_descriptor_semantics_version,
                    ),
                ));
            }
            SchemaContinuationClassification::Rejected => {
                return Err(unsupported_continuation_failure(
                    SubscriberStreamFailureClass::SchemaUnsupported,
                    "schema boundary rejected subscriber continuation",
                    subscriber_contract,
                    prior_proof,
                    &crossed_boundaries,
                    &boundary_assessments,
                    assessed_descriptor_semantics_version(
                        selected_envelopes,
                        resume_basis_descriptor_semantics_version,
                    ),
                ));
            }
            _ => unreachable!("unhandled Relational contract variant"),
        }

        continuation_outcome = strongest_outcome(continuation_outcome, subscriber_specific_outcome);
    }

    let descriptor_semantics_version = assessed_descriptor_semantics_version(
        selected_envelopes,
        resume_basis_descriptor_semantics_version,
    );
    let normalized_continuation_proof = compose_normalized_proof(
        runtime,
        prior_proof,
        &crossed_boundaries,
        descriptor_semantics_version,
        subscriber_contract,
        &boundary_assessments,
    )?;
    let continuation_summary = SubscriberContinuationSummary::new(
        subscriber_contract.contract_id.clone(),
        continuation_outcome,
        crossed_boundaries.len(),
        normalized_continuation_proof.normalized_boundary_count(),
        descriptor_semantics_version,
        contract_upgrade_applied,
    );
    runtime
        .performance_access()
        .count_subscriber_resume_evaluation(continuation_outcome);

    Ok(SubscriberContinuationAssessment::new(
        crossed_boundaries,
        continuation_outcome,
        contract_upgrade_applied,
        normalized_continuation_proof,
        continuation_summary,
        boundary_assessments,
    ))
}

fn assessed_descriptor_semantics_version(
    selected_envelopes: &[PositionedCanonicalCommit],
    resume_basis_descriptor_semantics_version: DescriptorSemanticsVersion,
) -> DescriptorSemanticsVersion {
    selected_envelopes
        .last()
        .map(|envelope| envelope.descriptor_semantics_version)
        .unwrap_or(resume_basis_descriptor_semantics_version)
}
