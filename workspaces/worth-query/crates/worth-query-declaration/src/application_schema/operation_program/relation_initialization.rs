use super::*;
use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationFieldMarkerIdentity, ApplicationRelationFieldRef,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Authorizes one field only while this operation creates its declared relation.
    pub fn operation_initialize_relation_field<
        Operation,
        Input,
        Relation,
        From,
        To,
        Aspect,
        Field,
        Write,
        Unit,
    >(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>,
    ) -> Self
    where
        Relation: OperationLinks<Operation>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
        Field:
            ApplicationFieldMarkerIdentity<Schema, Relation, Aspect> + OperationWrites<Operation>,
        Unit: ApplicationFieldUnit,
    {
        let relation = field.relation();
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::InitializeRelationField {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            },
        )
    }
}
