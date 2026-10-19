//! Bind authored context consumption to the actual installed source expectation.
use super::{
    denial, ApplicationMutationBindingDescriptor, ApplicationSchema, ApplicationSchemaMember,
    DenialKind, WorthQueryApplicationOperationInstallationDenial,
    WorthQueryInstalledApplicationSchema,
};

pub(super) fn validate<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
    for member in schema.installed_declaration().members() {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member
            && operation == descriptor.operation_name()
        {
            for (mutation_binding, query) in body.query_contexts() {
                if mutation_binding != descriptor.identity()
                    || descriptor.source_query_identifier() != Some(query.name())
                {
                    return Err(denial(
                        DenialKind::MutationBindingMeaningChanged,
                        descriptor.identity(),
                    ));
                }
            }
        }
    }
    let Some(identifier) = descriptor.source_query_identifier() else {
        return Ok(());
    };
    if schema
        .installed_query_binding_inventory()
        .any(|query| query.query_name() == identifier)
    {
        Ok(())
    } else {
        Err(denial(
            DenialKind::MutationSourceQueryNotInstalled,
            descriptor.identity(),
        ))
    }
}
