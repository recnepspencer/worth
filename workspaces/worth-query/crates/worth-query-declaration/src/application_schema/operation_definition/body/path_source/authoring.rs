//! Typed path endpoints and exact scalar bindings survive capture authoring.
use super::ApplicationOperationBodyPathFieldSource as Source;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
        ApplicationOperationBodyAuthoringDenial as Denial,
        ApplicationOperationBodyCaptureRole as Role, ApplicationOperationBodyCreatedField as Field,
        ApplicationOperationBodyCreationBuilder, ApplicationScalarValueBinding,
        DeclaredApplicationFieldValue, OperationReads, OperationWrites,
    },
};
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Capture at an exact native path, including the identity path for scope
    /// facts. Optional targets preserve absence; required targets require a value.
    pub fn capture_from_path<Root, Selected, SA, SF, SW, SQ, TA, TF, TW, TQ, V, U>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, Selected>,
        source: ApplicationFieldRef<S, Selected, SA, SF, V, SW, SQ, U>,
        target: ApplicationFieldRef<S, E, TA, TF, V, TW, TQ, U>,
        role: Role,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        SF: OperationReads<O> + DeclaredApplicationFieldValue<Value = V>,
        TF: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
    {
        if SF::Binding::IDENTITY != TF::Binding::IDENTITY
            || SF::Binding::SCALAR_FAMILY != TF::Binding::SCALAR_FAMILY
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        let source = Source {
            path: path.into_meaning(),
            field: ApplicationFieldBindingLocus::new(
                source.entity(),
                source.aspect(),
                source.field(),
            ),
            presence: SF::PRESENCE,
        };
        if !source.has_supported_shape() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Field::CapturedPathField {
            field: ApplicationFieldBindingLocus::new(
                target.entity(),
                target.aspect(),
                target.field(),
            ),
            binding: TF::Binding::IDENTITY,
            scalar_family: TF::Binding::SCALAR_FAMILY,
            source,
            role,
            target_presence: TF::PRESENCE,
        });
        Ok(self)
    }
}
