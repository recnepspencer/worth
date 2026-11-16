//! A live owner's member edit can require a genuinely new coupled source.
use super::*;
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_schema::fragment::*;
pub(super) mod correction;
pub(super) mod operation;
pub(super) mod program;
pub(super) mod records;
use operation::*;
use records::*;
#[cfg(test)]
pub(super) mod courts;

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema.application_mutation_binding::<Binding>()
}
pub(super) fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<Binding, _>(Handler)
}
