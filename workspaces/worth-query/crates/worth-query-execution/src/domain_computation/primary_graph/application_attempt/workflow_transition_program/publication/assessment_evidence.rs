#[derive(Debug)]
pub struct PerformedWorkflowAssessmentEvidence {
    pub(super) evidence: worth_relational::facade::identity::EntityId,
    pub(super) identity: String,
    pub(super) producer: String,
    pub(super) family: String,
    pub(super) query: String,
    pub(super) parameter_type: String,
    pub(super) result_type: String,
    pub(super) binding: String,
    pub(super) subject: worth_relational::facade::identity::EntityId,
    pub(super) source_identity: String,
    pub(super) passing: bool,
    pub(super) publication_identity: String,
    pub(super) output_content_identity: String,
}

impl PerformedWorkflowAssessmentEvidence {
    pub const fn evidence(&self) -> worth_relational::facade::identity::EntityId {
        self.evidence
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn producer(&self) -> &str {
        &self.producer
    }
    pub fn family(&self) -> &str {
        &self.family
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn parameter_type(&self) -> &str {
        &self.parameter_type
    }
    pub fn result_type(&self) -> &str {
        &self.result_type
    }
    pub fn binding(&self) -> &str {
        &self.binding
    }
    pub const fn subject(&self) -> worth_relational::facade::identity::EntityId {
        self.subject
    }
    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }
    pub const fn passing(&self) -> bool {
        self.passing
    }
    pub fn publication_identity(&self) -> &str {
        &self.publication_identity
    }
    pub fn output_content_identity(&self) -> &str {
        &self.output_content_identity
    }
}
