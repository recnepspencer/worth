//! Seal validated vocabulary and captured policies into one installed owner.
use super::*;
impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn finish(
        self,
    ) -> Result<
        WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        WorthQueryApplicationWorkflowInstallationDenial,
    > {
        if self.program.schema_binding() != &self.schema.binding_identity() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::ForeignSchemaBinding,
                Spec::IDENTITY.as_str(),
            ));
        }
        let actor_policy = Spec::actor_policy();
        let observation_policy = Spec::observation_policy();
        self.validate_policies(&actor_policy, &observation_policy)?;
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
        let support_identity = super::super::support_identity::derive(
            self.program.revision(),
            Spec::IDENTITY.as_str(),
            &actor_policy,
            &observation_policy,
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
            program_revision: *self.program.revision(),
            support_identity,
            actor_policy,
            observation_policy,
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
