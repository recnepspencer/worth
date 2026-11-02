worth_query_application!(pub Schema { owner: "class-read-cost", version: (1, 0), contributions: [Contribution], });
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "class-read-cost.contribution.v1", members: |schema| { declare(schema) }
} }
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Record for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x919B0201), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x919B0202), revision = AspectContractRevision(1));
worth_query_aspect!(pub Identity for Schema, Record; identity = AspectIdentity(0x919B0203), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Key for Schema, Record, Identity: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Sink for Schema, Record, Identity: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
macro_rules! class {
    ($aspect:ident, $field:ident, $identity:expr) => {
        worth_query_aspect!(pub $aspect for Schema, Record; identity = AspectIdentity($identity), revision = AspectContractRevision(1));
        worth_query_field!(pub $field for Schema, Record, $aspect: u64 => U64ApplicationValueBinding, read_write, equality);
    };
}
class!(Class1, Value1, 0x919B0211);
class!(Class2, Value2, 0x919B0212);
class!(Class3, Value3, 0x919B0213);
class!(Class4, Value4, 0x919B0214);
class!(Class5, Value5, 0x919B0215);
class!(Class6, Value6, 0x919B0216);
class!(Class7, Value7, 0x919B0217);
class!(Class8, Value8, 0x919B0218);
class!(Class9, Value9, 0x919B0219);
class!(Class10, Value10, 0x919B021A);
class!(Class11, Value11, 0x919B021B);
class!(Class12, Value12, 0x919B021C);
fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let mut schema = schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Record::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Record::reference(), Identity::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Record::reference(), Key::reference())
        .field(Record::reference(), Sink::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .principal_binding(Actor::reference());
    macro_rules! install {
        ($n:literal, $aspect:ident, $field:ident) => {
            if CLASSES >= $n {
                schema = schema
                    .optional_aspect(Record::reference(), $aspect::reference())
                    .field(Record::reference(), $field::reference());
            }
        };
    }
    install!(1, Class1, Value1);
    install!(2, Class2, Value2);
    install!(3, Class3, Value3);
    install!(4, Class4, Value4);
    install!(5, Class5, Value5);
    install!(6, Class6, Value6);
    install!(7, Class7, Value7);
    install!(8, Class8, Value8);
    install!(9, Class9, Value9);
    install!(10, Class10, Value10);
    install!(11, Class11, Value11);
    install!(12, Class12, Value12);
    schema
        .declared_operation::<CopySelected>(
            CopySelected::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
        .application_query(readback::definition())
        .application_query_binding::<readback::ReadBinding>()
}
