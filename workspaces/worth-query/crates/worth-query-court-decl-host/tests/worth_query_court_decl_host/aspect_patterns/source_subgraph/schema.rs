use super::*;
worth_query_application!(pub Schema { owner: "source-subgraph", version: (1, 0), contributions: [Contribution], });
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "source-subgraph.contribution.v1", corrections: [correction::Restore],
    members: |schema| { generation::declare(edit::declare(declare(schema))) }
} }
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Source for Schema);
worth_query_entity!(pub Generated for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x919B0301), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x919B0302), revision = AspectContractRevision(1));
worth_query_aspect!(pub SourceFacts for Schema, Source; identity = AspectIdentity(0x919B0303), revision = AspectContractRevision(1));
worth_query_aspect!(pub GeneratedFacts for Schema, Generated; identity = AspectIdentity(0x919B0304), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub SourceKey for Schema, Source, SourceFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub SourceValue for Schema, Source, SourceFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub GeneratedKey for Schema, Generated, GeneratedFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GeneratedValue for Schema, Generated, GeneratedFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub SourceMember in Schema, Source => Source; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GeneratedMember in Schema, Generated => Generated; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Source::reference())
        .entity(Generated::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Source::reference(), SourceFacts::reference())
        .aspect(Generated::reference(), GeneratedFacts::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Source::reference(), SourceKey::reference())
        .field(Source::reference(), SourceValue::reference())
        .unique_field(Generated::reference(), GeneratedKey::reference())
        .field(Generated::reference(), GeneratedValue::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .principal_binding(Actor::reference())
        .relation(
            SourceMember::reference(),
            Source::reference(),
            Source::reference(),
        )
        .relation(
            GeneratedMember::reference(),
            Generated::reference(),
            Generated::reference(),
        );
    graph_read::source::declare(graph_read::generated::declare(schema))
}
