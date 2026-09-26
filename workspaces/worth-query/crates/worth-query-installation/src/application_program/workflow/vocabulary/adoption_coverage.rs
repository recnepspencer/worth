//! The exact node vocabulary one installed workflow spec executes at one
//! program revision, compared against retained definition dependency facts.

use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowSpec,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

use super::WorthQueryInstalledApplicationWorkflowSpec;

/// One node a published workflow definition depends on, exactly as its
/// retained definition facts record it.
///
/// Adoption reads these from owner truth; it never reconstructs a compiled
/// plan to learn what a definition needs.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryWorkflowNodeDependency {
    Operation {
        identifier: String,
        input_type: String,
        binding: Option<String>,
        requires_authority: bool,
    },
    Assessment {
        identifier: String,
        parameter_type: String,
        result_type: String,
        binding: String,
    },
    Condition {
        identifier: String,
        parameter_type: String,
        result_type: String,
        binding: String,
    },
    Approval {
        identifier: String,
        capability_type: String,
        operation: String,
        capability_identity: String,
    },
}

impl WorthQueryWorkflowNodeDependency {
    /// Names this node supplies to program-change comparison: the operation
    /// or query identifier and, when present, its installed binding.
    pub fn program_names(&self) -> impl Iterator<Item = &str> {
        let (identifier, binding) = match self {
            Self::Operation {
                identifier,
                binding,
                ..
            } => (identifier.as_str(), binding.as_deref()),
            Self::Assessment {
                identifier,
                binding,
                ..
            }
            | Self::Condition {
                identifier,
                binding,
                ..
            } => (identifier.as_str(), Some(binding.as_str())),
            // The capability identity is compared through coverage lookup;
            // only the authorized operation's name enters program comparison.
            Self::Approval { operation, .. } => (operation.as_str(), None),
        };
        std::iter::once(identifier).chain(binding)
    }
}

/// Installed workflow vocabulary one supported spec executes at one program
/// revision.
///
/// Only an installed spec mints coverage, so holding one proves the host can
/// still execute each covered node. Its identity is the spec's installed
/// support identity, which already digests the program revision, the spec, and
/// every supported member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowVocabularyCoverage {
    schema_binding: ApplicationSchemaBindingIdentity,
    program_revision: ApplicationProgramRevision,
    spec: String,
    identity: [u8; 32],
    nodes: Box<[WorthQueryWorkflowNodeDependency]>,
}

impl WorthQueryWorkflowVocabularyCoverage {
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn program_revision(&self) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    /// Whether this vocabulary executes the node exactly as retained.
    pub fn covers(&self, node: &WorthQueryWorkflowNodeDependency) -> bool {
        self.nodes.binary_search(node).is_ok()
    }
}

impl<Schema, Spec, Program> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// The exact node vocabulary this installed spec executes, for program
    /// adoption to compare against retained definition facts.
    pub fn adoption_coverage(&self) -> WorthQueryWorkflowVocabularyCoverage {
        let operations =
            self.operations
                .iter()
                .map(|operation| WorthQueryWorkflowNodeDependency::Operation {
                    identifier: operation.identifier.to_owned(),
                    input_type: operation.input_type.as_str().to_owned(),
                    binding: Some(operation.binding_identity.to_owned()),
                    requires_authority: operation.requires_workflow_authority,
                });
        // An operation node declared without a mutation binding runs the
        // installed operation directly and never requires workflow authority.
        let direct = self
            .operations
            .iter()
            .filter(|operation| !operation.requires_workflow_authority)
            .map(|operation| WorthQueryWorkflowNodeDependency::Operation {
                identifier: operation.identifier.to_owned(),
                input_type: operation.input_type.as_str().to_owned(),
                binding: None,
                requires_authority: false,
            });
        let assessments = self.assessments.iter().map(|assessment| {
            WorthQueryWorkflowNodeDependency::Assessment {
                identifier: assessment.query_identifier.to_owned(),
                parameter_type: assessment.parameter_type.as_str().to_owned(),
                result_type: assessment.result_type.as_str().to_owned(),
                binding: assessment.binding_identity.to_owned(),
            }
        });
        let conditions =
            self.conditions
                .iter()
                .map(|condition| WorthQueryWorkflowNodeDependency::Condition {
                    identifier: condition.query_identifier.to_owned(),
                    parameter_type: condition.parameter_type.as_str().to_owned(),
                    result_type: condition.result_type.as_str().to_owned(),
                    binding: condition.binding_identity.to_owned(),
                });
        let approvals =
            self.approvals
                .iter()
                .map(|approval| WorthQueryWorkflowNodeDependency::Approval {
                    identifier: approval.binding.identifier.to_owned(),
                    capability_type: approval.binding.capability_type.as_str().to_owned(),
                    operation: approval.binding.operation_identifier.to_owned(),
                    capability_identity: hex(&approval.binding.installed_identity),
                });
        let mut nodes = operations
            .chain(direct)
            .chain(assessments)
            .chain(conditions)
            .chain(approvals)
            .collect::<Vec<_>>();
        nodes.sort_unstable();
        nodes.dedup();
        WorthQueryWorkflowVocabularyCoverage {
            schema_binding: self.schema_binding.clone(),
            program_revision: self.program_revision.clone(),
            spec: Spec::IDENTITY.as_str().to_owned(),
            identity: self.support_identity,
            nodes: nodes.into_boxed_slice(),
        }
    }
}

fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}
