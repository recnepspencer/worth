//! Nested placement reuses a complete component without erasing static branches.
use super::*;

struct Member;
struct Side;
struct Output;
struct Input;
struct StaticInput;
struct BackInput;
struct ComponentLink;
struct StaticLink;
impl ApplicationFeature<MutationSchema> for Member {
    type Inputs = ApplicationFeatureInputList<
        Input,
        ApplicationFeatureInputList<
            StaticInput,
            ApplicationFeatureInputList<BackInput, ApplicationFeatureInputLeaf>,
        >,
    >;
    const IDENTITY: &'static str = "test.nested.member";
}
impl ApplicationFeature<MutationSchema> for Side {
    type Inputs = ApplicationFeatureInputList<
        Input,
        ApplicationFeatureInputList<StaticInput, ApplicationFeatureInputLeaf>,
    >;
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
macro_rules! static_ports {
    ($feature:ty) => {
        impl ApplicationInputPort<MutationSchema, $feature> for StaticInput {
            type Value = RenameInputBinding;
            const IDENTITY: &'static str = "test.nested.static-input";
            const REQUIRED: bool = false;
        }
        impl ApplicationOutputPort<MutationSchema, $feature> for Output {
            type Value = RenameInputBinding;
            const IDENTITY: &'static str = "test.nested.output";
        }
    };
}
static_ports!(Member);
static_ports!(Side);
impl ApplicationInputPort<MutationSchema, Member> for BackInput {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.nested.back-input";
    const REQUIRED: bool = false;
}
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
/// Declares a named occurrence component over its structural connections.
macro_rules! component {
    ($name:ident = $connections:ty) => {
        struct $name;
        impl ApplicationOccurrenceComponent<MutationSchema> for $name {
            type Connections = $connections;
        }
    };
}
component!(Members = (ComponentConnection,));
type All = ApplicationOutputOccurrences<Members>;
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
            ApplicationFeatureSpec::root::<MutationSchema, Member>()
                .provides::<Output>()
                .finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Side>()
                .provides::<Output>()
                .finish(),
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
    component!(Duplicated = (ComponentConnection, ComponentConnection));
    type Invalid = ApplicationOutputOccurrences<Duplicated>;
    refuses::<ApplicationOutputGraph<RootConnection, (Invalid, Invalid)>>();
}

#[test]
fn repeated_component_does_not_hide_duplicate_static_edges() {
    type Static = ApplicationOutputEdge<StaticConnection, All>;
    refuses::<ApplicationOutputGraph<RootConnection, (All, (Static, Static))>>();
}

#[test]
fn different_components_do_not_hide_shared_connection_collisions() {
    component!(
        Overlapping = ApplicationOccurrenceGroup<(ComponentConnection,), (StaticConnection,)>
    );
    type Different = ApplicationOutputOccurrences<Overlapping>;
    refuses::<ApplicationOutputGraph<RootConnection, (All, Different)>>();
}

#[test]
fn a_component_is_its_name_not_its_connections() {
    // Identical structural connections under a second name are a different
    // component; their shared connection collides instead of merging.
    component!(Twin = (ComponentConnection,));
    type Renamed = ApplicationOutputOccurrences<Twin>;
    refuses::<ApplicationOutputGraph<RootConnection, (All, Renamed)>>();
}

macro_rules! connection {
    ($binding:ident, $name:ident, $id:literal, $source:ty, $output:ty, $target:ty, $input:ty) => {
        struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $id;
        }
        impl ApplicationOccurrenceConnectionBinding<MutationSchema, $source, $target> for $binding {}
        type $name = ApplicationConnectionRef<MutationSchema, $source, $output, $target, $input, $binding>;
    };
}
connection!(
    SideComponent,
    SideComponentConnection,
    "test.mixed.side-component",
    Target,
    Output,
    Side,
    Input
);
connection!(
    MemberStatic,
    MemberStaticConnection,
    "test.mixed.member-static",
    Target,
    Output,
    Member,
    StaticInput
);
connection!(
    SideStatic,
    SideStaticConnection,
    "test.mixed.side-static",
    Member,
    Output,
    Side,
    StaticInput
);
connection!(
    BackStatic,
    BackStaticConnection,
    "test.mixed.back-static",
    Side,
    Output,
    Member,
    BackInput
);
connection!(
    SeparateComponent,
    SeparateComponentConnection,
    "test.mixed.separate-component",
    Source,
    Export,
    Side,
    Input
);

component!(Mixed = (ComponentConnection, SideComponentConnection));
type Complete = ApplicationOutputOccurrences<Mixed>;

#[test]
fn internal_static_dag_keeps_its_runnable_precedence() {
    type Tree = ApplicationOutputGraph<
        RootConnection,
        (
            Complete,
            ApplicationOutputEdge<
                MemberStaticConnection,
                ApplicationOutputEdge<SideStaticConnection, Complete>,
            >,
        ),
    >;
    let program = ApplicationProgramAuthoring::<MutationSchema, NestedProgram<Tree>>::begin()
        .validated_program()
        .unwrap();
    let order = program.connection_order();
    let member = order.rank(&MemberStaticConnection::declaration()).unwrap();
    let side = order.rank(&SideStaticConnection::declaration()).unwrap();
    assert!(
        member < side,
        "internal static precedence survives occurrence condensation"
    );
}

fn refuses_cycle<R: ApplicationProgramOutputRootsShape<MutationSchema>>() {
    let denied = match ApplicationProgramAuthoring::<MutationSchema, NestedProgram<R>>::begin()
        .validated_program()
    {
        Err(denied) => denied,
        Ok(_) => panic!("static cycle admitted"),
    };
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::CyclicConnection
    );
}

#[test]
fn condensation_cannot_hide_an_internal_static_cycle() {
    type Tree = ApplicationOutputGraph<
        RootConnection,
        (
            Complete,
            ApplicationOutputEdge<
                MemberStaticConnection,
                ApplicationOutputEdge<
                    SideStaticConnection,
                    ApplicationOutputEdge<BackStaticConnection, Complete>,
                >,
            >,
        ),
    >;
    refuses_cycle::<Tree>();
}

#[test]
fn static_dag_cannot_cycle_between_condensed_groups() {
    // Source->Target and Member->Side are statically acyclic, but the two
    // occurrence groups {Source,Side} and {Target,Member} depend on each other.
    component!(Separate = (ComponentConnection, SeparateComponentConnection));
    type Groups = ApplicationOutputOccurrences<Separate>;
    type Tree = ApplicationOutputGraph<
        RootConnection,
        (
            Groups,
            ApplicationOutputEdge<
                MemberStaticConnection,
                ApplicationOutputEdge<SideStaticConnection, ApplicationOutputLeaf>,
            >,
        ),
    >;
    refuses_cycle::<Tree>();
}
