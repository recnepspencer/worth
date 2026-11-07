//! Both infallible binding callbacks retain field expectations at composition.
use super::{lifecycle::*, *};
use worth_query_decl::facade::application_schema::fragment::*;
type S = Model<0>;
fn invalid_actor(
) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding> {
    ApplicationPrincipalBindingRef::from_requirements(
        "Actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                record_field(
                    OwnershipModel::<false>,
                    Mapping,
                    MappingFacts,
                    "external_typo",
                ),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                record_field(OwnershipModel::<false>, Mapping, MappingFacts, "Status"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(MapsTo::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                OwnershipModel::<false>,
                RawId::<false>::at("Id"),
            ),
        },
    )
}
struct BadMutation;
impl ApplicationMutationBinding<S> for BadMutation {
    type Input = Create;
    type InputBinding = CreateInput;
    type Result = ();
    type ResultBinding = query::UnitBinding;
    type IdempotencyKey = String;
    type Operation = CreateOp;
    type Decision = CreationTargets<S>;
    type Denial = ();
    type DenialBinding = query::UnitBinding;
    type Output = CreatedOutputs;
    type ScopeBinding = <CreateBinding<S> as ApplicationMutationBinding<S>>::ScopeBinding;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node<S>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "principal.callback.mutation.v1";
    const HANDLER_IDENTITY: &'static str = "principal.callback.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "ownership.command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Id<S> {
        schema::id::<S>()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding>
    {
        invalid_actor()
    }
}
struct BadQuery;
impl ApplicationQueryBinding<S> for BadQuery {
    type Input = query::Input;
    type InputBinding = query::InputBinding;
    type Query = query::Query;
    type ParameterBinding = query::UnitBinding;
    type ResultBinding = query::RowBinding;
    type ScopeBinding = query::Scope<S>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node<S>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "principal.callback.query.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(16);
    fn scope_field() -> Id<S> {
        schema::id::<S>()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding>
    {
        invalid_actor()
    }
}
fn require_refusal(
    result: Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial>,
) {
    let expected = ExternalId::at("external_typo")
        .binding_recipe()
        .locus()
        .clone();
    assert_eq!(
        result.unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidPrincipalBindingField {
            binding: "Actor".into(),
            field: expected
        }
    );
}
#[test]
fn mutation_principal_callback_is_checked_at_composition() {
    let builder = ApplicationSchemaDeclarationBuilder::<S>::from_fragment(
        OwnershipRoot::<S, false>(PhantomData),
    )
    .unwrap();
    require_refusal(
        builder
            .application_mutation_binding::<BadMutation>()
            .build(),
    );
}
#[test]
fn query_principal_callback_is_checked_at_composition() {
    let builder = ApplicationSchemaDeclarationBuilder::<S>::from_fragment(
        OwnershipRoot::<S, false>(PhantomData),
    )
    .unwrap();
    require_refusal(builder.application_query_binding::<BadQuery>().build());
}
