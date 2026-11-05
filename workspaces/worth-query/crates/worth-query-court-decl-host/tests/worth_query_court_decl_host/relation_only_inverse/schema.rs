//! Nodes keyed by an authored id, held by an ordinary relation.
use super::*;
worth_query_application!(pub Schema { owner: "relation-only-inverse", version: (1, 0), contributions: [Contribution], });
worth_query_application_contribution! {
    pub contribution Contribution in Schema {
        identity: "relation-only-inverse.contribution.v1",
        corrections: [correction::UndoSpawn],
        members: |schema| { operations::declare(declare(schema)).application_query(query::definition()).application_query_binding::<query::Binding>() }
    }
}
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Node for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x919B6801), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x919B6802), revision = AspectContractRevision(1));
worth_query_aspect!(pub Facts for Schema, Node; identity = AspectIdentity(0x919B6803), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Id for Schema, Node, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Holds in Schema, Node => Node; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Unrelated in Schema, Node => Node; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Node::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Node::reference(), Facts::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Node::reference(), Id::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .relation(Holds::reference(), Node::reference(), Node::reference())
        .relation(Unrelated::reference(), Node::reference(), Node::reference())
        .principal_binding(Actor::reference())
}
