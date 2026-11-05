//! A distinct typed profile gives child realizations actual Native memberships.
use super::*;
struct RealizationPeers;
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
    let schema = schema.relation(
        realization::output_next(),
        realization::Realization::reference(),
        realization::Realization::reference(),
    );
    schema
        .relation(
            peers(),
            realization::Realization::reference(),
            realization::Realization::reference(),
        )
        .components(ApplicationComponentSetDefinition::new(set).coupled_by(peers()))
}
