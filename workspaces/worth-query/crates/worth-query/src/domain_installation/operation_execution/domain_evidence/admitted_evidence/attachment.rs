use super::super::super::{
    progression::evidence_completion::WorthQueryDirectDomainEvidenceAttachment,
    workflow_progression::evidence_validation::WorthQueryWorkflowDomainEvidenceAttachment,
};
use super::super::WorthQueryDomainEvidenceBinding;
use super::WorthQueryAdmittedDomainEvidence;

pub(in crate::domain_installation::operation_execution) enum WorthQueryCompletedDomainEvidenceAdmissionDenial
{
    Content(
        worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceAdmissionDenial,
    ),
    OutputOccurrenceMismatch { transformation_family: String },
}

impl WorthQueryCompletedDomainEvidenceAdmissionDenial {
    pub(in crate::domain_installation::operation_execution) fn kind(
        &self,
    ) -> worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceAdmissionDenialKind
{
        match self {
            Self::Content(denial) => denial.kind(),
            Self::OutputOccurrenceMismatch { .. } => {
                worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceAdmissionDenialKind::TransformationSummaryMismatch
            }
        }
    }

    pub(in crate::domain_installation::operation_execution) fn subject(&self) -> &str {
        match self {
            Self::Content(denial) => denial.subject(),
            Self::OutputOccurrenceMismatch {
                transformation_family,
            } => transformation_family,
        }
    }
}

pub(in crate::domain_installation::operation_execution) fn admit_direct_completion_content(
    attachment: WorthQueryDirectDomainEvidenceAttachment,
    material: Option<
        worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceMaterial,
    >,
) -> Result<
    Option<WorthQueryAdmittedDomainEvidence>,
    WorthQueryCompletedDomainEvidenceAdmissionDenial,
> {
    let content = worth_query_publication::facade::domain_computation::admit_domain_evidence_content(
        worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceContentAdmissionInput {
            contract: attachment.contract(),
            material,
            ledger: None,
        },
    )
    .map_err(WorthQueryCompletedDomainEvidenceAdmissionDenial::Content)?;
    attach_content(content, attachment.output_occurrence_identity(), || {
        WorthQueryDomainEvidenceBinding::from_direct(&attachment)
    })
}

pub(in crate::domain_installation::operation_execution) fn admit_workflow_completion_content(
    attachment: WorthQueryWorkflowDomainEvidenceAttachment,
    material: Option<
        worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceMaterial,
    >,
    ledger: &mut worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceAdmissionLedger,
) -> Result<
    Option<WorthQueryAdmittedDomainEvidence>,
    WorthQueryCompletedDomainEvidenceAdmissionDenial,
> {
    let content = worth_query_publication::facade::domain_computation::admit_domain_evidence_content(
        worth_query_publication::facade::domain_computation::WorthQueryDomainEvidenceContentAdmissionInput {
            contract: attachment.contract(),
            material,
            ledger: Some(ledger),
        },
    )
    .map_err(WorthQueryCompletedDomainEvidenceAdmissionDenial::Content)?;
    attach_content(content, attachment.output_occurrence_identity(), || {
        WorthQueryDomainEvidenceBinding::from_workflow(&attachment)
    })
}

fn attach_content(
    content: Option<
        worth_query_publication::facade::domain_computation::WorthQueryAdmittedDomainEvidenceContent,
    >,
    exact_output_occurrence_identity: &str,
    binding: impl FnOnce() -> WorthQueryDomainEvidenceBinding,
) -> Result<
    Option<WorthQueryAdmittedDomainEvidence>,
    WorthQueryCompletedDomainEvidenceAdmissionDenial,
> {
    let Some(content) = content else {
        return Ok(None);
    };
    if let Some(transformation) = content.core().transformation() {
        let claimed = transformation.parts().output_occurrence_identity.as_str();
        if claimed != exact_output_occurrence_identity {
            return Err(
                WorthQueryCompletedDomainEvidenceAdmissionDenial::OutputOccurrenceMismatch {
                    transformation_family: transformation.parts().transformation_family.clone(),
                },
            );
        }
    }
    Ok(Some(WorthQueryAdmittedDomainEvidence::attach(
        binding(),
        content,
    )))
}
