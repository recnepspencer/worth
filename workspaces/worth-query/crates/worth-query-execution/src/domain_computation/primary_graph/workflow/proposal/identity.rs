use sha2::{Digest, Sha256};

use crate::domain_computation::{
    canonical_indexed_operation_material, canonical_operation_material,
};

pub(in crate::domain_computation::primary_graph) struct WorkflowProposalMeaning {
    pub(super) identity: String,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) input_identity: String,
    pub(super) source_identity: Option<String>,
    pub(super) node_path: String,
    pub(super) coverages: Box<[WorkflowProposalCoverageMeaning]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowProposalCoverageMeaning {
    pub(in crate::domain_computation::primary_graph) identity: String,
    local_identity: String,
    pub(in crate::domain_computation::primary_graph) selector:
        worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
    pub(in crate::domain_computation::primary_graph) subject:
        worth_relational::facade::identity::EntityId,
}

/// Proposal meaning shared by every coverage in one authored proposal.
///
/// The transition identity is intentionally absent: a later occurrence of the
/// same proposal does not revise a subject's evidence. The selected coverage
/// identity is added by `coverage_identity`, so an unrelated subject can revise
/// the proposal as a whole without revising this subject's scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorkflowProposalCoverageScope {
    operation: String,
    input_type: String,
    input_identity: String,
    source_identity: Option<String>,
}

impl WorkflowProposalCoverageScope {
    pub(super) fn from_identities(
        operation: &str,
        input_type: &str,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
    ) -> Self {
        let input_identity = encode(input_identity);
        let source_identity = source_identity.map(encode);
        Self::from_encoded(
            operation,
            input_type,
            &input_identity,
            source_identity.as_deref(),
        )
    }

    pub(super) fn from_encoded(
        operation: &str,
        input_type: &str,
        input_identity: &str,
        source_identity: Option<&str>,
    ) -> Self {
        Self {
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
            input_identity: input_identity.to_owned(),
            source_identity: source_identity.map(str::to_owned),
        }
    }

    pub(super) fn coverage_identity(&self, local_identity: &str) -> String {
        let material = canonical_operation_material(vec![
            ("workflow.coverage.scope.operation", self.operation.clone()),
            (
                "workflow.coverage.scope.input-type",
                self.input_type.clone(),
            ),
            ("workflow.coverage.scope.input", self.input_identity.clone()),
            (
                "workflow.coverage.scope.source",
                self.source_identity
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
            ),
            (
                "workflow.coverage.scope.coverage",
                local_identity.to_owned(),
            ),
        ]);
        encode(Sha256::digest(material.as_bytes()).into())
    }
}

impl WorkflowProposalCoverageMeaning {
    pub(in crate::domain_computation::primary_graph) fn new(
        selector: worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
        subject: worth_relational::facade::identity::EntityId,
    ) -> Self {
        let identity = coverage_identity(&selector.persistence_identity(), subject);
        Self {
            identity: identity.clone(),
            local_identity: identity,
            selector,
            subject,
        }
    }

    pub(super) fn local_identity(&self) -> &str {
        &self.local_identity
    }
}

impl WorkflowProposalMeaning {
    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }
}

pub(in crate::domain_computation::primary_graph) fn derive_workflow_proposal(
    transition_identity: &str,
    operation: &str,
    input_type: &str,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    node_path: &str,
    mut coverages: Vec<WorkflowProposalCoverageMeaning>,
) -> WorkflowProposalMeaning {
    coverages.sort_by(|left, right| left.selector.cmp(&right.selector));
    let input_identity = encode(input_identity);
    let source_identity = source_identity.map(encode);
    let scope = WorkflowProposalCoverageScope::from_encoded(
        operation,
        input_type,
        &input_identity,
        source_identity.as_deref(),
    );
    for coverage in &mut coverages {
        coverage.identity = scope.coverage_identity(&coverage.local_identity);
    }
    let mut fields = vec![
        ("workflow.transition", transition_identity.to_owned()),
        ("workflow.operation", operation.to_owned()),
        ("workflow.input-type", input_type.to_owned()),
        ("workflow.input", input_identity.clone()),
        (
            "workflow.source",
            source_identity.clone().unwrap_or_else(|| "none".to_owned()),
        ),
    ];
    fields.push((
        "workflow.coverage-inventory",
        canonical_indexed_operation_material(
            "workflow.coverage",
            coverages.iter().map(|coverage| coverage.identity.clone()),
        ),
    ));
    let material = canonical_operation_material(fields);
    let identity_bytes: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    WorkflowProposalMeaning {
        identity: encode(identity_bytes),
        operation: operation.to_owned(),
        input_type: input_type.to_owned(),
        input_identity,
        source_identity,
        node_path: node_path.to_owned(),
        coverages: coverages.into_boxed_slice(),
    }
}

