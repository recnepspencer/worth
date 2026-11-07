use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::FoundationalProfileSet;
use worth_proof::TransitionOutcome;

use super::super::artifact::{
    WorthQueryContributionComposedContribution, WorthQueryContributionComposedSummary,
};
use super::super::aspect::WorthQueryContributionComposedIntentAspectRecord;
use super::super::input::{
    WorthQueryContributionComposedMaterializationPolicy, WorthQueryContributionIntent,
};
use super::super::intent_result::{
    WorthQueryContributionComposedIntentClassification,
    WorthQueryContributionComposedIntentRequestDescriptor,
    WorthQueryContributionComposedIntentResult, WorthQueryContributionComposedIntentStageResult,
};
use super::support::{
    denied_result, evidence_from_admitted, failed_result, rebind_required_result,
    retained_after_admission_result, stale_result,
};
use crate::application::WorthQueryDeclarationEntryContributionCategoryFamily;
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution, materialize_domain_capability_summary,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilityTargetBinding,
    WorthQueryInstalledDeclarationContributionTarget,
    WorthQueryRequestedDomainCapabilityContribution,
};

pub(crate) fn process_contributions(
    target: WorthQueryInstalledDeclarationContributionTarget,
    declaration_aspect_record: WorthQueryContributionComposedIntentAspectRecord,
    contributions: Vec<WorthQueryContributionIntent>,
    materialization_policy: WorthQueryContributionComposedMaterializationPolicy,
    linked_artifacts: WorthQueryBindingLinkedArtifacts,
) -> Result<Vec<WorthQueryContributionComposedIntentResult>, IdentityDerivationDenial> {
    Ok(contributions
        .into_iter()
        .enumerate()
        .map(|(order_index, value)| {
            process_intent(
                order_index,
                target.clone(),
                declaration_aspect_record.clone(),
                value,
                materialization_profile(&materialization_policy),
                linked_artifacts.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn process_intent(
    order_index: usize,
    target: WorthQueryInstalledDeclarationContributionTarget,
    declaration_aspect_record: WorthQueryContributionComposedIntentAspectRecord,
    intent: WorthQueryContributionIntent,
    materialization_profile: Option<&FoundationalProfileSet>,
    linked_artifacts: WorthQueryBindingLinkedArtifacts,
) -> Result<WorthQueryContributionComposedIntentResult, IdentityDerivationDenial> {
    Ok(match intent {
        WorthQueryContributionIntent::Admission(value) => process_requested(
            order_index,
            WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
            declaration_aspect_record,
            value.bind_to_installed_target(target)?,
            materialization_profile,
            linked_artifacts,
        )?,
        WorthQueryContributionIntent::Support(value) => process_requested(
            order_index,
            WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
            declaration_aspect_record,
            value.bind_to_installed_target(target)?,
            materialization_profile,
            linked_artifacts,
        )?,
        WorthQueryContributionIntent::Explanation(value) => process_requested(
            order_index,
            WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection,
            declaration_aspect_record,
            value.bind_to_installed_target(target)?,
            materialization_profile,
            linked_artifacts,
        )?,
        WorthQueryContributionIntent::Workflow(value) => process_requested(
            order_index,
            WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
            declaration_aspect_record,
            value.bind_to_installed_target(target)?,
            materialization_profile,
            linked_artifacts,
        )?,
        WorthQueryContributionIntent::Continuity(value) => process_requested(
            order_index,
            WorthQueryDeclarationEntryContributionCategoryFamily::ContinuityLineage,
            declaration_aspect_record,
            value.bind_to_installed_target(target)?,
            materialization_profile,
            linked_artifacts,
        )?,
    })
}

fn process_requested<P>(
    order_index: usize,
    category_family: WorthQueryDeclarationEntryContributionCategoryFamily,
    declaration_aspect_record: WorthQueryContributionComposedIntentAspectRecord,
    requested: WorthQueryRequestedDomainCapabilityContribution<
        P,
        WorthQueryInstalledDeclarationContributionTarget,
    >,
    materialization_profile: Option<&FoundationalProfileSet>,
    _linked_artifacts: WorthQueryBindingLinkedArtifacts,
) -> Result<WorthQueryContributionComposedIntentResult, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    (P, WorthQueryInstalledDeclarationContributionTarget):
        crate::domain_capabilities::AllowedContributionBinding<
            P,
            WorthQueryInstalledDeclarationContributionTarget,
        >,
{
    let request = WorthQueryContributionComposedIntentRequestDescriptor::new(
        order_index,
        category_family,
        requested.payload().request_identity().clone(),
        requested.payload().target().binding_identity(),
        requested.payload().payload().semantic_code(),
        requested.payload().payload().detail(),
        declaration_aspect_record,
    );
    let evaluation = WorthQueryContributionComposedIntentStageResult::succeeded(
        "contribution request evaluated",
        Some(requested.requested_identity()?),
    );
    let eligible = match evaluate_requested_domain_capability_contribution(requested)? {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(value) => {
            return Ok(denied_result(
                request,
                WorthQueryContributionComposedIntentStageResult::denied(value.message()),
            ))
        }
        TransitionOutcome::Stale(value) => {
            return Ok(stale_result(
                request,
                format!(
                    "target {} no longer matches current {}",
                    value.bound_target_for_reporting(),
                    value.current_target_for_reporting()
                ),
            ))
        }
        TransitionOutcome::RebindRequired(value) => {
            return Ok(rebind_required_result(
                request,
                format!(
                    "target {} requires rebind to {}",
                    value.bound_target_for_reporting(),
                    value.current_target_for_reporting()
                ),
            ))
        }
        TransitionOutcome::Failed(value) => {
            return Ok(failed_result(request, value.message().to_string()))
        }
        TransitionOutcome::Deferred(never) => match never {},
    };
    let _eligibility = WorthQueryContributionComposedIntentStageResult::succeeded(
        "contribution request is eligible for admission",
        Some(eligible.eligibility_identity()?),
    );
    let admitted = match admit_eligible_domain_capability_contribution(eligible)? {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(value) => {
            return Ok(WorthQueryContributionComposedIntentResult::new(
                request,
                evaluation,
                WorthQueryContributionComposedIntentStageResult::denied(value.message()),
                WorthQueryContributionComposedIntentStageResult::not_attempted(),
                WorthQueryContributionComposedIntentClassification::Denied,
                None,
            ))
        }
        TransitionOutcome::Stale(value) => {
            return Ok(WorthQueryContributionComposedIntentResult::new(
                request,
                evaluation,
                WorthQueryContributionComposedIntentStageResult::stale(format!(
                    "target {} no longer matches current {}",
                    value.bound_target_for_reporting(),
                    value.current_target_for_reporting()
                )),
                WorthQueryContributionComposedIntentStageResult::not_attempted(),
                WorthQueryContributionComposedIntentClassification::Stale,
                None,
            ))
        }
        TransitionOutcome::RebindRequired(value) => {
            return Ok(WorthQueryContributionComposedIntentResult::new(
                request,
                evaluation,
                WorthQueryContributionComposedIntentStageResult::rebind_required(format!(
                    "target {} requires rebind to {}",
                    value.bound_target_for_reporting(),
                    value.current_target_for_reporting()
                )),
                WorthQueryContributionComposedIntentStageResult::not_attempted(),
                WorthQueryContributionComposedIntentClassification::RebindRequired,
                None,
            ))
        }
        TransitionOutcome::Failed(value) => {
            return Ok(WorthQueryContributionComposedIntentResult::new(
                request,
                evaluation,
                WorthQueryContributionComposedIntentStageResult::failed(value.message()),
                WorthQueryContributionComposedIntentStageResult::not_attempted(),
                WorthQueryContributionComposedIntentClassification::Failed,
                None,
            ))
        }
        TransitionOutcome::Deferred(never) => match never {},
    };
    let evidence = evidence_from_admitted(&admitted)?;
    let installed_authority =
        crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
            admitted.payload().target().authority_arc(),
        );
    let contribution_category = admitted.payload().payload().category();
    let semantic_posture = admitted.payload().payload().semantic_posture();
    let contribution = WorthQueryContributionComposedContribution::new(
        evidence,
        contribution_category,
        semantic_posture,
        request.request_digest().to_string(),
        installed_authority,
        None,
    );
    let admitted_digest = admitted.admitted_identity()?;
    let admission_stage = WorthQueryContributionComposedIntentStageResult::succeeded(
        "contribution admitted",
        Some(admitted_digest),
    );
    if let Some(profile) = materialization_profile.cloned() {
        let current_target = admitted.payload().target().clone();
        let ready = match prepare_admitted_domain_capability_contribution_for_materialization(
            admitted,
            current_target,
        )? {
            TransitionOutcome::Success(value) => value,
            TransitionOutcome::Denied(value) => {
                return Ok(retained_after_admission_result(
                    request,
                    evaluation,
                    admission_stage,
                    WorthQueryContributionComposedIntentStageResult::denied(value.message()),
                    contribution,
                ))
            }
            TransitionOutcome::Stale(value) => {
                return Ok(retained_after_admission_result(
                    request,
                    evaluation,
                    admission_stage,
                    WorthQueryContributionComposedIntentStageResult::stale(format!(
                        "target {} no longer matches current {}",
                        value.bound_target_for_reporting(),
                        value.current_target_for_reporting()
                    )),
                    contribution,
                ))
            }
            TransitionOutcome::RebindRequired(value) => {
                return Ok(retained_after_admission_result(
                    request,
                    evaluation,
                    admission_stage,
                    WorthQueryContributionComposedIntentStageResult::rebind_required(format!(
                        "target {} requires rebind to {}",
                        value.bound_target_for_reporting(),
                        value.current_target_for_reporting()
                    )),
                    contribution,
                ))
            }
            TransitionOutcome::Failed(value) => {
                return Ok(retained_after_admission_result(
                    request,
                    evaluation,
                    admission_stage,
                    WorthQueryContributionComposedIntentStageResult::failed(value.message()),
                    contribution,
                ))
            }
            TransitionOutcome::Deferred(never) => match never {},
        };
        let ready_identity = ready.materialization_ready_identity()?;
        return Ok(
            match materialize_domain_capability_summary(ready, profile) {
                Ok(value) => {
                    let summary = WorthQueryContributionComposedSummary::new(
                        ready_identity.as_str().to_string(),
                        value.outcome_kind(),
                        format!("{:?}", value.primary_code()),
                        value.required_row_count(),
                        value.standard_row_count(),
                        value.forensic_row_count(),
                    );
                    let contribution = contribution.with_summary(summary);
                    WorthQueryContributionComposedIntentResult::new(
                        request,
                        evaluation,
                        admission_stage,
                        WorthQueryContributionComposedIntentStageResult::succeeded(
                            "contribution summary materialized",
                            Some(ready_identity),
                        ),
                        WorthQueryContributionComposedIntentClassification::Admitted,
                        Some(contribution),
                    )
                }
                Err(value) => retained_after_admission_result(
                    request,
                    evaluation,
                    admission_stage,
                    WorthQueryContributionComposedIntentStageResult::unsupported(format!(
                        "{value:?}"
                    )),
                    contribution,
                ),
            },
        );
    }
    Ok(WorthQueryContributionComposedIntentResult::new(
        request,
        evaluation,
        admission_stage,
        WorthQueryContributionComposedIntentStageResult::not_attempted(),
        WorthQueryContributionComposedIntentClassification::Admitted,
        Some(contribution),
    ))
}

fn materialization_profile(
    policy: &WorthQueryContributionComposedMaterializationPolicy,
) -> Option<&FoundationalProfileSet> {
    match policy {
        WorthQueryContributionComposedMaterializationPolicy::None => None,
        WorthQueryContributionComposedMaterializationPolicy::Summary(value) => Some(value),
    }
}
