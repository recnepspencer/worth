use super::*;
use worth_query_declaration::facade::application_query::*;

worth_query_declaration::worth_query_application_query!(
    pub(super) HistoryQuery for MutationSchema, identity "fixture.history.v1",
    parameters TestInputBinding, result TestInputBinding,
    scope FixtureEntity<MutationSchema> => "FixtureEntity", name "correction-history"
);
pub(super) struct HistoryBinding;
impl ApplicationQueryBinding<MutationSchema> for HistoryBinding {
    type Input = TestInput;
    type InputBinding = TestInputBinding;
    type Query = HistoryQuery;
    type ParameterBinding = TestInputBinding;
    type ResultBinding = TestInputBinding;
    type ScopeBinding = ApplicationQueryFieldScope<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixturePrincipalIdentityField<MutationSchema>,
        u64,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = FixtureEntity<MutationSchema>;
    type Principal = FixtureEntity<MutationSchema>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "fixture.history-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixturePrincipalIdentityField<MutationSchema>,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        ApplicationFieldRef::from_schema_types()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        MutationSchema,
        PrincipalBinding,
        FixtureEntity<MutationSchema>,
        FixtureEntity<MutationSchema>,
        u64,
        U64ApplicationValueBinding,
    > {
        test_principal_binding::<MutationSchema>()
    }
}
