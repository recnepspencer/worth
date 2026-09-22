use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::{
    InstalledWorkflowAdvanceCapability, InstalledWorkflowApproval, InstalledWorkflowAssessment,
    InstalledWorkflowAuthoringCapability, InstalledWorkflowCapabilityBinding,
    InstalledWorkflowCondition, InstalledWorkflowInstanceStartCapability,
    InstalledWorkflowOperation, WorthQueryApplicationWorkflowResourceCeiling,
    WorthQueryInstalledApplicationWorkflowSpec,
};

impl<Schema, Spec, Program> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
where
    Schema: super::ApplicationSchema,
    Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
        Schema = Schema,
    >,
{
    #[doc(hidden)]
    pub const fn support_identity_bytes(&self) -> &[u8; 32] {
        &self.support_identity
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn derive(
    program: &ApplicationProgramRevision,
    vocabulary: &str,
    authoring: &InstalledWorkflowAuthoringCapability,
    start: &InstalledWorkflowInstanceStartCapability,
    advance: &InstalledWorkflowAdvanceCapability,
    operations: &[InstalledWorkflowOperation],
    assessments: &[InstalledWorkflowAssessment],
    conditions: &[InstalledWorkflowCondition],
    approvals: &[InstalledWorkflowApproval],
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> [u8; 32] {
    let mut entries = Vec::new();
    entries.push(encoded("program", program.as_bytes()));
    entries.push(encoded("vocabulary", vocabulary.as_bytes()));
    entries.push(capability("authoring", &authoring.binding));
    entries.push(capability("start", &start.binding));
    entries.push(capability("advance", &advance.binding));
    entries.extend(operations.iter().map(|operation| {
        fields(
            "operation",
            &[operation.identifier, operation.input_type.as_str()],
        )
    }));
    entries.extend(assessments.iter().map(|assessment| {
        fields(
            "assessment",
            &[
                assessment.query_identifier,
                assessment.parameter_type.as_str(),
                assessment.result_type.as_str(),
                assessment.binding_identity,
            ],
        )
    }));
    entries.extend(conditions.iter().map(|condition| {
        fields(
            "condition",
            &[
                condition.query_identifier,
                condition.parameter_type.as_str(),
                condition.result_type.as_str(),
                condition.binding_identity,
            ],
        )
    }));
    entries.extend(
        approvals
            .iter()
            .map(|approval| capability("approval", &approval.binding)),
    );
    let components = resources.component_limits();
    let resource_values = [
        resources.maximum_definition_nodes().to_string(),
        resources.maximum_definition_connections().to_string(),
        resources.maximum_definition_effects().to_string(),
        components.maximum_occurrences().to_string(),
        components.maximum_depth().to_string(),
        components.maximum_node_provenance().to_string(),
        components.maximum_connection_provenance().to_string(),
        components.maximum_port_provenance().to_string(),
        resources.maximum_canonical_bytes().to_string(),
        resources.maximum_live_instances().to_string(),
        resources
            .maximum_retained_transitions_per_instance()
            .to_string(),
        resources.maximum_evidence_bytes().to_string(),
    ];
    entries.push(fields(
        "resources",
        &resource_values
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    ));
    entries.sort_unstable();
    let mut digest = Sha256::new();
    digest.update(b"worth.query.workflow.installed-support.v1");
    for entry in entries {
        frame(&mut digest, &entry);
    }
    digest.finalize().into()
}

fn capability(label: &str, binding: &InstalledWorkflowCapabilityBinding) -> Vec<u8> {
    let mut value = fields(
        label,
        &[
            binding.identifier,
            binding.capability_type.as_str(),
            binding.operation_identifier,
        ],
    );
    append_frame(&mut value, &binding.installed_identity);
    value
}

fn fields(label: &str, values: &[&str]) -> Vec<u8> {
    let mut encoded = encoded("kind", label.as_bytes());
    for value in values {
        append_frame(&mut encoded, value.as_bytes());
    }
    encoded
}

fn encoded(label: &str, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    append_frame(&mut encoded, label.as_bytes());
    append_frame(&mut encoded, value);
    encoded
}

fn append_frame(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
