use super::super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
use super::WorthQueryApplicationEffectProgramBuilder;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// Add the one Signal definition change that publishes with this
    /// application's Relational effects.
    pub fn advance_conditional_definition(
        &mut self,
        change: WorthQueryAdmittedApplicationConditionalDefinition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if self.conditional_definition.is_some() {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DuplicateConditionalDefinitionChange,
                self.read_set.admission.operation(),
            ));
        }
        if !change.admits_application(
            self.read_set.admission.runtime_authority(),
            self.read_set.admission.binding_identity(),
            self.read_set.lease.product().observation(),
        ) {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::ForeignConditionalDefinitionChange,
                self.read_set.admission.operation(),
            ));
        }
        self.conditional_definition = Some(change);
        Ok(())
    }
}
