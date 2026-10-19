use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRef},
    application_program::{
        ApplicationExpressionOperandValue, ApplicationWorkflowApprovalRef, ApplicationWorkflowSpec,
    },
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
    InstalledWorkflowAssessment, InstalledWorkflowAuthoringCapability, InstalledWorkflowBirth,
    InstalledWorkflowCapabilityBinding, InstalledWorkflowCondition,
    InstalledWorkflowInstanceStartCapability, InstalledWorkflowOperation,
    WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryInstalledApplicationWorkflowSpec,
};

mod birth;
#[path = "installation/capability_binding.rs"]
mod capability_binding;
mod finish;
mod operation;
mod policies;

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
    births: Vec<InstalledWorkflowBirth>,
    assessments: Vec<InstalledWorkflowAssessment>,
    conditions: Vec<InstalledWorkflowCondition>,
    approvals: Vec<InstalledWorkflowApproval>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
    markers: HashSet<(u8, TypeId)>,
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
            births: Vec::new(),
            assessments: Vec::new(),
            conditions: Vec::new(),
            approvals: Vec::new(),
            resources,
            markers: HashSet::new(),
            marker: PhantomData,
        }
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
        self.assessments
            .push(InstalledWorkflowAssessment::declared::<Spec, Binding>());
        Ok(self)
    }

    /// Installs a query whose result condition expressions may read as an
    /// operand.
    pub fn condition_operand<Binding>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Binding: ApplicationQueryBinding<Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
        <<Binding::Query as ApplicationQueryMarkerIdentity<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            ApplicationExpressionOperandValue,
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
        self.conditions
            .push(InstalledWorkflowCondition::declared::<Spec, Binding>());
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
        self.approvals.push(InstalledWorkflowApproval {
            binding,
            reference: ApplicationWorkflowApprovalRef::declared::<Spec, Capability>(),
        });
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
}
