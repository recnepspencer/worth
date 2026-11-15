//! Query identity, projection bounds and ordered batch cardinality remain delivery meaning.
use super::*;
use crate::application_query::*;

struct Query<const OTHER: bool>;
impl<const OTHER: bool> ApplicationQueryMarkerIdentity<MutationSchema> for Query<OTHER> {
    type ParameterBinding = RenameInputBinding;
    type ResultBinding = RenameInputBinding;
    type Scope = Account;
    const IDENTIFIER: &'static str = "DeliveryRead";
    const QUERY_TYPE_NAME: &'static str = if OTHER {
        "test.delivery.query.other.v1"
    } else {
        "test.delivery.query.v1"
    };
    const SCOPE_TYPE_NAME: &'static str = "Account";
}
struct Read<const MAX: usize, const OTHER: bool>;
impl<const MAX: usize, const OTHER: bool> ApplicationQueryBinding<MutationSchema>
    for Read<MAX, OTHER>
{
    type Input = RenameInput;
    type InputBinding = RenameInputBinding;
    type Query = Query<OTHER>;
    type ParameterBinding = RenameInputBinding;
    type ResultBinding = RenameInputBinding;
    type ScopeBinding = ApplicationQueryFieldScope<AccountIdField>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Account;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = crate::application_schema::U64ApplicationValueBinding;
    const IDENTITY: &'static str = "test.delivery.query-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(MAX);
    fn scope_field() -> AccountIdField {
        RenameBinding::scope_field()
    }
    fn principal_binding() -> crate::application_schema::ApplicationPrincipalBindingRef<
        MutationSchema,
        PrincipalBinding,
        ExternalMapping,
        Account,
        u64,
        Self::PrincipalIdentityBinding,
    > {
        RenameBinding::principal_binding()
    }
}
struct Projected<const MAX: usize, const OTHER: bool, const N: usize>;
impl<const MAX: usize, const OTHER: bool, const N: usize>
    ApplicationActionDelivery<MutationSchema, RenameBinding> for Projected<MAX, OTHER, N>
{
    type Mode =
        ApplicationFixedDeliveryMode<Root, ApplicationQueryBatchProjection<Read<MAX, OTHER>, N>>;
    const IDENTITY: &'static str = "test.delivery.projected.v1";
}
impl<const MAX: usize, const OTHER: bool, const N: usize>
    ApplicationFixedActionDelivery<MutationSchema, RenameBinding> for Projected<MAX, OTHER, N>
{
    type Root = Root;
    type Projection = ApplicationQueryBatchProjection<Read<MAX, OTHER>, N>;
}
#[test]
fn query_and_exact_projection_bounds_distinguish_shared_definitions() {
    let baseline =
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<1, false, 1>>>::begin()
            .validated_program()
            .unwrap();
    let changes = [
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<2, false, 1>>>::begin()
            .validated_program()
            .unwrap()
            .normalized_manifest(),
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<1, true, 1>>>::begin()
            .validated_program()
            .unwrap()
            .normalized_manifest(),
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<1, false, 2>>>::begin()
            .validated_program()
            .unwrap()
            .normalized_manifest(),
    ];
    let original = baseline.normalized_manifest();
    for changed in changes {
        assert_eq!(
            records(&original, "delivery-connection-v1"),
            records(&changed, "delivery-connection-v1")
        );
        assert_ne!(
            records(&original, "delivery-definition-v1"),
            records(&changed, "delivery-definition-v1")
        );
    }
    for denial in [
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<0, false, 1>>>::begin()
            .validated_program()
            .err()
            .unwrap(),
        ApplicationProgramAuthoring::<MutationSchema, Single<Projected<1, false, 0>>>::begin()
            .validated_program()
            .err()
            .unwrap(),
    ] {
        assert_eq!(
            denial.kind(),
            ApplicationProgramValidationDenialKind::InvalidActionDelivery
        );
    }
}