fn coverage_identity(
    selector: &str,
    subject: worth_relational::facade::identity::EntityId,
) -> String {
    encode(
        Sha256::digest(
            canonical_operation_material(vec![
                ("workflow.coverage.selector", selector.to_owned()),
                (
                    "workflow.coverage.partition",
                    subject.partition_value().to_string(),
                ),
                (
                    "workflow.coverage.slot",
                    subject.local_slot_value().to_string(),
                ),
                (
                    "workflow.coverage.generation",
                    subject.generation_value().to_string(),
                ),
            ])
            .as_bytes(),
        )
        .into(),
    )
}

pub(in crate::domain_computation::primary_graph) fn derive_workflow_proposal_context_identity(
    instance: worth_relational::facade::identity::EntityId,
) -> [u8; 32] {
    Sha256::digest(
        canonical_operation_material(vec![
            (
                "workflow.instance.partition",
                instance.partition_value().to_string(),
            ),
            (
                "workflow.instance.slot",
                instance.local_slot_value().to_string(),
            ),
            (
                "workflow.instance.generation",
                instance.generation_value().to_string(),
            ),
        ])
        .as_bytes(),
    )
    .into()
}

fn encode(identity: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").expect("writing a workflow identity to String cannot fail");
    }
    text
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use super::{derive_workflow_proposal, WorkflowProposalCoverageMeaning};

    #[test]
    fn changing_one_subject_changes_only_its_coverage_identity() {
        let first_a = WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 10, 1),
        );
        let second_a = WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 11, 1),
        );
        let stable_b = WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Related,
            EntityId::new(PartitionId::new(1), 20, 1),
        );
        let first = derive_workflow_proposal(
            "transition",
            "operation",
            "input",
            [1; 32],
            None,
            "proposal",
            vec![first_a.clone(), stable_b.clone()],
        );
        let second = derive_workflow_proposal(
            "transition",
            "operation",
            "input",
            [1; 32],
            None,
            "proposal",
            vec![second_a.clone(), stable_b.clone()],
        );

        assert_ne!(first.identity, second.identity);
        assert_ne!(first_a.identity, second_a.identity);
        assert_eq!(first.coverages[1].identity, second.coverages[1].identity);
    }

    #[test]
    fn changing_shared_proposal_meaning_changes_each_coverage_scope() {
        let coverage = WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 10, 1),
        );
        let first = derive_workflow_proposal(
            "transition",
            "operation",
            "input",
            [1; 32],
            Some([2; 32]),
            "proposal",
            vec![coverage.clone()],
        );
        let changed = derive_workflow_proposal(
            "transition",
            "changed-operation",
            "input",
            [1; 32],
            Some([2; 32]),
            "proposal",
            vec![coverage],
        );
        let changed_input = derive_workflow_proposal(
            "transition",
            "operation",
            "input",
            [4; 32],
            Some([2; 32]),
            "proposal",
            vec![WorkflowProposalCoverageMeaning::new(
                ApplicationWorkflowSubjectSelector::Resource,
                EntityId::new(PartitionId::new(1), 10, 1),
            )],
        );
        let changed_source = derive_workflow_proposal(
            "transition",
            "operation",
            "input",
            [1; 32],
            Some([3; 32]),
            "proposal",
            vec![WorkflowProposalCoverageMeaning::new(
                ApplicationWorkflowSubjectSelector::Resource,
                EntityId::new(PartitionId::new(1), 10, 1),
            )],
        );

        assert_ne!(first.coverages[0].identity, changed.coverages[0].identity);
        assert_ne!(
            first.coverages[0].identity,
            changed_input.coverages[0].identity
        );
        assert_ne!(
            first.coverages[0].identity,
            changed_source.coverages[0].identity
        );
    }

    #[test]
    fn changing_only_transition_occurrence_preserves_each_coverage_scope() {
        let coverage = WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 10, 1),
        );
        let first = derive_workflow_proposal(
            "transition/first-occurrence",
            "operation",
            "input",
            [1; 32],
            None,
            "proposal",
            vec![coverage.clone()],
        );
        let later = derive_workflow_proposal(
            "transition/later-occurrence",
            "operation",
            "input",
            [1; 32],
            None,
            "proposal",
            vec![coverage],
        );

        assert_ne!(first.identity, later.identity);
        assert_eq!(first.coverages[0].identity, later.coverages[0].identity);
    }
}
