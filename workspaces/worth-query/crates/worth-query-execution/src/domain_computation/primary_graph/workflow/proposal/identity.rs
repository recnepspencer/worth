use sha2::{Digest, Sha256};

use crate::domain_computation::{
    canonical_indexed_operation_material, canonical_operation_material,
};

pub(in crate::domain_computation::primary_graph) struct WorkflowProposalMeaning {
    pub(super) captured_identity: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) captured:
        Vec<super::super::captured_input::WorkflowCapturedTarget>,
    pub(super) identity: String,
    pub(super) proposer: worth_relational::facade::identity::EntityId,
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
    captured_identity: Option<[u8; 32]>,
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
            captured_identity: None,
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
            input_identity: input_identity.to_owned(),
            source_identity: source_identity.map(str::to_owned),
        }
    }

    pub(super) fn with_captured(mut self, identity: Option<[u8; 32]>) -> Self {
        self.captured_identity = identity;
        self
    }

    pub(super) fn coverage_identity(&self, local_identity: &str) -> String {
        let mut fields = vec![
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
        ];
        if let Some(identity) = self.captured_identity {
            fields.push(("workflow.coverage.scope.captured-inputs", encode(identity)));
        }
        let material = canonical_operation_material(fields);
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
    pub(in crate::domain_computation::primary_graph) fn take_captured_support(
        &mut self,
    ) -> Result<
        Option<std::sync::Arc<super::super::captured_input::WorkflowCapturedInputSupport>>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        self.captured_identity
            .map(|identity| {
                super::super::captured_input::WorkflowCapturedInputSupport::validate(
                    self.identity.clone(),
                    identity,
                    std::mem::take(&mut self.captured),
                )
                .map(std::sync::Arc::new)
            })
            .transpose()
    }
    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }
}

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) fn derive_workflow_proposal(
    transition_identity: &str,
    operation: &str,
    input_type: &str,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    node_path: &str,
    coverages: Vec<WorkflowProposalCoverageMeaning>,
    proposer: worth_relational::facade::identity::EntityId,
) -> WorkflowProposalMeaning {
    derive_captured_workflow_proposal(
        transition_identity,
        operation,
        input_type,
        input_identity,
        source_identity,
        node_path,
        coverages,
        proposer,
        None,
    )
}

pub(in crate::domain_computation::primary_graph) fn derive_captured_workflow_proposal(
    transition_identity: &str,
    operation: &str,
    input_type: &str,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    node_path: &str,
    mut coverages: Vec<WorkflowProposalCoverageMeaning>,
    proposer: worth_relational::facade::identity::EntityId,
    captured_identity: Option<[u8; 32]>,
) -> WorkflowProposalMeaning {
    coverages.sort_by(|left, right| left.selector.cmp(&right.selector));
    let input_identity = encode(input_identity);
    let source_identity = source_identity.map(encode);
    let scope = WorkflowProposalCoverageScope::from_encoded(
        operation,
        input_type,
        &input_identity,
        source_identity.as_deref(),
    )
    .with_captured(captured_identity);
    for coverage in &mut coverages {
        coverage.identity = scope.coverage_identity(&coverage.local_identity);
    }
    let mut fields = vec![
        (
            "workflow.proposer.partition",
            proposer.partition_value().to_string(),
        ),
        (
            "workflow.proposer.slot",
            proposer.local_slot_value().to_string(),
        ),
        (
            "workflow.proposer.generation",
            proposer.generation_value().to_string(),
        ),
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
    if let Some(identity) = captured_identity {
        fields.push(("workflow.captured-inputs", encode(identity)));
    }
    let material = canonical_operation_material(fields);
    let identity_bytes: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    WorkflowProposalMeaning {
        captured_identity,
        captured: Vec::new(),
        proposer,
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
#[path = "identity/tests.rs"]
mod tests;
