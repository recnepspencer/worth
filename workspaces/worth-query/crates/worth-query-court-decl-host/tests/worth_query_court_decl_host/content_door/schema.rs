use super::*;
worth_query_application!(pub Schema { owner: "content-door", version: (1,0), contributions: [Contribution], });
worth_query_application_contribution! {
    pub contribution Contribution in Schema {
        identity: "content-door.contribution.v1", corrections: [revision::Restore, created_content::Capture],
        members: |schema| { source::declare(created_content::declare(revision::declare(operation::declare(declare(schema))))) }
    }
}
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Record for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x919C101), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x919C102), revision = AspectContractRevision(1));
worth_query_aspect!(pub Identity for Schema, Record; identity = AspectIdentity(0x919C103), revision = AspectContractRevision(1));
worth_query_aspect!(pub Samples for Schema, Record; identity = AspectIdentity(0x919C104), revision = AspectContractRevision(1));
worth_query_aspect!(pub OtherSamples for Schema, Record; identity = AspectIdentity(0x919C105), revision = AspectContractRevision(1));
worth_query_aspect!(pub VersionedSamples for Schema, Record; identity = AspectIdentity(0x919C106), revision = AspectContractRevision(1));
worth_query_field!(pub OtherValues for Schema, Record, OtherSamples: ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub VersionedValues for Schema, Record, VersionedSamples: ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Key for Schema, Record, Identity: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Values for Schema, Record, Samples: ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub OptionalValues for Schema, Record, Samples: optional ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub UnboundValues for Schema, Record, Samples: optional ContentRefId => ContentRefApplicationValueBinding, read_write, no_equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
pub fn samples() -> ApplicationContentFieldRef<
    Schema,
    Record,
    Samples,
    Values,
    ReadWrite,
    NoEqualityPredicate,
    NoApplicationUnit,
    codec::GridFormat,
> {
    content_field(Values::reference(), codec::GridFormat)
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Record::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Record::reference(), Identity::reference())
        .optional_aspect(Record::reference(), Samples::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Record::reference(), Key::reference())
        .optional_aspect(Record::reference(), OtherSamples::reference())
        .optional_aspect(Record::reference(), VersionedSamples::reference())
        .content_format::<codec::OtherGrid>()
        .content_field(
            Record::reference(),
            content_field(OtherValues::reference(), codec::OtherGrid),
        )
        .content_format::<codec::GridV2>()
        .content_field(
            Record::reference(),
            content_field(VersionedValues::reference(), codec::GridV2),
        )
        .field(Record::reference(), UnboundValues::reference())
        .content_format::<codec::GridFormat>()
        .content_field(Record::reference(), samples())
        .content_field(
            Record::reference(),
            content_field(OptionalValues::reference(), codec::GridFormat),
        )
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .principal_binding(Actor::reference())
}

worth_query_relation!(pub ContainsSamples in Schema,Record=>Record;integrity=same_context_unbounded_retain_dangling);
