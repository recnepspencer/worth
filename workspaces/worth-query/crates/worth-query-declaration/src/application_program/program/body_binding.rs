//! Bind action meaning to completed schema bodies before revision minting.
use super::{
    ApplicationActionDeclaration, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationSchema,
};
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationSchemaDeclaration, ApplicationSchemaMember,
};

impl<Schema, Program> ApplicationProgramAuthoring<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Captures executable bodies from the same completed schema the program
    /// will install against. No callback re-runs schema or body authoring.
    /// Installation denies a body-backed action left unbound or bound differently.
    pub fn with_schema(mut self, schema: &ApplicationSchemaDeclaration<Schema>) -> Self {
        self.operation_bodies = schema
            .erased()
            .members()
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::OperationBody { operation, body } => {
                    Some((operation.clone(), body.clone()))
                }
                _ => None,
            })
            .collect();
        self
    }
}

pub(super) fn bind(
    actions: &mut [ApplicationActionDeclaration],
    bodies: &[(String, ApplicationOperationBodyProgram)],
) {
    for action in actions {
        if let Some((_, body)) = bodies
            .iter()
            .find(|(operation, _)| operation == action.operation_identifier())
        {
            action.bind_operation_body(body.clone());
        }
    }
}
