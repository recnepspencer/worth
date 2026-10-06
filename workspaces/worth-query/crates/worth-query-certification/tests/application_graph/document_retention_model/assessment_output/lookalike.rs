//! A lookalike output family that shares the real family's identity but
//! always reports a failing assessment.

use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerApplicability,
    WorthQueryProducerOutputFamily, WorthQueryWorkflowAssessmentOutputFamily,
    WorthQueryWorkflowAssessmentPosture,
};

use super::super::{
    retention_entry::{DocumentRetentionQueryBinding, DocumentRetentionRead},
    schema::{Document, DocumentRetentionRow, DocumentRetentionSchema},
};
use super::{RetentionAssessmentOutputFamily, APPLICABILITY};

pub struct LookalikeRetentionAssessmentOutputFamily;

impl WorthQueryProducerOutputFamily<DocumentRetentionSchema>
    for LookalikeRetentionAssessmentOutputFamily
{
    type Source = DocumentRetentionQueryBinding;
    type Entity = Document;
    const IDENTITY: &'static str = RetentionAssessmentOutputFamily::IDENTITY;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;

    fn profile_kind(_: &DocumentRetentionRow) -> &'static str {
        "retention-assessment"
    }
}

impl WorthQueryWorkflowAssessmentOutputFamily<DocumentRetentionSchema>
    for LookalikeRetentionAssessmentOutputFamily
{
    fn assessment_posture(_: &DocumentRetentionRow) -> WorthQueryWorkflowAssessmentPosture {
        WorthQueryWorkflowAssessmentPosture::Failing
    }
}

pub struct LookalikeRetentionAssessmentDemand {
    identity: String,
}

impl LookalikeRetentionAssessmentDemand {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }
}

impl WorthQueryApplicationOutputDemand<DocumentRetentionSchema>
    for LookalikeRetentionAssessmentDemand
{
    type OutputFamily = LookalikeRetentionAssessmentOutputFamily;

    fn source_intent(&self) -> DocumentRetentionRead {
        DocumentRetentionRead {
            identity: self.identity.clone(),
        }
    }
}
