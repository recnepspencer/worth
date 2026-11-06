//! A distinct typed profile gives child realizations actual Native memberships.
use super::*;
pub struct RealizationPeers;
impl worth_query_decl::facade::application_schema::fragment::Relation for RealizationPeers {
    const NAME: &'static str = "RealizationPeers";
    type To = realization::Realization;
}
struct RealizationComponents;
fn peers() -> ApplicationRelationRef<
    Schema,
    RealizationPeers,
    realization::Realization,
    realization::Realization,
> {
    ApplicationRelationRef::from_schema_identifiers(
        "RealizationPeers",
        "Realization",
        "Realization",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let set =
        ApplicationComponentSetRef::<Schema, RealizationComponents, realization::Realization>::new(
            "RealizationComponents",
            realization::Realization::reference(),
        );
    schema.components(ApplicationComponentSetDefinition::new(set).coupled_by(peers()))
}
