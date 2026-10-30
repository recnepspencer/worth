//! Explicit whole-parent removal, never inferred from child writes.
use super::super::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityMarkerIdentity,
    ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchemaDeclarationBuilder, OperationRemovesAspects,
};
impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_remove_aspect<Operation, Input, Entity, Aspect>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Self
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
        Aspect:
            ApplicationAspectMarkerIdentity<Schema, Entity> + OperationRemovesAspects<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::RemoveAspect {
                entity: Entity::IDENTIFIER.to_owned(),
                aspect: aspect.name().to_owned(),
            },
        )
    }
}
