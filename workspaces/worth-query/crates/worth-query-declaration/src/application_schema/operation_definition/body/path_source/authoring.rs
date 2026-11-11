//! Typed path endpoints and exact scalar bindings survive capture authoring.
use super::ApplicationOperationBodyPathFieldSource as Source;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        fragment::RecordSelection, ApplicationOperationBodyAuthoringDenial as Denial,
        ApplicationOperationBodyCaptureRole as Role, ApplicationOperationBodyCreatedField as Field,
        ApplicationOperationBodyCreationBuilder, OperationReads,
    },
};
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Capture at an exact native path, preserving optional absence.
    pub fn capture_from_path<Root, Selected, SourceField, Target>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, Selected>,
        source: SourceField,
        target: Target,
        role: Role,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        SourceField: RecordSelection<Entity = Selected>,
        Target:
            RecordSelection<Entity = E, Value = SourceField::Value, Binding = SourceField::Binding>,
    {
        let source_field = super::super::record_authoring::resolve(source)?;
        let target = super::super::record_authoring::resolve(target)?;
        let source = Source {
            path: path.into_meaning(),
            field: source_field.locus().clone(),
            presence: source_field.presence(),
        };
        if !source.has_supported_shape() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Field::CapturedPathField {
            field: target.locus().clone(),
            binding: target.recipe().binding_identity().clone(),
            scalar_family: target.recipe().scalar_family(),
            source,
            role,
            target_presence: target.presence(),
        });
        Ok(self)
    }
}
