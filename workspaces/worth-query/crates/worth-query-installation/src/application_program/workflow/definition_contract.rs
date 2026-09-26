use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramRevision, ApplicationWorkflowNodeKind, ApplicationWorkflowSpec,
        ValidatedWorkflowDefinition,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};

use super::vocabulary::{
    denial, InstalledWorkflowAuthoringCapability, WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryInstalledApplicationWorkflowSpec,
};

pub struct WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    schema_binding: ApplicationSchemaBindingIdentity,
    program_revision: ApplicationProgramRevision,
    definition: ValidatedWorkflowDefinition<Spec>,
    assessment_bindings: Box<[(String, &'static str)]>,
    condition_bindings: Box<[(String, &'static str)]>,
    approval_bindings: Box<[WorthQueryInstalledWorkflowApprovalBinding]>,
    authoring_capability: InstalledWorkflowAuthoringCapability,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

#[doc(hidden)]
pub struct WorthQueryInstalledWorkflowApprovalBinding {
    pub node_path: String,
    pub operation: &'static str,
    pub installed_capability_identity: [u8; 32],
}

/// The validated definition and its installation-selected member bindings.
#[doc(hidden)]
pub struct WorthQueryInstalledWorkflowDefinitionParts<Spec: ApplicationWorkflowSpec> {
    pub definition: ValidatedWorkflowDefinition<Spec>,
    pub assessment_bindings: Box<[(String, &'static str)]>,
    pub condition_bindings: Box<[(String, &'static str)]>,
    pub approval_bindings: Box<[WorthQueryInstalledWorkflowApprovalBinding]>,
}

impl<Schema, Spec, Program> WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn program_revision(&self) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub fn definition(&self) -> &ValidatedWorkflowDefinition<Spec> {
        &self.definition
    }

    #[doc(hidden)]
    pub fn assessment_bindings(&self) -> &[(String, &'static str)] {
        &self.assessment_bindings
    }

    #[doc(hidden)]
    pub fn condition_bindings(&self) -> &[(String, &'static str)] {
        &self.condition_bindings
    }

    #[doc(hidden)]
    pub fn approval_bindings(&self) -> &[WorthQueryInstalledWorkflowApprovalBinding] {
        &self.approval_bindings
    }

    #[doc(hidden)]
    pub fn authoring_binding_matches<Capability, Operation>(&self) -> bool
    where
        Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<
                Schema = Schema,
            > + 'static,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    {
        self.authoring_capability.binding.matches_binding(
            std::any::TypeId::of::<Capability>(),
            Capability::IDENTIFIER,
            &Capability::PORTABLE_TYPE_IDENTITY,
            std::any::TypeId::of::<Operation>(),
            Operation::IDENTIFIER,
        )
    }

    #[doc(hidden)]
    pub fn authoring_capability_identity_bytes(&self) -> &[u8; 32] {
        &self.authoring_capability.binding.installed_identity
    }

    #[doc(hidden)]
    pub fn into_definition(self) -> WorthQueryInstalledWorkflowDefinitionParts<Spec> {
        WorthQueryInstalledWorkflowDefinitionParts {
            definition: self.definition,
            assessment_bindings: self.assessment_bindings,
            condition_bindings: self.condition_bindings,
            approval_bindings: self.approval_bindings,
        }
    }
}

pub(super) fn bind<Schema, Spec, Program>(
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
    definition: ValidatedWorkflowDefinition<Spec>,
) -> Result<
    WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
    WorthQueryApplicationWorkflowInstallationDenial,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    let limits = definition.limits();
    let resources = installed.resources;
    if limits.maximum_nodes() > resources.maximum_definition_nodes()
        || limits.maximum_connections() > resources.maximum_definition_connections()
        || limits.maximum_effects() > resources.maximum_definition_effects()
        || !limits
            .component_limits()
            .fits_within(resources.component_limits())
        || limits.maximum_canonical_bytes() > resources.maximum_canonical_bytes()
    {
        return Err(denial(
            WorthQueryApplicationWorkflowInstallationDenialKind::DefinitionLimitExceeded,
            definition.identity().as_str(),
        ));
    }
    let mut assessment_bindings = Vec::new();
    let mut condition_bindings = Vec::new();
    let mut approval_bindings = Vec::new();
    for node in definition.nodes() {
        let supported = match node.kind() {
            ApplicationWorkflowNodeKind::Operation {
                operation,
                requires_workflow_authority,
            } => {
                let matching = installed.operations.iter().filter(|candidate| {
                    candidate.marker == operation.operation_type()
                        && candidate.identifier == operation.identifier()
                        && &candidate.input_type == operation.input_type()
                        && candidate.requires_workflow_authority == *requires_workflow_authority
                        && match operation.binding() {
                            Some((identity, marker, posture)) => {
                                candidate.binding_type == marker
                                    && candidate.binding_identity == identity
                                    && candidate.requires_workflow_authority == posture
                            }
                            None => !*requires_workflow_authority,
                        }
                });
                matching.count() == 1
            }
            ApplicationWorkflowNodeKind::Assessment(assessment) => installed
                .assessments
                .iter()
                .find(|candidate| {
                    candidate.query_marker == assessment.query_type()
                        && candidate.query_identifier == assessment.identifier()
                        && &candidate.parameter_type == assessment.parameter_type()
                        && &candidate.result_type == assessment.result_type()
                })
                .map(|candidate| {
                    assessment_bindings.push((
                        node.identity().as_str().to_owned(),
                        candidate.binding_identity,
                    ));
                })
                .is_some(),
            ApplicationWorkflowNodeKind::Condition(condition) => installed
                .conditions
                .iter()
                .find(|candidate| {
                    candidate.query_marker == condition.query_type()
                        && candidate.query_identifier == condition.identifier()
                        && &candidate.parameter_type == condition.parameter_type()
                        && &candidate.result_type == condition.result_type()
                })
                .map(|candidate| {
                    condition_bindings.push((
                        node.identity().as_str().to_owned(),
                        candidate.binding_identity,
                    ));
                })
                .is_some(),
            ApplicationWorkflowNodeKind::Approval(approval) => installed
                .approvals
                .iter()
                .find(|candidate| {
                    candidate.binding.matches_capability(
                        approval.marker_type(),
                        approval.identifier(),
                        approval.capability_type(),
                    )
                })
                .map(|candidate| {
                    approval_bindings.push(WorthQueryInstalledWorkflowApprovalBinding {
                        node_path: node.identity().as_str().to_owned(),
                        operation: candidate.binding.operation_identifier,
                        installed_capability_identity: candidate.binding.installed_identity,
                    });
                })
                .is_some(),
            ApplicationWorkflowNodeKind::EvidenceJoin(_)
            | ApplicationWorkflowNodeKind::Terminal => true,
        };
        if !supported {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::UnsupportedDefinitionMember,
                node.identity().as_str(),
            ));
        }
    }
    assessment_bindings.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    condition_bindings.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    approval_bindings.sort_unstable_by(|left, right| left.node_path.cmp(&right.node_path));
    Ok(WorthQueryInstalledWorkflowDefinitionContract {
        schema_binding: installed.schema_binding.clone(),
        program_revision: installed.program_revision.clone(),
        definition,
        assessment_bindings: assessment_bindings.into_boxed_slice(),
        condition_bindings: condition_bindings.into_boxed_slice(),
        approval_bindings: approval_bindings.into_boxed_slice(),
        authoring_capability: installed.authoring_capability.clone(),
        marker: PhantomData,
    })
}
