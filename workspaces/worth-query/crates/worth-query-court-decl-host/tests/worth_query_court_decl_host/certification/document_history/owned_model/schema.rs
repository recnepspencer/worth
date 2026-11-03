use super::*;
worth_query_application!(pub Schema { owner: "owned-model", version: (1, 0), contributions: [Contribution], });
worth_query_application_contribution! { pub contribution Contribution in Schema {
    identity: "owned-model.contribution.v1", members: |schema| { declare(schema) }
} }
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Model for Schema);
worth_query_entity!(pub Member for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x91820301), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x91820302), revision = AspectContractRevision(1));
worth_query_aspect!(pub ModelFacts for Schema, Model; identity = AspectIdentity(0x91820303), revision = AspectContractRevision(1));
worth_query_aspect!(pub MemberFacts for Schema, Member; identity = AspectIdentity(0x91820304), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalId for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalId for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ModelKey for Schema, Model, ModelFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ModelValue for Schema, Model, ModelFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub MemberKey for Schema, Member, MemberFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub MemberValue for Schema, Member, MemberFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ModelMember in Schema, Model => Member; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Child in Schema, Member => Member; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Reference in Schema, Model => Member; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub Actor in Schema, mapping Mapping { identity: ExternalId, status: Status, target: MappingTarget => Principal, principal_identity: PrincipalId });

fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .entity(Model::reference())
        .entity(Member::reference())
        .aspect(Mapping::reference(), MappingFacts::reference())
        .aspect(Principal::reference(), PrincipalFacts::reference())
        .aspect(Model::reference(), ModelFacts::reference())
        .aspect(Member::reference(), MemberFacts::reference())
        .field(Mapping::reference(), ExternalId::reference())
        .field(Mapping::reference(), Status::reference())
        .field(Principal::reference(), PrincipalId::reference())
        .unique_field(Model::reference(), ModelKey::reference())
        .field(Model::reference(), ModelValue::reference())
        .unique_field(Member::reference(), MemberKey::reference())
        .field(Member::reference(), MemberValue::reference())
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
        .relation(Child::reference(), Member::reference(), Member::reference())
        .relation(
            Reference::reference(),
            Model::reference(),
            Member::reference(),
        );
    query::member::declare(query::model::declare(edit::declare(schema)))
        .declared_operation::<ownership_change::ChangeOwnership>(
            ownership_change::ChangeOwnership::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<ownership_change::Binding>()
        .declared_operation::<generation::RecreateMember>(
            generation::RecreateMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<generation::Binding>()
        .application_history(
            ApplicationHistoryDefinition::<Schema, History>::for_binding(
                ApplicationQueryDisclosureContract::public(),
            )
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .owns(ModelMember::reference())
            .owns(Child::reference())
            .operation::<edit::model::Binding>("Edit model")
            .operation::<ownership_change::Binding>("Move ownership")
            .owned_operation::<edit::member::Binding>("Edit member"),
        )
        .application_history(
            ApplicationHistoryDefinition::<Schema, ResultLimitedHistory>::for_binding(
                ApplicationQueryDisclosureContract::public(),
            )
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .owns(ModelMember::reference())
            .owns(Child::reference())
            .operation::<edit::model::Binding>("Edit model")
            .owned_operation::<edit::member::Binding>("Edit member"),
        )
}
