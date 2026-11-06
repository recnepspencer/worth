use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDomainCapabilityTargetBinding,
};
use crate::domain_capabilities::{
    WorthQueryCanonicalAftermathArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyAftermathContribution,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::projection_consumption::{
    declare_projection_consumption, discover_projection_consumption_support,
    evaluate_projection_consumption_eligibility, AdmittedProjectionConsumption,
    DeferredProjectionConsumptionReason, MaterializedProjectionContract,
    ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError,
    ProjectionConsumptionDenialReason, ProjectionConsumptionEligibility,
    ProjectionConsumptionSupportReport,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAftermathProjectionConsumptionReview {
    semantic_code: String,
    request_identity: WorthQueryEvidenceIdentity,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    declaration: ProjectionConsumptionDeclaration,
    support_report: ProjectionConsumptionSupportReport,
    eligibility: ProjectionConsumptionEligibility,
}

impl WorthQueryAftermathProjectionConsumptionReview {
    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn target_kind(&self) -> crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind {
        self.target_kind
    }

    pub fn declaration(&self) -> &ProjectionConsumptionDeclaration {
        &self.declaration
    }

    pub fn support_report(&self) -> &ProjectionConsumptionSupportReport {
        &self.support_report
    }

    pub fn eligibility(&self) -> &ProjectionConsumptionEligibility {
        &self.eligibility
    }
}

pub fn materialize_canonical_aftermath_artifact<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<WorthQueryCanonicalAftermathArtifact<T>, IdentityDerivationDenial>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    Ok(super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)?)
}

pub(crate) fn materialize_projection_consumption_eligibility<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<ProjectionConsumptionEligibility>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryAftermathProjectionTarget,
{
    Ok(
        match materialize_projection_consumption_review(contribution)? {
            TransitionOutcome::Success(review) => {
                TransitionOutcome::Success(review.eligibility().clone())
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
            TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
            TransitionOutcome::Deferred(never) => match never {},
        },
    )
}

pub(crate) fn materialize_projection_consumption_support_report<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<ProjectionConsumptionSupportReport>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryAftermathProjectionTarget,
{
    Ok(
        match materialize_projection_consumption_review(contribution)? {
            TransitionOutcome::Success(review) => {
                TransitionOutcome::Success(review.support_report().clone())
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
            TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
            TransitionOutcome::Deferred(never) => match never {},
        },
    )
}

pub(crate) fn materialize_projection_consumption_review<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryAftermathProjectionConsumptionReview>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryAftermathProjectionTarget,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some(runtime_semantics) = payload.runtime_semantics() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload.semantic_code(),
            domain_contribution.request_identity().clone(),
        )));
    };

    let declaration = match declare_projection_consumption(
        runtime_semantics.source().clone(),
        runtime_semantics.binding().clone(),
        runtime_semantics.requested_facts().clone(),
    ) {
        Ok(declaration) => declaration,
        Err(error) => {
            return Ok(TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
                WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                "consequence-aftermath",
                domain_contribution.target().kind(),
                domain_contribution.request_identity().clone(),
                format!(
                    "projection consumption declaration denied for `{}` with `{}`",
                    payload.semantic_code(),
                    declaration_error_label(&error),
                ),
            )));
        }
    };

    let support_report = discover_projection_consumption_support(runtime_semantics.source())?;
    let eligibility = evaluate_projection_consumption_eligibility(&declaration)?;

    Ok(TransitionOutcome::Success(
        WorthQueryAftermathProjectionConsumptionReview {
            semantic_code: payload.semantic_code().to_string(),
            request_identity: domain_contribution.request_identity().clone(),
            target_kind: domain_contribution.target().kind(),
            declaration,
            support_report,
            eligibility,
        },
    ))
}

