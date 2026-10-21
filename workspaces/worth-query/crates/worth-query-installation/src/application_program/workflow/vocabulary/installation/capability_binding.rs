use super::*;

pub(super) enum ControlBodyProfile {
    None,
    SignedApproval,
}

impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) fn bind_control_capability<Capability, Operation, Input>(
        &self,
        denial_kind: WorthQueryApplicationWorkflowInstallationDenialKind,
        profile: ControlBodyProfile,
    ) -> Result<InstalledWorkflowCapabilityBinding, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        let binding = self.bind_capability::<Capability, Operation, Input>(denial_kind)?;
        let operation = self
            .schema
            .installed_operation(
                ApplicationOperationRef::<Schema, Operation, Input>::from_declaration(),
            )
            .map_err(|_| denial(denial_kind, Operation::IDENTIFIER))?;
        if let Some(body) = operation.body() {
            let supported = match profile {
                ControlBodyProfile::None => false,
                ControlBodyProfile::SignedApproval => {
                    self.schema
                        .installed_mutation_binding_inventory()
                        .any(|binding| {
                            binding.operation_name() == Operation::IDENTIFIER
                                && binding.workflow_control()
                        })
                        && body.succession().is_none()
                        && body.creation().is_some_and(|creation| {
                            creation.signing().is_some() && creation.enrollment().is_none()
                        })
                        && !operation.contracts().external_effect().is_declared()
                }
            };
            if !supported {
                return Err(denial(denial_kind, Operation::IDENTIFIER));
            }
        }
        Ok(binding)
    }

    pub(super) fn bind_capability<Capability, Operation, Input>(
        &self,
        denial_kind: WorthQueryApplicationWorkflowInstallationDenialKind,
    ) -> Result<InstalledWorkflowCapabilityBinding, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: 'static,
    {
        if !self.program.contains_operation_type::<Operation>() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInProgram,
                Operation::IDENTIFIER,
            ));
        }
        let installed = self
            .schema
            .capability(
                ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
                ApplicationOperationRef::<Schema, Operation, Input>::from_declaration(),
            )
            .map_err(|_| denial(denial_kind, Capability::IDENTIFIER))?;
        Ok(InstalledWorkflowCapabilityBinding {
            marker: TypeId::of::<Capability>(),
            identifier: Capability::IDENTIFIER,
            capability_type: Capability::PORTABLE_TYPE_IDENTITY,
            operation_marker: TypeId::of::<Operation>(),
            operation_identifier: Operation::IDENTIFIER,
            installed_identity: *installed.identity().bytes(),
        })
    }

    pub(super) fn insert_marker(
        &mut self,
        family: u8,
        marker: TypeId,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationWorkflowInstallationDenial> {
        if !self.markers.insert((family, marker)) {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::DuplicateVocabularyMember,
                subject,
            ));
        }
        Ok(())
    }
}
