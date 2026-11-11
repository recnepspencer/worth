//! Query authoring remains infallible in both manual and complete feature hooks.
use super::*;
type S = Model<0>;
worth_query_application_query!(FeatureQuery for Schema: Scenario, identity "ownership.feature-query.v1", parameters query::UnitBinding, result query::RowBinding, scope Node<Schema> => "Node", name "feature_query");
fn definition<S: Scenario, const INVALID: bool>(
) -> ApplicationQueryDefinitionBuilder<S, FeatureQuery, (), query::Row, Node<S>> {
    ApplicationQueryDefinitionBuilder::declare(FeatureQuery::reference())
        .root(schema::node::<S>())
        .scope(schema::node::<S>())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(schema::node::<S>())
                .field(ApplicationQueryResultFieldRef::<
                    FeatureQuery,
                    IdSlot,
                    S,
                    Id<S>,
                >::new("id", schema::id::<S>()))
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(if INVALID {
            ApplicationQueryLaneEligibility::one_shot().with_preview()
        } else {
            ApplicationQueryLaneEligibility::one_shot()
        })
        .public()
}
pub(super) struct Manual<S, const INVALID: bool>(PhantomData<fn() -> S>);
impl<S: Scenario, const INVALID: bool> ApplicationFeatureDefinition<S> for Manual<S, INVALID> {
    type Outputs = ApplicationNoOutputGraph;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        schema
            .application_query(definition::<S, INVALID>())
            .application_query_binding::<Binding<S>>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Vec::new()
    }
}
worth_query_host::facade::worth_query_application_contribution! {
    feature Complete<S: Scenario> for program::Entry => Runtime {
        schema: |schema| { schema.application_query(definition::<S, true>()) };
    }
}
fn builder() -> ApplicationSchemaDeclarationBuilder<S> {
    ApplicationSchemaDeclarationBuilder::from_fragment(OwnershipRoot::<S, false>(PhantomData))
        .unwrap()
}
#[test]
fn an_invalid_query_in_a_feature_retains_its_owner_and_original_cause() {
    let denial = Manual::<S, true>::declare_members(builder())
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidQueryDefinition {
            owner: std::any::type_name::<Manual<S, true>>().to_owned(),
            query: "feature_query".into(),
            cause: ApplicationQueryDefinitionDenial::PreviewLaneWithoutBasisSupport,
        }
    );
}
#[test]
fn a_lawful_query_in_a_feature_installs() {
    // Mode 71 registers this feature inside the contribution's real member closure.
    let schema = Model::<71>::declaration().unwrap();
    let installed = in_memory(schema, ((),), ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000008").unwrap(), host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), |_, graph, installed| {
        graph.bind_principal(
            &installed.principal_binding(actor::<Model<71>>()).unwrap(),
            WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
            1,
            WorthQueryExternalPrincipalIdentity::new("https://ownership.invalid", "operator").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )?;
        Ok(())
    });
    assert!(installed.is_ok(), "{:?}", installed.err());
}

#[test]
fn an_invalid_query_in_a_complete_feature_is_refused_at_composition() {
    let denial = Complete::<S>::declare_members(builder())
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidQueryDefinition {
            owner: std::any::type_name::<Complete<S>>().to_owned(),
            query: "feature_query".into(),
            cause: ApplicationQueryDefinitionDenial::PreviewLaneWithoutBasisSupport,
        }
    );
}

struct IdSlot;
worth_query_portable_type!(IdSlot => "ownership.feature-query.id-slot.v1");
struct Binding<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationQueryBinding<S> for Binding<S> {
    type Input = query::Input;
    type InputBinding = query::InputBinding;
    type Query = FeatureQuery;
    type ParameterBinding = query::UnitBinding;
    type ResultBinding = query::RowBinding;
    type ScopeBinding = query::Scope<S>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node<S>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "ownership.feature-query.binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(16);
    fn scope_field() -> Id<S> {
        schema::id::<S>()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding>
    {
        actor()
    }
}
