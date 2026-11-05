//! Its own installed schema: model, owned member and two derived output entities.
use super::*;
worth_query_application!(pub Schema { owner: "owned-outputs", version: (1, 0), contributions: [Contribution], });
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Model for Schema);
worth_query_entity!(pub Member for Schema);
worth_query_entity!(pub FirstOutput for Schema);
worth_query_entity!(pub SecondOutput for Schema);
worth_query_entity!(pub ChildOutput for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x91830301), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x91830302), revision = AspectContractRevision(1));
worth_query_aspect!(pub ModelFacts for Schema, Model; identity = AspectIdentity(0x91830303), revision = AspectContractRevision(1));
worth_query_aspect!(pub MemberFacts for Schema, Member; identity = AspectIdentity(0x91830304), revision = AspectContractRevision(1));
worth_query_aspect!(pub FirstFacts for Schema, FirstOutput; identity = AspectIdentity(0x91830306), revision = AspectContractRevision(1));
worth_query_aspect!(pub ChildFacts for Schema, ChildOutput; identity = AspectIdentity(0x91830308), revision = AspectContractRevision(1));
worth_query_aspect!(pub SecondFacts for Schema, SecondOutput; identity = AspectIdentity(0x91830307), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ModelKey for Schema, Model, ModelFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ModelValue for Schema, Model, ModelFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub MemberKey for Schema, Member, MemberFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub MemberValue for Schema, Member, MemberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub FirstKey for Schema, FirstOutput, FirstFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub FirstValue for Schema, FirstOutput, FirstFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub SecondKey for Schema, SecondOutput, SecondFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub SecondValue for Schema, SecondOutput, SecondFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub ChildKey for Schema, ChildOutput, ChildFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ChildValue for Schema, ChildOutput, ChildFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub ChildOf in Schema, Member => ChildOutput; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ModelMember in Schema, Model => Member; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub FirstOf in Schema, FirstOutput => Model; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub SecondOf in Schema, SecondOutput => Model; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "owned-outputs.contribution.v1", members: |schema| { declare(schema) }
} }

fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Model::reference())
        .entity(Member::reference())
        .entity(FirstOutput::reference())
        .entity(SecondOutput::reference())
        .entity(ChildOutput::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Model::reference(), ModelFacts::reference())
        .aspect(Member::reference(), MemberFacts::reference())
        .aspect(FirstOutput::reference(), FirstFacts::reference())
        .aspect(SecondOutput::reference(), SecondFacts::reference())
        .aspect(ChildOutput::reference(), ChildFacts::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Model::reference(), ModelKey::reference())
        .field(Model::reference(), ModelValue::reference())
        .unique_field(Member::reference(), MemberKey::reference())
        .field(Member::reference(), MemberValue::reference())
        .unique_field(FirstOutput::reference(), FirstKey::reference())
        .field(FirstOutput::reference(), FirstValue::reference())
        .unique_field(SecondOutput::reference(), SecondKey::reference())
        .field(SecondOutput::reference(), SecondValue::reference())
        .unique_field(ChildOutput::reference(), ChildKey::reference())
        .field(ChildOutput::reference(), ChildValue::reference())
        .relation(
            ChildOf::reference(),
            Member::reference(),
            ChildOutput::reference(),
        )
        .principal_binding(Actor::reference())
        .relation(
            MappingTarget::reference(),
            Mapping::reference(),
            Principal::reference(),
        )
        .relation(
            ModelMember::reference(),
            Model::reference(),
            Member::reference(),
        )
        .relation(
            FirstOf::reference(),
            FirstOutput::reference(),
            Model::reference(),
        )
        .relation(
            SecondOf::reference(),
            SecondOutput::reference(),
            Model::reference(),
        );
    let schema = query::declare(member_delete::declare(member_edit::declare(edit::declare(
        navigation::declare(schema),
    ))));
    restricted::declare(child::declare(second::declare(first::declare(
        permission::declare(schema),
    ))))
    .application_history(
        ApplicationHistoryDefinition::<Schema, History>::for_binding(
            ApplicationQueryDisclosureContract::public(),
        )
        .model_domain(ApplicationHistoryModelDomain::OwnedModelGraph)
        .navigation::<navigation::Navigation>()
        .owns(ModelMember::reference())
        .owns(ChildOf::reference())
        .operation::<edit::Binding>("Edit model")
        .owned_operation::<member_edit::Binding>("Edit member")
        .operation::<member_delete::Binding>("Delete member"),
    )
}
