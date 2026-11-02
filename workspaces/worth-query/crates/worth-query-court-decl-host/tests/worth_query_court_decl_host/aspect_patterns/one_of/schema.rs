use super::*;
worth_query_application!(pub Schema { owner: "native-record-patterns", version: (1, 0), contributions: [Contribution], });
worth_query_application_contribution! {
    pub contribution Contribution in Schema {
        identity: "native-record-patterns.contribution.v1",
        corrections: [correction::Restore],
        members: |schema| { restore_first::declare(retirement::declare(authoring::declare(declare(schema)))).invariant(rule::definition()).application_query(query::definition()).application_query_binding::<query::Binding>().application_query_inspection(ApplicationQueryInspectionDefinition::<Schema, query::Binding>::new(ApplicationQueryDisclosureContract::public()).unwrap()) }
    }
}
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Record for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x919B0101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x919B0102), revision = AspectContractRevision(1));
worth_query_aspect!(pub Identity for Schema, Record; identity = AspectIdentity(0x919B0103), revision = AspectContractRevision(1));
worth_query_aspect!(pub First for Schema, Record; identity = AspectIdentity(0x919B0104), revision = AspectContractRevision(1));
worth_query_aspect!(pub Second for Schema, Record; identity = AspectIdentity(0x919B0105), revision = AspectContractRevision(1));
worth_query_aspect!(pub Third for Schema, Record; identity = AspectIdentity(0x919B0106), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Key for Schema, Record, Identity: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Class for Schema, Record, Identity: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub FirstValue for Schema, Record, First: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub SecondValue for Schema, Record, Second: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub SecondRequired for Schema, Record, Second: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub ThirdValue for Schema, Record, Third: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Record::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Record::reference(), Identity::reference())
        .optional_aspect(Record::reference(), First::reference())
        .optional_aspect(Record::reference(), Second::reference())
        .optional_aspect(Record::reference(), Third::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Record::reference(), Key::reference())
        .field(Record::reference(), Class::reference())
        .field(Record::reference(), FirstValue::reference())
        .field(Record::reference(), SecondValue::reference())
        .field(Record::reference(), SecondRequired::reference())
        .field(Record::reference(), ThirdValue::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .principal_binding(Actor::reference())
}
