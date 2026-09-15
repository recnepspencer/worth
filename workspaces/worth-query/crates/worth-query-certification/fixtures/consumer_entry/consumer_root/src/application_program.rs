use worth_query_decl::facade::application_program::{
    ApplicationConnectionRef, ApplicationFeatureDeclaration, ApplicationLocalRuleRef,
    ApplicationPortRef, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationSharedRuleRef,
};
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_parameter_entry::{ParameterFeature, PositiveParameterCount};
use worth_query_topology_entry::{
    PlanarBodyInput, PlanarBodyOutput, PlanarOutputFeature, PlanarSourceFeature,
    PlanarSourceToOutputConnection, PositivePlanarTurn, PlanarDerivedBodyInput,
    PlanarDerivedBodyOutput, PlanarFinalOutputFeature, PlanarOutputToFinalConnection,
};

use crate::ConsumerSchema;

pub struct ConsumerProgram;

type PlanarConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;

pub const PLANAR_CONNECTION: PlanarConnection =
    PlanarConnection::connect(ApplicationPortRef::new(), ApplicationPortRef::new());

type PlanarDependentConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;

pub const PLANAR_DEPENDENT_CONNECTION: PlanarDependentConnection =
    PlanarDependentConnection::connect(ApplicationPortRef::new(), ApplicationPortRef::new());

pub fn validated_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        ConsumerProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<ConsumerSchema, ConsumerProgram>::begin()
        .connect(PLANAR_CONNECTION)
        .connect_dependent(PLANAR_DEPENDENT_CONNECTION)
        .local_rule(
            ApplicationLocalRuleRef::<ConsumerSchema, ParameterFeature, PositiveParameterCount>::new(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )
        .shared_rule(
            ApplicationSharedRuleRef::<ConsumerSchema, PositivePlanarTurn>::new(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )
        .validated_program()
}

const FEATURES: &[ApplicationFeatureDeclaration] = &[
    ApplicationFeatureDeclaration::new("worth.query.certification.planar-source-feature.v1", &[]),
    ApplicationFeatureDeclaration::new(
        "worth.query.certification.planar-output-feature.v1",
        &["source"],
    ),
    ApplicationFeatureDeclaration::new(
        "worth.query.certification.planar-final-output-feature.v1",
        &["source"],
    ),
    ApplicationFeatureDeclaration::new("worth.query.certification.parameter-feature.v1", &[]),
];

impl ApplicationProgramDefinition<ConsumerSchema> for ConsumerProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Connections = PlanarSourceToOutputConnection;
    type DependentConnection = PlanarOutputToFinalConnection;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.consumer-program.v1");

    fn features() -> &'static [ApplicationFeatureDeclaration] {
        FEATURES
    }
}
