//! An optional source must be present; no missing-value default is copied.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{fragment::RecordSelection, ApplicationFieldPresence};
impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    pub fn initialize_from_present_scope<Source, Field>(
        mut self,
        source: Source,
        target: Field,
    ) -> Result<Self, Denial>
    where
        Source: RecordSelection,
        Field: RecordSelection<Entity = Entity, Value = Source::Value, Binding = Source::Binding>,
    {
        let source = super::record_authoring::resolve(source)?;
        let target = super::record_authoring::resolve(target)?;
        if source.presence() != ApplicationFieldPresence::Optional
            || target.presence() != ApplicationFieldPresence::Required
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Step::PresentScopeField {
            field: target.locus().clone(),
            binding: target.recipe().binding_identity().clone(),
            scalar_family: target.recipe().scalar_family(),
            source: source.locus().clone(),
        });
        Ok(self)
    }
}
