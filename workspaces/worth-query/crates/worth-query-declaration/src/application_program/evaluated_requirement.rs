use crate::application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema};

/// Domain-owned conditional requirement evaluated through an installed program rule.
pub trait ApplicationEvaluatedRequirementRule<Schema, Operation>: Sized + 'static
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
{
    const IDENTITY: &'static str;
    type Context;
    type Requirement: Clone + std::fmt::Debug + Eq + PartialEq;
    type Finding: Clone + std::fmt::Debug + Eq + PartialEq;

    fn evaluate(
        context: &Self::Context,
    ) -> ApplicationEvaluatedRequirement<Self::Requirement, Self::Finding>;
}

/// One rule evaluation shared by submission enforcement and input guidance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationEvaluatedRequirement<Requirement, Finding> {
    required: Option<Requirement>,
    finding: Finding,
}

impl<Requirement, Finding> ApplicationEvaluatedRequirement<Requirement, Finding> {
    pub const fn required(requirement: Requirement, finding: Finding) -> Self {
        Self {
            required: Some(requirement),
            finding,
        }
    }

    pub const fn not_required(finding: Finding) -> Self {
        Self {
            required: None,
            finding,
        }
    }

    pub const fn required_input_guidance(&self) -> Option<&Requirement> {
        self.required.as_ref()
    }

    pub const fn finding(&self) -> &Finding {
        &self.finding
    }
}

impl<Requirement, Finding> ApplicationEvaluatedRequirement<Requirement, Finding>
where
    Requirement: Clone,
    Finding: Clone,
{
    pub fn enforce_submission(
        &self,
        supplied: bool,
    ) -> Result<(), ApplicationRequirementSubmissionDenial<Requirement, Finding>> {
        match (&self.required, supplied) {
            (Some(requirement), false) => Err(ApplicationRequirementSubmissionDenial {
                missing: requirement.clone(),
                finding: self.finding.clone(),
            }),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRequirementSubmissionDenial<Requirement, Finding> {
    missing: Requirement,
    finding: Finding,
}

impl<Requirement, Finding> ApplicationRequirementSubmissionDenial<Requirement, Finding> {
    pub const fn missing(&self) -> &Requirement {
        &self.missing
    }

    pub const fn finding(&self) -> &Finding {
        &self.finding
    }
}
