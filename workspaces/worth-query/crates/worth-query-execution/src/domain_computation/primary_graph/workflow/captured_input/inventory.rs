//! One proposal's exact qualified target contracts, in canonical node order.
use super::*;
use crate::domain_computation::primary_graph::{
    workflow::definition::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind},
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::{
    canonical_indexed_operation_material, canonical_operation_material,
};
use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowCapturedTarget {
    pub node: String,
    pub contract: WorkflowCapturedInputContract,
}
/// Descriptive native contracts plus publication dependencies, not a retained input registry.
pub struct WorkflowCapturedInputs {
    origin_binding: Option<&'static str>,
    pub(in crate::domain_computation::primary_graph) targets: Vec<WorkflowCapturedTarget>,
    pub(in crate::domain_computation::primary_graph) identity: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) facts: Vec<WorthQueryApplicationObservedFact>,
}
impl WorkflowCapturedInputs {
    pub(in crate::domain_computation::primary_graph) fn empty() -> Self {
        Self {
            origin_binding: None,
            targets: Vec::new(),
            identity: None,
            facts: Vec::new(),
        }
    }
    pub fn prepare<S: ApplicationSchema, B: ApplicationMutationBinding<S>, C>(
        access: &WorthQueryProgramPublicationAccess,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        admission: &mut WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, C>,
        inputs: Vec<(
            ApplicationWorkflowNodeIdentity,
            PreparedWorkflowCapturedInput,
        )>,
    ) -> Result<Self, WorkflowCapturedInputDenial> {
        let mut result = Self::empty();
        if !inputs.is_empty() {
            if !B::WORKFLOW_CONTROL || B::REQUIRES_WORKFLOW_AUTHORITY {
                return Err(WorkflowCapturedInputDenial::Binding);
            }
            result.origin_binding = Some(B::IDENTITY);
        }
        let basis = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            selected.product().observation(),
        );
        for (node, mut input) in inputs {
            if input.runtime != selected.application().runtime.authority_identity().as_u64()
                || input.basis != basis
                || input.contract.resource != admission.scope_entity_id()
            {
                return Err(WorkflowCapturedInputDenial::ForeignSelection);
            }
            admission.record_request_identity_work(access, input.request_work);
            admission.retain_execution_canonical_work(input.precondition_work);
            result.targets.push(WorkflowCapturedTarget {
                node: node.as_str().to_owned(),
                contract: input.contract,
            });
            result.facts.append(&mut input.facts);
        }
        result.targets.sort_by(|a, b| a.node.cmp(&b.node));
        if result
            .targets
            .windows(2)
            .any(|pair| pair[0].node == pair[1].node)
        {
            return Err(WorkflowCapturedInputDenial::DuplicateTarget);
        }
        result.identity = inventory_identity(&result.targets);
        Ok(result)
    }
    pub fn bind_idempotency(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        binding.bind_workflow_captured_inputs(self.identity)
    }
    pub(in crate::domain_computation::primary_graph) fn validate(
        &self,
        compiled: &CompiledWorkflowDefinition,
        source: EntityId,
    ) -> Result<(), WorkflowCapturedInputDenial> {
        let mut expected = compiled.captured_input_targets(source).collect::<Vec<_>>();
        if !expected.is_empty() {
            let source = compiled
                .node(source)
                .ok_or(WorkflowCapturedInputDenial::TargetMismatch)?;
            if !matches!(source.kind(), CompiledWorkflowNodeKind::Operation {
                binding: Some(binding), requires_workflow_authority: false, ..
            } if Some(binding.as_str()) == self.origin_binding)
            {
                return Err(WorkflowCapturedInputDenial::TargetMismatch);
            }
        }
        expected.sort_by(|a, b| a.path().cmp(b.path()));
        if expected.len() != self.targets.len() {
            return Err(WorkflowCapturedInputDenial::TargetMismatch);
        }
        for (node, captured) in expected.into_iter().zip(&self.targets) {
            let CompiledWorkflowNodeKind::Operation {
                binding: Some(binding),
                input_type,
                requires_workflow_authority: true,
                ..
            } = node.kind()
            else {
                return Err(WorkflowCapturedInputDenial::TargetMismatch);
            };
            if node.path() != captured.node
                || binding != &captured.contract.binding
                || input_type != &captured.contract.input_type
            {
                return Err(WorkflowCapturedInputDenial::TargetMismatch);
            }
        }
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph) fn inventory_identity(
    targets: &[WorkflowCapturedTarget],
) -> Option<[u8; 32]> {
    if targets.is_empty() {
        return None;
    }
    Some(
        Sha256::digest(
            canonical_indexed_operation_material(
                "workflow.captured-input",
                targets.iter().map(|target| target.material()),
            )
            .as_bytes(),
        )
        .into(),
    )
}
impl WorkflowCapturedTarget {
    pub(in crate::domain_computation::primary_graph) fn material(&self) -> String {
        let c = &self.contract;
        let mut fields = vec![
            ("node", self.node.clone()),
            ("binding", c.binding.clone()),
            ("input-type", c.input_type.clone()),
            ("input", hex(c.input)),
            ("key", hex(c.key)),
            (
                "resource.partition",
                c.resource.partition_value().to_string(),
            ),
            ("resource.slot", c.resource.local_slot_value().to_string()),
            (
                "resource.generation",
                c.resource.generation_value().to_string(),
            ),
            ("source.kind", c.source.map_or(0, |s| s.0).to_string()),
        ];
        if let Some((_, identity, partition)) = c.source {
            fields.push(("source", hex(identity)));
            fields.push(("source.partition", hex(partition)));
        }
        if let Some(identity) = c.preconditions {
            fields.push(("preconditions", hex(identity)));
        }
        canonical_operation_material(fields)
    }
}
fn hex(identity: [u8; 32]) -> String {
    use std::fmt::Write;
    identity
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            write!(&mut text, "{byte:02x}").expect("String write");
            text
        })
}
