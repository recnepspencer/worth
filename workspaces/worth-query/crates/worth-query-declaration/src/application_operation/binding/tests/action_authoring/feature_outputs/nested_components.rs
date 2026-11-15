//! Nested placement reuses a complete component without erasing static branches.
use super::*;

struct Member;
struct Side;
struct Output;
struct Input;
struct ComponentLink;
struct StaticLink;
impl ApplicationFeature<MutationSchema> for Member {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "test.nested.member";
}
impl ApplicationFeature<MutationSchema> for Side {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "test.nested.side";
}
impl ApplicationOutputPort<MutationSchema, Target> for Output {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.nested.output";
}
macro_rules! input {
    ($feature:ty) => {
        impl ApplicationInputPort<MutationSchema, $feature> for Input {
            type Value = RenameInputBinding;
            const IDENTITY: &'static str = "test.nested.input";
            const REQUIRED: bool = false;
        }
    };
}
input!(Member);
input!(Side);
impl ApplicationConnectionIdentity for ComponentLink {
    const IDENTITY: &'static str = "test.nested.component";
}
impl ApplicationConnectionIdentity for StaticLink {
    const IDENTITY: &'static str = "test.nested.static";
}
impl ApplicationOccurrenceConnectionBinding<MutationSchema, Target, Member> for ComponentLink {}
impl ApplicationOccurrenceConnectionBinding<MutationSchema, Target, Side> for StaticLink {}
type RootConnection = <Root as ApplicationOutputGraphShape<MutationSchema>>::RootConnection;
type ComponentConnection =
    ApplicationConnectionRef<MutationSchema, Target, Output, Member, Input, ComponentLink>;
type StaticConnection =
    ApplicationConnectionRef<MutationSchema, Target, Output, Side, Input, StaticLink>;
type All = ApplicationOutputOccurrences<(ComponentConnection,)>;
type Flat = ApplicationOutputGraph<
    RootConnection,
    (
        All,
        ApplicationOutputEdge<StaticConnection, ApplicationOutputLeaf>,
    ),
>;
type Nested = ApplicationOutputGraph<
    RootConnection,
    (All, ApplicationOutputEdge<StaticConnection, (All, All)>),
>;
struct NestedProgram<R>(PhantomData<R>);
impl<R: ApplicationProgramOutputRootsShape<MutationSchema>>
    ApplicationProgramDefinition<MutationSchema> for NestedProgram<R>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<R>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.nested.component.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<MutationSchema, Source>()
                .provides::<Export>()
                .finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Target>()
                .provides::<Output>()
                .finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Member>().finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Side>().finish(),
        ]
    }
}

#[test]
fn nested_exact_component_keeps_static_connections_and_aligned_projections() {
    type Outputs = ApplicationProgramOutputs<Nested>;
    let expected = <Flat as ApplicationProgramOutputShape<MutationSchema>>::connections();
    let expected_types =
        <Flat as ApplicationProgramOutputShape<MutationSchema>>::connection_types();
    assert_eq!(
        <Nested as ApplicationProgramOutputShape<MutationSchema>>::connections(),
        expected
    );
    assert_eq!(
        <Nested as ApplicationProgramOutputShape<MutationSchema>>::connection_types(),
        expected_types
    );
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::connections(),
        expected
    );
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::connection_types(),
        expected_types
    );
    assert_eq!(
        <Outputs as ApplicationProgramOutputsShape<MutationSchema>>::root_graph_types(),
        [TypeId::of::<Nested>()]
    );
    let actual = ApplicationProgramAuthoring::<MutationSchema, NestedProgram<Nested>>::begin()
        .validated_program()
        .unwrap();
    let flat = ApplicationProgramAuthoring::<MutationSchema, NestedProgram<Flat>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(actual.normalized_manifest(), flat.normalized_manifest());
}

fn refuses<R: ApplicationProgramOutputRootsShape<MutationSchema>>() {
    let denied = match ApplicationProgramAuthoring::<MutationSchema, NestedProgram<R>>::begin()
        .validated_program()
    {
        Err(denied) => denied,
        Ok(_) => panic!("duplicate declaration admitted"),
    };
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::DuplicateConnection
    );
}

#[test]
fn first_component_expansion_retains_invalid_duplicate_members() {
    type Invalid = ApplicationOutputOccurrences<(ComponentConnection, ComponentConnection)>;
    refuses::<ApplicationOutputGraph<RootConnection, (Invalid, Invalid)>>();
}

#[test]
fn repeated_component_does_not_hide_duplicate_static_edges() {
    type Static = ApplicationOutputEdge<StaticConnection, All>;
    refuses::<ApplicationOutputGraph<RootConnection, (All, (Static, Static))>>();
}

#[test]
fn different_components_do_not_hide_shared_connection_collisions() {
    type Different = ApplicationOutputOccurrences<
        ApplicationOccurrenceGroup<(ComponentConnection,), (StaticConnection,)>,
    >;
    refuses::<ApplicationOutputGraph<RootConnection, (All, Different)>>();
}
