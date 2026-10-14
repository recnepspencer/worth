//! Bind program adoption meaning to the existing installed capability owner.
use crate::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportDenial,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef, application_program::*,
    application_schema::ApplicationOperationRef,
};

pub(super) fn validate<Schema: ApplicationSchema, Program: ApplicationProgramDefinition<Schema>>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryProgramSupportDenial> {
    Program::adoption_authorization(InstallationVisitor {
        schema,
        program: Program::IDENTITY.clone(),
    })
}
struct InstallationVisitor<'a, Schema> {
    schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    program: ApplicationProgramIdentity,
}
impl<Schema: ApplicationSchema> ApplicationProgramAdoptionAuthorizationVisitor<Schema>
    for InstallationVisitor<'_, Schema>
{
    type Output = Result<(), WorthQueryProgramSupportDenial>;
    fn unavailable(self) -> Self::Output {
        Ok(())
    }
    fn capability<Binding: ApplicationProgramAdoptionBinding<Schema>>(self) -> Self::Output {
        self.schema
            .principal_binding(Binding::principal_binding())
            .map_err(
                |denial| WorthQueryProgramSupportDenial::UnsupportedAdoptionAuthorization {
                    program: self.program.clone(),
                    subject: format!("{denial:?}"),
                },
            )?;
        self.schema.capability(ApplicationCapabilityRef::<Schema, Binding::Capability>::from_declaration(), ApplicationOperationRef::<Schema, Binding::Operation, Binding::Input>::from_declaration()).map_err(|denial| WorthQueryProgramSupportDenial::UnsupportedAdoptionAuthorization { program: self.program, subject: format!("{denial:?}") })?;
        Ok(())
    }
}
