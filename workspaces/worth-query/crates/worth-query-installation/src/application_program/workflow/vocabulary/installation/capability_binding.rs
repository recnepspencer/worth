use super::*;

impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
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
