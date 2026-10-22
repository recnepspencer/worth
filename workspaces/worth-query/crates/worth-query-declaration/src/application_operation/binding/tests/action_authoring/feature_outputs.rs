//! Root selection must happen before global connection validation.
use super::*;
use std::marker::PhantomData;

type Repeated = ApplicationFeatureOutputs<(
    ApplicationActionOutput<RenameBinding, Delivery>,
    (
        ApplicationActionOutput<RenameBinding, Delivery>,
        ApplicationActionOutput<RenameBinding, Delivery>,
    ),
)>;
type DifferentRoot = ApplicationOutputGraph<
    <Root as ApplicationOutputGraphShape<MutationSchema>>::RootConnection,
    (ApplicationOutputLeaf, ApplicationOutputLeaf),
>;
struct DifferentDelivery;
impl ApplicationActionDelivery<MutationSchema, RenameBinding> for DifferentDelivery {
    type Mode = ApplicationFixedDeliveryMode<DifferentRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "test.action.different-full-root.v1";
}
impl ApplicationFixedActionDelivery<MutationSchema, RenameBinding> for DifferentDelivery {
    type Root = DifferentRoot;
    type Projection = ApplicationNoProjection;
}
type Conflicting = ApplicationFeatureOutputs<(
    ApplicationActionOutput<RenameBinding, Delivery>,
    ApplicationActionOutput<RenameBinding, DifferentDelivery>,
)>;

struct Projected<Roots>(PhantomData<Roots>);
impl<Roots: ApplicationProgramOutputRootsShape<MutationSchema>>
    ApplicationProgramDefinition<MutationSchema> for Projected<Roots>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<Roots>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        <Program<false> as ApplicationProgramDefinition<MutationSchema>>::IDENTITY;
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::<false>::feature_specs()
    }
}

#[test]
fn repeated_action_deliveries_keep_the_actual_root_once_before_flattening() {
    type Outputs = ApplicationProgramOutputs<Repeated>;
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::root_graph_types(),
        [TypeId::of::<Root>()]
    );
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::connections(),
        <Root as ApplicationProgramOutputShape<MutationSchema>>::connections()
    );
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::connection_types(),
        <Root as ApplicationProgramOutputShape<MutationSchema>>::connection_types()
    );
    let actual = ApplicationProgramAuthoring::<MutationSchema, Projected<Repeated>>::begin()
        .validated_program()
        .unwrap();
    let expected = ApplicationProgramAuthoring::<MutationSchema, Program<false>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(actual.normalized_manifest(), expected.normalized_manifest());
}

#[test]
fn different_full_roots_cannot_hide_a_duplicate_connection() {
    assert_ne!(TypeId::of::<Root>(), TypeId::of::<DifferentRoot>());
    assert_eq!(
        <ApplicationProgramOutputs<Conflicting> as ApplicationProgramOutputsShape<
            MutationSchema,
        >>::root_graph_types(),
        [TypeId::of::<Root>(), TypeId::of::<DifferentRoot>()]
    );
    let denied =
        match ApplicationProgramAuthoring::<MutationSchema, Projected<Conflicting>>::begin()
            .validated_program()
        {
            Err(denied) => denied,
            Ok(_) => panic!("different roots sharing a connection must be refused"),
        };
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::DuplicateConnection
    );
}
