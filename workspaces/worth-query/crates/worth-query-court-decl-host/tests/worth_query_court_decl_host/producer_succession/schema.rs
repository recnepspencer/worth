use super::*;
use worth_query_decl::facade::authentication::*;
worth_query_application!(pub Schema { owner: "worth.query.generic-dependent-dispatch", version: (1, 0), contributions: [Contribution], });
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Node for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x9176A101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x9176A102), revision = AspectContractRevision(1));
worth_query_aspect!(pub Facts for Schema, Node; identity = AspectIdentity(0x9176A103), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentity for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Key for Schema, Node, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ParentName for Schema, Node, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Value for Schema, Node, Facts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Width for Schema, Node, Facts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_aspect!(pub OutputFacts for Schema, Node; identity = AspectIdentity(0x9176A104), revision = AspectContractRevision(1));
worth_query_field!(pub Produced for Schema, Node, OutputFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub GeneratedBy in Schema, Node => Node; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub PrincipalBinding in Schema, mapping Mapping { identity: ExternalIdentity, status: MappingStatus, target: MappingTarget => Principal, principal_identity: PrincipalIdentity });
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Node::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Node::reference(), Facts::reference())
        .optional_aspect(Node::reference(), OutputFacts::reference())
        .field(Mapping::reference(), ExternalIdentity::reference())
        .field(Mapping::reference(), MappingStatus::reference())
        .field(Principal::reference(), PrincipalIdentity::reference())
        .unique_field(Node::reference(), Key::reference())
        .field(Node::reference(), ParentName::reference())
        .field(Node::reference(), Value::reference())
        .field(Node::reference(), Width::reference())
        .field(Node::reference(), Produced::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .principal_binding(PrincipalBinding::reference())
        .relation(
            GeneratedBy::reference(),
            Node::reference(),
            Node::reference(),
        )
        .application_query(route_query::definition())
        .application_query_binding::<route_query::Binding>()
        .application_query(query::definition())
        .application_query_binding::<ReadBinding>();
    let schema = realization::declare(schema);
    if super::PRIVATE_AGGREGATES {
        aggregate_schema::declare(schema)
    } else {
        schema
    }
}
