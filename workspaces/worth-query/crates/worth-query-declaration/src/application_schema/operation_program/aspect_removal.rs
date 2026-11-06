//! Explicit whole-parent removal, never inferred from child writes.
use super::super::{
    fragment::{Aspect, AspectSelection, Entity, Fragment},
    ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchemaDeclarationBuilder, OperationRemovesAspects,
};
impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_remove_aspect<Operation, Input, F, E, A>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        aspect: AspectSelection<F, E, A>,
    ) -> Self
    where
        F: Fragment,
        E: Entity,
        A: Aspect + OperationRemovesAspects<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::RemoveAspect {
                entity: aspect.entity().to_owned(),
                aspect: aspect.name().to_owned(),
            },
        )
    }
}
