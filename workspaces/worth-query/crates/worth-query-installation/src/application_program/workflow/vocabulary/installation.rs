use std::any::TypeId;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRef},
    application_operation::ApplicationMutationBinding,
    application_program::ApplicationWorkflowSpec,
    application_query::{ApplicationQueryBinding, ApplicationQueryMarkerIdentity},
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};

use crate::application_program::WorthQueryInstalledApplicationProgram;
use crate::application_schema::WorthQueryInstalledApplicationSchema;

use super::{
    denial, InstalledWorkflowAdvanceCapability, InstalledWorkflowApproval,
    InstalledWorkflowAssessment, InstalledWorkflowAuthoringCapability,
    InstalledWorkflowCapabilityBinding, InstalledWorkflowCondition,
    InstalledWorkflowInstanceStartCapability, InstalledWorkflowOperation,
    WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryInstalledApplicationWorkflowSpec,
};

#[path = "installation/capability_binding.rs"]
mod capability_binding;

pub struct WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    schema: &'installed WorthQueryInstalledApplicationSchema<Schema>,
    program: &'installed WorthQueryInstalledApplicationProgram<Schema, Program>,
    authoring_capability: Option<InstalledWorkflowAuthoringCapability>,
    instance_start_capability: Option<InstalledWorkflowInstanceStartCapability>,
    advance_capability: Option<InstalledWorkflowAdvanceCapability>,
    operations: Vec<InstalledWorkflowOperation>,
    assessments: Vec<InstalledWorkflowAssessment>,
    conditions: Vec<InstalledWorkflowCondition>,
    approvals: Vec<InstalledWorkflowApproval>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
    markers: BTreeSet<(u8, TypeId)>,
    marker: PhantomData<fn() -> Spec>,
}

impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn begin(
        schema: &'installed WorthQueryInstalledApplicationSchema<Schema>,
        program: &'installed WorthQueryInstalledApplicationProgram<Schema, Program>,
        resources: WorthQueryApplicationWorkflowResourceCeiling,
    ) -> Self {
        Self {
            schema,
            program,
            authoring_capability: None,
            instance_start_capability: None,
            advance_capability: None,
            operations: Vec::new(),
            assessments: Vec::new(),
            conditions: Vec::new(),
            approvals: Vec::new(),
            resources,
            markers: BTreeSet::new(),
            marker: PhantomData,
        }
    }

    pub fn operation<Binding>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.insert_marker(0, TypeId::of::<Binding>(), Binding::IDENTITY)?;
        if !self
            .program
            .actions()
            .iter()
            .any(|action| action.mutation_binding_type() == Some(TypeId::of::<Binding>()))
        {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInProgram,
                Binding::IDENTITY,
            ));
        }
        self.schema
            .installed_mutation_binding::<Binding>()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInstalled,
                    Binding::IDENTITY,
                )
            })?;
        self.operations.push(InstalledWorkflowOperation {
            marker: TypeId::of::<Binding::Operation>(),
            identifier: Binding::Operation::IDENTIFIER,
            input_type: Binding::InputBinding::IDENTITY,
            binding_type: TypeId::of::<Binding>(),
            binding_identity: Binding::IDENTITY,
            requires_workflow_authority: Binding::REQUIRES_WORKFLOW_AUTHORITY,
        });
        Ok(self)
    }

    pub fn assessment<Binding>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Binding: ApplicationQueryBinding<Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
    {
        self.insert_marker(
            1,
            TypeId::of::<Binding::Query>(),
            Binding::Query::IDENTIFIER,
        )?;
        self.schema
            .installed_query_binding::<Binding>()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationWorkflowInstallationDenialKind::AssessmentNotInstalled,
                    Binding::IDENTITY,
                )
            })?;
        self.assessments.push(InstalledWorkflowAssessment {
            query_marker: TypeId::of::<Binding::Query>(),
            query_identifier: Binding::Query::IDENTIFIER,
            parameter_type: Binding::Query::PARAMETER_TYPE_IDENTITY,
            result_type: Binding::Query::RESULT_TYPE_IDENTITY,
            binding_identity: Binding::IDENTITY,
        });
        Ok(self)
    }

    pub fn condition<Binding>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Binding: ApplicationQueryBinding<Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
        <Binding::Query as ApplicationQueryMarkerIdentity<Schema>>::ResultBinding:
            ApplicationStructuredValueBinding<Value = bool>,
    {
        self.insert_marker(
            6,
            TypeId::of::<Binding::Query>(),
            Binding::Query::IDENTIFIER,
        )?;
        self.schema
            .installed_query_binding::<Binding>()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationWorkflowInstallationDenialKind::AssessmentNotInstalled,
                    Binding::IDENTITY,
                )
            })?;
        self.conditions.push(InstalledWorkflowCondition {
            query_marker: TypeId::of::<Binding::Query>(),
            query_identifier: Binding::Query::IDENTIFIER,
            parameter_type: Binding::Query::PARAMETER_TYPE_IDENTITY,
            result_type: Binding::Query::RESULT_TYPE_IDENTITY,
            binding_identity: Binding::IDENTITY,
        });
        Ok(self)
    }

    pub fn approval<Capability, Operation, Input>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        self.insert_marker(2, TypeId::of::<Capability>(), Capability::IDENTIFIER)?;
        let binding = self.bind_capability::<Capability, Operation, Input>(
            WorthQueryApplicationWorkflowInstallationDenialKind::ApprovalNotInstalled,
        )?;
        self.approvals.push(InstalledWorkflowApproval { binding });
        Ok(self)
    }

    pub fn authoring_capability<Capability, Operation, Input>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        if self.authoring_capability.is_some() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        if self
            .instance_start_capability
            .as_ref()
            .is_some_and(|installed| {
                installed.binding.matches_capability(
                    TypeId::of::<Capability>(),
                    Capability::IDENTIFIER,
                    &Capability::PORTABLE_TYPE_IDENTITY,
                )
            })
        {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        if self.advance_capability.as_ref().is_some_and(|installed| {
            installed.binding.matches_capability(
                TypeId::of::<Capability>(),
                Capability::IDENTIFIER,
                &Capability::PORTABLE_TYPE_IDENTITY,
            )
        }) {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        self.insert_marker(3, TypeId::of::<Capability>(), Capability::IDENTIFIER)?;
        let binding = self.bind_capability::<Capability, Operation, Input>(
            WorthQueryApplicationWorkflowInstallationDenialKind::AuthoringCapabilityNotInstalled,
        )?;
        self.authoring_capability = Some(InstalledWorkflowAuthoringCapability { binding });
        Ok(self)
    }

    pub fn instance_start_capability<Capability, Operation, Input>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        if self.instance_start_capability.is_some() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        if self.authoring_capability.as_ref().is_some_and(|installed| {
            installed.binding.matches_capability(
                TypeId::of::<Capability>(),
                Capability::IDENTIFIER,
                &Capability::PORTABLE_TYPE_IDENTITY,
            )
        }) {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        if self.advance_capability.as_ref().is_some_and(|installed| {
            installed.binding.matches_capability(
                TypeId::of::<Capability>(),
                Capability::IDENTIFIER,
                &Capability::PORTABLE_TYPE_IDENTITY,
            )
        }) {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        self.insert_marker(4, TypeId::of::<Capability>(), Capability::IDENTIFIER)?;
        let binding = self.bind_capability::<Capability, Operation, Input>(
            WorthQueryApplicationWorkflowInstallationDenialKind::InstanceStartCapabilityNotInstalled,
        )?;
        self.instance_start_capability = Some(InstalledWorkflowInstanceStartCapability { binding });
        Ok(self)
    }

    pub fn advance_capability<Capability, Operation, Input>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        if self.advance_capability.is_some() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                Capability::IDENTIFIER,
            ));
        }
        for installed in [
            self.authoring_capability
                .as_ref()
                .map(|value| &value.binding),
            self.instance_start_capability
                .as_ref()
                .map(|value| &value.binding),
        ]
        .into_iter()
        .flatten()
        {
            if installed.matches_capability(
                TypeId::of::<Capability>(),
                Capability::IDENTIFIER,
                &Capability::PORTABLE_TYPE_IDENTITY,
            ) {
                return Err(denial(
                    WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                    Capability::IDENTIFIER,
                ));
            }
        }
        self.insert_marker(5, TypeId::of::<Capability>(), Capability::IDENTIFIER)?;
        let binding = self.bind_capability::<Capability, Operation, Input>(
            WorthQueryApplicationWorkflowInstallationDenialKind::AdvanceCapabilityNotInstalled,
        )?;
        self.advance_capability = Some(InstalledWorkflowAdvanceCapability { binding });
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<
        WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        WorthQueryApplicationWorkflowInstallationDenial,
    > {
        if self.program.schema_binding() != &self.schema.binding_identity() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::ForeignSchemaBinding,
                Spec::IDENTITY.as_str(),
            ));
        }
        let authoring_capability = self.authoring_capability.ok_or_else(|| {
            denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::MissingAuthoringCapability,
                Spec::IDENTITY.as_str(),
            )
        })?;
        let instance_start_capability = self.instance_start_capability.ok_or_else(|| {
            denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::MissingInstanceStartCapability,
                Spec::IDENTITY.as_str(),
            )
        })?;
        let advance_capability = self.advance_capability.ok_or_else(|| {
            denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::MissingAdvanceCapability,
                Spec::IDENTITY.as_str(),
            )
        })?;
        if self.operations.is_empty() || self.assessments.is_empty() || self.approvals.is_empty() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::EmptyVocabulary,
                Spec::IDENTITY.as_str(),
            ));
        }
        let support_identity = super::support_identity::derive(
            self.program.revision(),
            Spec::IDENTITY.as_str(),
            &authoring_capability,
            &instance_start_capability,
            &advance_capability,
            &self.operations,
            &self.assessments,
            &self.conditions,
            &self.approvals,
            self.resources,
        );
        Ok(WorthQueryInstalledApplicationWorkflowSpec {
            schema_binding: self.schema.binding_identity(),
            program_revision: self.program.revision().clone(),
            support_identity,
            authoring_capability,
            instance_start_capability,
            advance_capability,
            operations: self.operations.into_boxed_slice(),
            assessments: self.assessments.into_boxed_slice(),
            conditions: self.conditions.into_boxed_slice(),
            approvals: self.approvals.into_boxed_slice(),
            resources: self.resources,
            marker: PhantomData,
        })
    }
}