pub(crate) fn materialize_admitted_projection_consumption<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<AdmittedProjectionConsumption>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryAftermathProjectionTarget,
{
    Ok(match materialize_projection_consumption_review(contribution)? {
        TransitionOutcome::Success(review) => match review.eligibility() {
            ProjectionConsumptionEligibility::Admitted(admitted) => {
                TransitionOutcome::Success(admitted.clone())
            }
            ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, _) => {
                TransitionOutcome::Success(admitted.clone())
            }
            ProjectionConsumptionEligibility::Denied(denied) => {
                TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
                    WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                    "consequence-aftermath",
                    review.target_kind(),
                    review.request_identity().clone(),
                    format!(
                        "projection consumption eligibility denied for `{}` with `{}`",
                        review.semantic_code(),
                        denial_reason_label(denied.reason()),
                    ),
                ))
            }
            ProjectionConsumptionEligibility::Deferred(deferred) => {
                TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
                    WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                    "consequence-aftermath",
                    review.target_kind(),
                    review.request_identity().clone(),
                    format!(
                        "projection consumption eligibility deferred for `{}` with `{}`",
                        review.semantic_code(),
                        deferred_reason_label(deferred.reason()),
                    ),
                ))
            }
            ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
                TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
                    WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics,
                    "consequence-aftermath",
                    review.target_kind(),
                    review.request_identity().clone(),
                    format!(
                        "projection consumption source mismatch for `{}` with `{}` / `{}`",
                        review.semantic_code(),
                        mismatch.source_family().as_str(),
                        mismatch.requested_fact_kind().as_str(),
                    ),
                ))
            }
        },
        TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
        TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
        TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
        TransitionOutcome::Deferred(never) => match never {},
    })
}

pub(crate) fn materialize_projection_consumption_contract<T>(
    contribution: WorthQueryMaterializationReadyAftermathContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<MaterializedProjectionContract>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryAftermathProjectionTarget,
{
    Ok(
        match materialize_admitted_projection_consumption(contribution)? {
            TransitionOutcome::Success(admitted) => {
                TransitionOutcome::Success(admitted.bind_contract()?)
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
            TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
            TransitionOutcome::Deferred(never) => match never {},
        },
    )
}

pub(crate) trait WorthQueryAftermathProjectionTarget:
    WorthQueryDomainCapabilityTargetBinding
{
}

impl WorthQueryAftermathProjectionTarget for WorthQueryAdmittedPlanBoundContributionTarget {}

impl WorthQueryAftermathProjectionTarget
    for crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget
{
}

fn missing_runtime_semantics_denial(
    semantic_code: &str,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "consequence-aftermath",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "projection consumption materialization requires runtime aftermath semantics for `{semantic_code}`"
        ),
    )
}

fn declaration_error_label(error: &ProjectionConsumptionDeclarationError) -> &'static str {
    match error {
        ProjectionConsumptionDeclarationError::NoRequestedFacts => "no-requested-facts",
        ProjectionConsumptionDeclarationError::SourceAuthorizedProjectionQueryMismatch {
            ..
        } => "source-authorized-projection-query-mismatch",
        ProjectionConsumptionDeclarationError::BindingAuthorizedProjectionResultShapeMismatch {
            ..
        } => "binding-authorized-projection-result-shape-mismatch",
        ProjectionConsumptionDeclarationError::SourceBindingResultShapeMismatch { .. } => {
            "source-binding-result-shape-mismatch"
        }
    }
}

fn denial_reason_label(reason: &ProjectionConsumptionDenialReason) -> String {
    match reason {
        ProjectionConsumptionDenialReason::FactFamilyNotVisible { field_key } => {
            let mut label = String::from("fact-family-not-visible:");
            label.push_str(field_key);
            label
        }
    }
}

fn deferred_reason_label(reason: &DeferredProjectionConsumptionReason) -> &'static str {
    match reason {
        DeferredProjectionConsumptionReason::WriteReceiptContractBindingPending => {
            "write-receipt-contract-binding-pending"
        }
        DeferredProjectionConsumptionReason::SourceFamilySupportPending => {
            "source-family-support-pending"
        }
    }
}
