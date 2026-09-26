//! Retargeting a declared vocabulary member as an untrusted draft names it.
//!
//! A decoded draft resolves each member against installed vocabulary, whose
//! refs were declared from types. These adjust only what an authored node
//! chooses: the subject an assessment reads, when it applies, and whether an
//! operation names its binding.

use crate::application_schema::{ApplicationSchemaDeclaration, ApplicationSchemaMember};

use super::{
    ApplicationWorkflowAssessmentApplicability, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowOperationRef, ApplicationWorkflowSubjectSelector,
};

impl ApplicationWorkflowOperationRef {
    /// The operation this binding performs, naming no binding, as `declared`
    /// builds it.
    pub fn without_binding(&self) -> Self {
        Self {
            binding: None,
            ..self.clone()
        }
    }
}

impl ApplicationWorkflowAssessmentRef {
    /// The same query read for `subject`, applying always.
    pub fn for_subject(mut self, subject: ApplicationWorkflowSubjectSelector) -> Self {
        self.subject = subject;
        self.applicability = ApplicationWorkflowAssessmentApplicability::Always;
        self
    }

    /// The same query read for the related subject only while `relation`
    /// joins it, as `declared_when_related_relation_present` builds it.
    /// `None` unless `declaration` declares that relation between `from`
    /// and `to`.
    pub fn when_related_relation_declared_in<Schema>(
        mut self,
        declaration: &ApplicationSchemaDeclaration<Schema>,
        relation: &str,
        from: &str,
        to: &str,
    ) -> Option<Self> {
        let declared = declaration.erased().members().iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Relation {
                    relation: declared,
                    from: declared_from,
                    to: declared_to,
                    ..
                } if declared == relation && declared_from == from && declared_to == to
            )
        });
        declared.then(|| {
            self.subject = ApplicationWorkflowSubjectSelector::Related;
            self.applicability =
                ApplicationWorkflowAssessmentApplicability::WhenRelatedRelationPresent {
                    relation: relation.to_owned(),
                    from: from.to_owned(),
                    to: to.to_owned(),
                };
            self
        })
    }
}
