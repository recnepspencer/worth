use std::any::TypeId;
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramRevision, ApplicationWorkflowSpec, ValidatedWorkflowDefinition,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
    portable_identity::WorthQueryPortableTypeIdentity,
};

mod adoption_coverage;
mod approval_binding;
mod installation;
mod support_identity;
pub use adoption_coverage::{
    WorthQueryWorkflowNodeDependency, WorthQueryWorkflowVocabularyCoverage,
};
pub use installation::WorthQueryApplicationWorkflowSpecInstallation;

mod resource_ceiling;
pub use resource_ceiling::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryWorkflowHistoryReconstructionBudget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationWorkflowInstallationDenialKind {
    OperationNotInProgram,
    OperationNotInstalled,
    AssessmentNotInstalled,
    ApprovalNotInstalled,
    AuthoringCapabilityNotInstalled,
    InstanceStartCapabilityNotInstalled,
    AdvanceCapabilityNotInstalled,
    DuplicateVocabularyMember,
    MissingAuthoringCapability,
    MissingInstanceStartCapability,
    MissingAdvanceCapability,
    EmptyVocabulary,
    DefinitionLimitExceeded,
    UnsupportedDefinitionMember,
    ForeignSchemaBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationWorkflowInstallationDenial {
    kind: WorthQueryApplicationWorkflowInstallationDenialKind,
    subject: String,
}

impl WorthQueryApplicationWorkflowInstallationDenial {
    pub const fn kind(&self) -> WorthQueryApplicationWorkflowInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationWorkflowInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application workflow installation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationWorkflowInstallationDenial {}

#[derive(Clone)]
pub(super) struct InstalledWorkflowOperation {
    pub(super) marker: TypeId,
    pub(super) identifier: &'static str,
    pub(super) input_type: WorthQueryPortableTypeIdentity,
    pub(super) binding_type: TypeId,
    pub(super) binding_identity: &'static str,
    pub(super) requires_workflow_authority: bool,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowAssessment {
    pub(super) query_marker: TypeId,
    pub(super) query_identifier: &'static str,
    pub(super) parameter_type: WorthQueryPortableTypeIdentity,
    pub(super) result_type: WorthQueryPortableTypeIdentity,
    pub(super) binding_identity: &'static str,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowCondition {
    pub(super) query_marker: TypeId,
    pub(super) query_identifier: &'static str,
    pub(super) parameter_type: WorthQueryPortableTypeIdentity,
    pub(super) result_type: WorthQueryPortableTypeIdentity,
    pub(super) binding_identity: &'static str,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowCapabilityBinding {
    pub(super) marker: TypeId,
    pub(super) identifier: &'static str,
    pub(super) capability_type: WorthQueryPortableTypeIdentity,
    pub(super) operation_marker: TypeId,
    pub(super) operation_identifier: &'static str,
    pub(super) installed_identity: [u8; 32],
}

impl InstalledWorkflowCapabilityBinding {
    pub(super) fn matches_capability(
        &self,
        marker: TypeId,
        identifier: &str,
        capability_type: &WorthQueryPortableTypeIdentity,
    ) -> bool {
        self.marker == marker
            && self.identifier == identifier
            && &self.capability_type == capability_type
    }

    pub(super) fn matches_binding(
        &self,
        capability_marker: TypeId,
        capability_identifier: &str,
        capability_type: &WorthQueryPortableTypeIdentity,
        operation_marker: TypeId,
        operation_identifier: &str,
    ) -> bool {
        self.matches_capability(capability_marker, capability_identifier, capability_type)
            && self.operation_marker == operation_marker
            && self.operation_identifier == operation_identifier
    }
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowAuthoringCapability {
    pub(super) binding: InstalledWorkflowCapabilityBinding,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowInstanceStartCapability {
    pub(super) binding: InstalledWorkflowCapabilityBinding,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowAdvanceCapability {
    pub(super) binding: InstalledWorkflowCapabilityBinding,
}

#[derive(Clone)]
pub(super) struct InstalledWorkflowApproval {
    pub(super) binding: InstalledWorkflowCapabilityBinding,
}

pub struct WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) schema_binding: ApplicationSchemaBindingIdentity,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) support_identity: [u8; 32],
    pub(super) authoring_capability: InstalledWorkflowAuthoringCapability,
    pub(super) instance_start_capability: InstalledWorkflowInstanceStartCapability,
    pub(super) advance_capability: InstalledWorkflowAdvanceCapability,
    pub(super) operations: Box<[InstalledWorkflowOperation]>,
    pub(super) assessments: Box<[InstalledWorkflowAssessment]>,
    pub(super) conditions: Box<[InstalledWorkflowCondition]>,
    pub(super) approvals: Box<[InstalledWorkflowApproval]>,
    pub(super) resources: WorthQueryApplicationWorkflowResourceCeiling,
    pub(super) marker: PhantomData<fn() -> (Schema, Spec, Program)>,
}

impl<Schema, Spec, Program> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
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

    pub const fn resources(&self) -> WorthQueryApplicationWorkflowResourceCeiling {
        self.resources
    }

    pub fn authoring_capability(&self) -> (&str, &WorthQueryPortableTypeIdentity) {
        (
            self.authoring_capability.binding.identifier,
            &self.authoring_capability.binding.capability_type,
        )
    }

    pub fn instance_start_capability(&self) -> (&str, &WorthQueryPortableTypeIdentity) {
        (
            self.instance_start_capability.binding.identifier,
            &self.instance_start_capability.binding.capability_type,
        )
    }

    pub fn advance_capability(&self) -> (&str, &WorthQueryPortableTypeIdentity) {
        (
            self.advance_capability.binding.identifier,
            &self.advance_capability.binding.capability_type,
        )
    }

    #[doc(hidden)]
    pub fn advance_binding_matches<Capability, Operation>(&self) -> bool
    where
        Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<
                Schema = Schema,
            > + 'static,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    {
        self.advance_capability.binding.matches_binding(
            TypeId::of::<Capability>(),
            Capability::IDENTIFIER,
            &Capability::PORTABLE_TYPE_IDENTITY,
            TypeId::of::<Operation>(),
            Operation::IDENTIFIER,
        )
    }

    #[doc(hidden)]
    pub fn advance_capability_identity_bytes(&self) -> &[u8; 32] {
        &self.advance_capability.binding.installed_identity
    }

    #[doc(hidden)]
    pub fn operation_binding_matches<Operation>(&self) -> bool
    where
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    {
        let marker = TypeId::of::<Operation>();
        let input_type = <Operation::InputBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::IDENTITY;
        self.operations.iter().any(|operation| {
            operation.marker == marker
                && operation.identifier == Operation::IDENTIFIER
                && operation.input_type == input_type
        })
    }

    #[doc(hidden)]
    pub fn instance_start_binding_matches<Capability, Operation>(&self) -> bool
    where
        Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<
                Schema = Schema,
            > + 'static,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
    {
        self.instance_start_capability.binding.matches_binding(
            TypeId::of::<Capability>(),
            Capability::IDENTIFIER,
            &Capability::PORTABLE_TYPE_IDENTITY,
            TypeId::of::<Operation>(),
            Operation::IDENTIFIER,
        )
    }

    #[doc(hidden)]
    pub fn instance_start_capability_identity_bytes(&self) -> &[u8; 32] {
        &self.instance_start_capability.binding.installed_identity
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
            TypeId::of::<Capability>(),
            Capability::IDENTIFIER,
            &Capability::PORTABLE_TYPE_IDENTITY,
            TypeId::of::<Operation>(),
            Operation::IDENTIFIER,
        )
    }

    #[doc(hidden)]
    pub fn authoring_capability_identity_bytes(&self) -> &[u8; 32] {
        &self.authoring_capability.binding.installed_identity
    }

    pub fn bind_definition(
        &self,
        definition: ValidatedWorkflowDefinition<Spec>,
    ) -> Result<
        super::WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
        WorthQueryApplicationWorkflowInstallationDenial,
    > {
        super::definition_contract::bind(self, definition)
    }
}

pub(super) fn denial(
    kind: WorthQueryApplicationWorkflowInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationWorkflowInstallationDenial {
    WorthQueryApplicationWorkflowInstallationDenial {
        kind,
        subject: subject.into(),
    }
}

#[cfg(test)]
#[path = "vocabulary/tests.rs"]
mod tests;
